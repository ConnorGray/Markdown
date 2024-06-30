//! Convert from "unflattened" [`pulldown_cmark::Event`]s to AST [`Block`]s.

use pulldown_cmark::{Event, Tag};

use std::mem;

use crate::{
    unflatten::UnflattenedEvent, Block, CodeBlockKind, Inline, Inlines,
    ListItem,
};

//======================================
// AST Builder
//======================================

pub(crate) fn ast_events_to_ast(events: Vec<UnflattenedEvent>) -> Vec<Block> {
    let mut complete: Vec<Block> = vec![];

    let mut text_spans: Vec<Inline> = vec![];

    for event in events {
        // println!("event: {:?}", event);

        if !is_inline(&event) {
            if !text_spans.is_empty() {
                complete.push(Block::Paragraph(Inlines(mem::replace(
                    &mut text_spans,
                    vec![],
                ))));
            }
        }

        match event {
            UnflattenedEvent::Event(event) => match event {
                Event::Start(_) | Event::End(_) => {
                    panic!("illegal Event::{{Start, End}} in UnflattenedEvent::Event")
                },
                Event::Text(text) => {
                    text_spans.push(Inline::Text(text.to_string()))
                },
                Event::Code(code) => {
                    text_spans.push(Inline::Code(code.to_string()))
                },
                Event::SoftBreak => text_spans.push(Inline::SoftBreak),
                Event::HardBreak => text_spans.push(Inline::HardBreak),
                Event::Html(_) => todo!("error: unhandled inline HTML"),
                Event::InlineHtml(_) => todo!(),
                Event::Rule => complete.push(Block::Rule),
                Event::TaskListMarker(_) | Event::FootnoteReference(_) => {
                    todo!("handle: {event:?}")
                },
                Event::InlineMath(_) => todo!(),
                Event::DisplayMath(_) => todo!(),
            },
            UnflattenedEvent::Nested { tag, events } => {
                match tag {
                    //
                    // Inline content
                    //
                    Tag::Emphasis => {
                        text_spans.push(Inline::Emphasis(unwrap_text(events)));
                    },
                    Tag::Strong => {
                        text_spans.push(Inline::Strong(unwrap_text(events)));
                    },
                    Tag::Strikethrough => {
                        text_spans
                            .push(Inline::Strikethrough(unwrap_text(events)));
                    },

                    Tag::Link {
                        link_type,
                        dest_url,
                        title,
                        id,
                    } => {
                        let content_text = unwrap_text(events);

                        text_spans.push(Inline::Link {
                            link_type,
                            dest_url: dest_url.to_string(),
                            title: title.to_string(),
                            id: id.to_string(),
                            content_text,
                        })
                    },

                    //
                    // Block content
                    //

                    // TODO: Use the two Heading fields that are ignored here?
                    Tag::Heading {
                        level,
                        id: _,
                        classes: _,
                        attrs: _,
                    } => {
                        complete
                            .push(Block::Heading(level, unwrap_text(events)));
                    },
                    // TODO(test):
                    //     Is this disappearance of the Paragraph tag correct?
                    Tag::Paragraph => text_spans.extend(unwrap_text(events)),
                    // TODO: Include the list start number in the metadata
                    Tag::List(_start) => {
                        let mut items: Vec<ListItem> = Vec::new();

                        for event in events {
                            if let UnflattenedEvent::Nested {
                                tag: Tag::Item,
                                events: item_events,
                            } = event
                            {
                                let item_blocks =
                                    ast_events_to_ast(item_events);
                                items.push(ListItem(item_blocks));
                            } else {
                                todo!("handle list element: {event:?}");
                            }
                        }

                        complete.push(Block::List(items));
                    },
                    Tag::Item => {
                        complete.extend(ast_events_to_ast(events));
                    },
                    Tag::CodeBlock(kind) => {
                        let text_spans = unwrap_text(events);
                        let code_text = text_to_string(&text_spans);

                        let kind = CodeBlockKind::from_pulldown_cmark(kind);

                        complete.push(Block::CodeBlock {
                            kind,
                            code: code_text,
                        })
                    },
                    Tag::BlockQuote(kind) => {
                        let blocks = ast_events_to_ast(events);
                        complete.push(Block::BlockQuote { kind, blocks })
                    },
                    Tag::Table(alignments) => {
                        let mut events = events.into_iter();
                        let header_events = match events.next().unwrap() {
                            UnflattenedEvent::Event(_) => panic!(),
                            UnflattenedEvent::Nested { tag, events } => {
                                assert!(tag == Tag::TableHead);
                                events
                            },
                        };

                        let mut headers = Vec::new();

                        for table_cell in header_events {
                            let table_cell_text =
                                unwrap_text(unwrap_table_cell(table_cell));

                            headers.push(table_cell_text);
                        }

                        let mut rows = Vec::new();

                        for row_events in events {
                            let row_events = match row_events {
                                UnflattenedEvent::Event(_) => panic!(),
                                UnflattenedEvent::Nested { tag, events } => {
                                    assert!(tag == Tag::TableRow);
                                    events
                                },
                            };

                            let mut row = Vec::new();

                            for table_cell in row_events {
                                let table_cell_text =
                                    unwrap_text(unwrap_table_cell(table_cell));

                                row.push(table_cell_text);
                            }

                            rows.push(row);
                        }

                        complete.push(Block::Table {
                            alignments,
                            headers,
                            rows,
                        })
                    },
                    _ => todo!("handle: {tag:?}"),
                }
            },
        }
    }

    if !text_spans.is_empty() {
        complete.push(Block::paragraph(text_spans));
    }

    complete
}

/// Returns `true` if `event` contains content that can be added "inline" with text
/// content.
///
/// `event`'s that cannot be added inline will start a new [`Block`].
fn is_inline(event: &UnflattenedEvent) -> bool {
    match event {
        UnflattenedEvent::Event(event) => match event {
            Event::Start(_) | Event::End(_) => unreachable!(),
            Event::Text(_) => true,
            Event::Code(_) => true,
            Event::SoftBreak => true,
            Event::HardBreak => true,
            // TODO: HTML could cause break to next block?
            Event::Html(_) => false,
            Event::InlineHtml(_) => todo!(),
            Event::Rule => false,
            Event::TaskListMarker(_) => false,
            Event::FootnoteReference(_) => true,
            Event::InlineMath(_) => todo!(),
            Event::DisplayMath(_) => todo!(),
        },
        UnflattenedEvent::Nested { tag, events: _ } => match tag {
            Tag::Emphasis | Tag::Strong | Tag::Strikethrough => true,
            Tag::Heading {
                level: _,
                id: _,
                classes: _,
                attrs: _,
            } => false,
            Tag::Paragraph => false,
            Tag::List(_) => false,
            Tag::Item => false,
            Tag::CodeBlock(_) => false,
            Tag::BlockQuote(_kind) => false,
            Tag::Table(_) => false,
            Tag::TableHead | Tag::TableRow => unreachable!(),
            Tag::Link { .. } => true,
            _ => todo!("handle tag: {tag:?}"),
        },
    }
}

fn unwrap_text(events: Vec<UnflattenedEvent>) -> Inlines {
    let mut text_spans: Vec<Inline> = vec![];

    for event in events {
        match event {
            UnflattenedEvent::Event(event) => match event {
                Event::Start(_) | Event::End(_) => unreachable!(),
                Event::Text(text) => {
                    text_spans.push(Inline::Text(text.to_string()))
                },
                Event::Code(code) => {
                    text_spans.push(Inline::Code(code.to_string()))
                },
                Event::SoftBreak => text_spans.push(Inline::SoftBreak),
                Event::HardBreak => text_spans.push(Inline::HardBreak),
                Event::Html(_) => todo!("error: skipping inline HTML"),
                Event::InlineHtml(_) => todo!(),
                Event::TaskListMarker(_)
                | Event::Rule
                | Event::FootnoteReference(_) => {
                    todo!("handle: {event:?}")
                },
                Event::InlineMath(_) => todo!(),
                Event::DisplayMath(_) => todo!(),
            },
            UnflattenedEvent::Nested { tag, events } => match tag {
                Tag::Emphasis => {
                    text_spans.push(Inline::Emphasis(unwrap_text(events)));
                },
                Tag::Strong => {
                    text_spans.push(Inline::Strong(unwrap_text(events)));
                },
                Tag::Strikethrough => {
                    text_spans.push(Inline::Strikethrough(unwrap_text(events)));
                },
                Tag::Paragraph => {
                    // If this is a separate paragraph, insert two hardbreaks
                    // (two newlines). Don't insert hardbreaks if there isn't any existing
                    // text content, to avoid having leading empty lines.
                    if !text_spans.is_empty() {
                        // TODO: Replace this with a new Inline::ParagraphBreak?
                        //       A HardBreak is just a newline.
                        text_spans.push(Inline::HardBreak);
                        text_spans.push(Inline::HardBreak);
                    }
                    text_spans.extend(unwrap_text(events))
                },
                Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                } => {
                    let content_text = unwrap_text(events);

                    text_spans.push(Inline::Link {
                        link_type,
                        dest_url: dest_url.to_string(),
                        title: title.to_string(),
                        id: id.to_string(),
                        content_text,
                    })
                },
                Tag::Image {
                    link_type,
                    dest_url,
                    title,
                    id
                } => {
                    let image_description = unwrap_text(events);

                    text_spans.push(Inline::Image {
                        link_type,
                        dest_url: dest_url.to_string(),
                        title: title.to_string(),
                        id: id.to_string(),
                        image_description,
                    })
                },

                //--------------------------
                // Illegal in inline content
                //--------------------------

                Tag::Heading { .. }
                | Tag::BlockQuote(_)
                | Tag::CodeBlock(_)
                | Tag::HtmlBlock
                | Tag::List(_)
                | Tag::Item
                | Tag::FootnoteDefinition(_)
                | Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell
                | Tag::MetadataBlock(_) => panic!(
                    "unexpected non-inline element inside inlines parse context: {tag:?}"
                ),
            },
        }
    }

    Inlines(text_spans)
}

fn unwrap_table_cell(event: UnflattenedEvent) -> Vec<UnflattenedEvent> {
    match event {
        UnflattenedEvent::Event(_) => panic!(),
        UnflattenedEvent::Nested { tag, events } => {
            assert_eq!(tag, Tag::TableCell, "expected to get Tag::TableCell");
            events
        },
    }
}

fn text_to_string(Inlines(text_spans): &Inlines) -> String {
    let mut string = String::new();

    for span in text_spans {
        match span {
            Inline::Text(text) => {
                string.push_str(&text);
            },
            Inline::SoftBreak => {
                string.push_str(" ");
            },
            Inline::HardBreak => {
                string.push_str("\n");
            },
            _ => todo!("handle span: {span:?}"),
        }
    }

    string
}
