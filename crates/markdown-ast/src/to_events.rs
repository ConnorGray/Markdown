//! Convert from AST [`Block`]s to "flattened" [`pulldown_cmark::Event`]s.

use pulldown_cmark::{CowStr, Event, Tag};

use crate::{Block, Inline, Inlines, ListItem};

//======================================
// AST blocks to Events
//======================================

pub(crate) fn block_to_events<'ast>(
    block: &'ast Block,
    events: &mut Vec<Event<'ast>>,
) {
    match block {
        Block::Paragraph(inlines) => wrap(Tag::Paragraph, events, |events| {
            inlines_to_events(inlines, events)
        }),
        Block::List(list_items) => {
            // TODO: Handle this for numbered lists.
            let first_item_number = None;

            wrap(Tag::List(first_item_number), events, |events| {
                for ListItem(list_item_blocks) in list_items {
                    wrap(Tag::Item, events, |events| {
                        // NOTE:
                        //  Handle a special case where a single-item list
                        //  containing a sequence of inlines is parsed by
                        //  clap-markdown NOT wrapped in paired
                        //  Start(Tag::Paragraph) / End(_) events.
                        match list_item_blocks.as_slice() {
                            [Block::Paragraph(inlines)]
                                if list_items.len() == 1 =>
                            {
                                inlines_to_events(inlines, events);

                                // Return from inner closure.
                                return;
                            },
                            _ => (),
                        }

                        for list_item_block in list_item_blocks {
                            block_to_events(list_item_block, events);
                        }
                    });
                }
            })
        },
        Block::Heading(level, inlines) => {
            let tag = Tag::Heading {
                level: *level,
                // FIXME: Set this id.
                id: None,
                // FIXME: Support these classes and attrs.
                classes: Vec::new(),
                attrs: Vec::new(),
            };

            wrap(tag, events, |events| inlines_to_events(inlines, events));
        },
        Block::CodeBlock { kind, code } => {
            let kind = kind.to_pulldown_cmark();

            wrap(Tag::CodeBlock(kind), events, |events| {
                // FIXME: Is this the right event for raw codeblock content?
                events.push(Event::Text(CowStr::from(code.as_str())))
            })
        },
        Block::BlockQuote { kind, blocks } => {
            wrap(Tag::BlockQuote(*kind), events, |events| {
                for block in blocks {
                    block_to_events(block, events)
                }
            })
        },
        Block::Table {
            alignments,
            headers,
            rows,
        } => {
            // Structure of a table in Events:
            //
            // * Tag::Table
            //   * Tag::TableHead
            //     * Tag::TableCell...
            //   * Tag::TableRow
            //     * Tag::TableCell...

            wrap(Tag::Table(alignments.clone()), events, |events| {
                wrap(Tag::TableHead, events, |events| {
                    for header_cell in headers {
                        wrap(Tag::TableCell, events, |events| {
                            inlines_to_events(header_cell, events);
                        })
                    }
                });

                for row in rows {
                    wrap(Tag::TableRow, events, |events| {
                        for row_cell in row {
                            wrap(Tag::TableCell, events, |events| {
                                inlines_to_events(row_cell, events);
                            })
                        }
                    })
                }
            })
        },
        Block::Rule => events.push(Event::Rule),
    }
}

fn wrap<'ast, F: FnOnce(&mut Vec<Event<'ast>>)>(
    tag: Tag<'ast>,
    events: &mut Vec<Event<'ast>>,
    action: F,
) {
    let end = tag.to_end();

    events.push(Event::Start(tag));
    action(events);
    events.push(Event::End(end));
}

fn inlines_to_events<'ast>(
    inlines: &'ast Inlines,
    events: &mut Vec<Event<'ast>>,
) {
    let Inlines(inlines) = inlines;

    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                events.push(Event::Text(CowStr::from(text.as_str())));
            },
            Inline::Emphasis(inlines) => {
                wrap(Tag::Emphasis, events, |events| {
                    inlines_to_events(inlines, events)
                })
            },
            Inline::Strong(inlines) => wrap(Tag::Strong, events, |events| {
                inlines_to_events(inlines, events)
            }),
            Inline::Strikethrough(inlines) => {
                wrap(Tag::Strikethrough, events, |events| {
                    inlines_to_events(inlines, events)
                })
            },
            Inline::Code(code) => {
                events.push(Event::Code(CowStr::from(code.as_str())))
            },
            Inline::Link {
                link_type,
                dest_url,
                title,
                id,
                content_text,
            } => wrap(
                Tag::Link {
                    link_type: *link_type,
                    dest_url: CowStr::from(dest_url.as_str()),
                    // FIXME:
                    //  Pass through this title; have a test that fails
                    //  if this is empty.
                    title: CowStr::from(title.as_str()),
                    // FIXME: Passthrough this id
                    // FIXME:
                    //  Add test for the value of this field for every
                    //  link type.
                    id: CowStr::from(id.as_str()),
                },
                events,
                |events| inlines_to_events(content_text, events),
            ),
            Inline::Image {
                link_type,
                dest_url,
                title,
                id,
                image_description,
            } => wrap(
                Tag::Image {
                    link_type: *link_type,
                    dest_url: CowStr::from(dest_url.as_str()),
                    title: CowStr::from(title.as_str()),
                    id: CowStr::from(id.as_str()),
                },
                events,
                |events| inlines_to_events(image_description, events),
            ),
            Inline::SoftBreak => events.push(Event::SoftBreak),
            Inline::HardBreak => events.push(Event::HardBreak),
        }
    }
}
