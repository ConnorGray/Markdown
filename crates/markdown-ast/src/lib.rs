//! Parse a Markdown input string into a sequence of Markdown abstract syntax
//! tree [`Block`]s.
//!
//! This module compensates for the fact that the `pulldown-cmark` crate is
//! focused on efficient incremental output (pull parsing), and consequently
//! doesn't provide it's own AST types.

mod unflatten;


use std::mem;

use pulldown_cmark::{self as md, Event, LinkType, Tag};

use self::unflatten::UnflattenedEvent;

pub use pulldown_cmark::HeadingLevel;

//======================================
// AST Representation
//======================================

/// A piece of structural Markdown content.
///
/// *CommonMark Spec:* [blocks](https://spec.commonmark.org/0.30/#blocks),
/// [container blocks](https://spec.commonmark.org/0.30/#container-blocks)
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Inlines),
    List(Vec<ListItem>),
    Heading(HeadingLevel, Inlines),
    /// An indented or fenced code block.
    ///
    /// *CommonMark Spec:* [indented code blocks](https://spec.commonmark.org/0.30/#indented-code-blocks),
    /// [fenced code blocks](https://spec.commonmark.org/0.30/#fenced-code-blocks)
    CodeBlock {
        /// If this `CodeBlock` is a fenced code block, this is its info string.
        ///
        /// *CommonMark Spec:* [info string](https://spec.commonmark.org/0.30/#info-string)
        info_string: Option<String>,
        code: String,
    },
    /// *CommonMark Spec:* [block quotes](https://spec.commonmark.org/0.30/#block-quotes)
    BlockQuote(Vec<Block>),
    Table {
        headers: Vec<Inlines>,
        rows: Vec<Vec<Inlines>>,
    },
    /// *CommonMark Spec:* [thematic breaks](https://spec.commonmark.org/0.30/#thematic-breaks)
    Rule,
}

/// A sequence of [`Inline`]s.
#[derive(Debug, Clone, PartialEq)]
pub struct Inlines(pub Vec<Inline>);

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem(pub Vec<Block>);

/// An inline piece of Markdown content.
///
/// *CommonMark Spec:* [inlines](https://spec.commonmark.org/0.30/#inlines)
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Emphasis(Inlines),
    Strong(Inlines),
    Strikethrough(Inlines),
    Code(String),
    Link { label: Inlines, destination: String },
    SoftBreak,
    HardBreak,
}

//======================================
// AST Builder
//======================================

pub fn markdown_to_ast(input: &str) -> Vec<Block> {
    /* For Markdown parsing debugging.
    {
        let mut options = md::Options::empty();
        options.insert(md::Options::ENABLE_STRIKETHROUGH);
        let parser = md::Parser::new_ext(input, options);

        let events: Vec<_> = parser.into_iter().collect();

        println!("==== All events =====\n");
        for event in &events {
            println!("{event:?}");
        }
        println!("\n=====================\n");

        println!("==== Unflattened events =====\n");
        for event in unflatten::parse_markdown_to_unflattened_events(input) {
            println!("{event:#?}")
        }
        println!("=============================\n");
    }
    */

    let events = unflatten::parse_markdown_to_unflattened_events(input);

    events_to_blocks(events)
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
            _ => todo!("handle tag: {tag:?}"),
        },
    }
}

fn events_to_blocks(events: Vec<UnflattenedEvent>) -> Vec<Block> {
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
                Event::Text(text) => text_spans.push(Inline::Text(text.to_string())),
                Event::Code(code) => text_spans.push(Inline::Code(code.to_string())),
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
                        text_spans.push(Inline::Strikethrough(unwrap_text(events)));
                    },

                    Tag::Link {
                        link_type,
                        dest_url: destination,
                        title: label,
                        // TODO: Use this `id`?
                        id: _,
                    } => {
                        let text = unwrap_text(events);
                        text_spans.push(Inline::from_link(
                            link_type,
                            text,
                            destination.to_string(),
                            label.to_string(),
                        ))
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
                        complete.push(Block::Heading(level, unwrap_text(events)));
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
                                let item_blocks = events_to_blocks(item_events);
                                items.push(ListItem(item_blocks));
                            } else {
                                todo!("handle list element: {event:?}");
                            }
                        }

                        complete.push(Block::List(items));
                    },
                    Tag::Item => {
                        complete.extend(events_to_blocks(events));
                    },
                    Tag::CodeBlock(kind) => {
                        let fence_label = match kind {
                            md::CodeBlockKind::Indented => None,
                            md::CodeBlockKind::Fenced(label) => Some(label.to_string()),
                        };

                        let text_spans = unwrap_text(events);
                        let code_text = text_to_string(&text_spans);

                        complete.push(Block::CodeBlock {
                            info_string: fence_label,
                            code: code_text,
                        })
                    },
                    Tag::BlockQuote(_kind) => {
                        let blocks = events_to_blocks(events);
                        complete.push(Block::BlockQuote(blocks))
                    },
                    // TODO: Support table column alignments.
                    Tag::Table(_alignments) => {
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

                        complete.push(Block::Table { headers, rows })
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

fn unwrap_text(events: Vec<UnflattenedEvent>) -> Inlines {
    let mut text_spans: Vec<Inline> = vec![];

    for event in events {
        match event {
            UnflattenedEvent::Event(event) => match event {
                Event::Start(_) | Event::End(_) => unreachable!(),
                Event::Text(text) => text_spans.push(Inline::Text(text.to_string())),
                Event::Code(code) => text_spans.push(Inline::Code(code.to_string())),
                Event::SoftBreak => text_spans.push(Inline::SoftBreak),
                Event::HardBreak => text_spans.push(Inline::HardBreak),
                Event::Html(_) => todo!("error: skipping inline HTML"),
                Event::InlineHtml(_) => todo!(),
                Event::TaskListMarker(_) | Event::Rule | Event::FootnoteReference(_) => {
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
                    dest_url: destination,
                    title: label,
                    // TODO: Use this `id`?
                    id: _,
                } => {
                    let text = unwrap_text(events);
                    text_spans.push(Inline::from_link(
                        link_type,
                        text,
                        destination.to_string(),
                        label.to_string(),
                    ))
                },
                _ => todo!("handle {tag:?}"),
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

//======================================
// Impls
//======================================

impl Inline {
    pub fn emphasis(inline: Inline) -> Self {
        Inline::Emphasis(Inlines(vec![inline]))
    }

    pub fn strong(inline: Inline) -> Self {
        Inline::Strong(Inlines(vec![inline]))
    }

    pub fn strikethrough(inline: Inline) -> Self {
        Inline::Strikethrough(Inlines(vec![inline]))
    }

    fn from_link(
        link_type: LinkType,
        text: Inlines,
        destination: String,
        label: String,
    ) -> Inline {
        if !label.is_empty() {
            eprintln!("warning: link label is ignored: {label:?}");
        }

        match link_type {
            LinkType::Inline => (),
            LinkType::Reference => (),
            LinkType::Shortcut => (),
            LinkType::Collapsed => (),
            LinkType::Autolink => (),
            LinkType::Email => (),
            // Unknown
            LinkType::ReferenceUnknown
            | LinkType::CollapsedUnknown
            | LinkType::ShortcutUnknown => {
                eprintln!(
                    "warning: unable to resolve location of link with text '{}'",
                    text_to_string(&text)
                )
            },
        }

        Inline::Link {
            label: text,
            destination,
        }
    }
}

impl Block {
    fn paragraph(text: Vec<Inline>) -> Block {
        Block::Paragraph(Inlines(text))
    }
}

impl IntoIterator for Inlines {
    type Item = Inline;
    type IntoIter = std::vec::IntoIter<Inline>;

    fn into_iter(self) -> Self::IntoIter {
        let Inlines(vec) = self;
        vec.into_iter()
    }
}

//======================================
// Tests
//======================================

#[test]
fn tests() {
    use pretty_assertions::assert_eq;

    assert_eq!(
        markdown_to_ast("hello"),
        vec![Block::paragraph(vec![Inline::Text("hello".into())])]
    );

    //--------------
    // Styled text
    //--------------

    assert_eq!(
        markdown_to_ast("*hello*"),
        vec![Block::paragraph(vec![Inline::emphasis(Inline::Text(
            "hello".into()
        ))])]
    );

    assert_eq!(
        markdown_to_ast("**hello**"),
        vec![Block::paragraph(vec![Inline::strong(Inline::Text(
            "hello".into()
        ))])]
    );

    assert_eq!(
        markdown_to_ast("~~hello~~"),
        vec![Block::paragraph(vec![Inline::strikethrough(Inline::Text(
            "hello".into()
        ))])]
    );

    assert_eq!(
        markdown_to_ast("**`strong code`**"),
        vec![Block::paragraph(vec![Inline::strong(Inline::Code(
            "strong code".into()
        ))])]
    );

    assert_eq!(
        markdown_to_ast("~~`foo`~~"),
        vec![Block::paragraph(vec![Inline::strikethrough(Inline::Code(
            "foo".into()
        ))])]
    );

    assert_eq!(
        markdown_to_ast("**[example](example.com)**"),
        vec![Block::paragraph(vec![Inline::strong(Inline::Link {
            label: Inlines(vec![Inline::Text("example".into())]),
            destination: "example.com".into()
        })])]
    );

    // Test composition of emphasis, strong, strikethrough and code
    assert_eq!(
        markdown_to_ast("_~~**`foo`**~~_"),
        vec![Block::paragraph(vec![Inline::emphasis(
            Inline::strikethrough(Inline::strong(Inline::Code("foo".into())))
        )])]
    );

    //--------------
    // Lists
    //--------------

    assert_eq!(
        markdown_to_ast("* hello"),
        vec![Block::List(vec![ListItem(vec![Block::paragraph(vec![
            Inline::Text("hello".into())
        ])])])]
    );

    // List items with styled text

    assert_eq!(
        markdown_to_ast("* *hello*"),
        vec![Block::List(vec![ListItem(vec![Block::paragraph(vec![
            Inline::emphasis(Inline::Text("hello".into()))
        ])])])]
    );

    assert_eq!(
        markdown_to_ast("* **hello**"),
        vec![Block::List(vec![ListItem(vec![Block::paragraph(vec![
            Inline::strong(Inline::Text("hello".into()))
        ])])])]
    );

    assert_eq!(
        markdown_to_ast("* ~~hello~~"),
        vec![Block::List(vec![ListItem(vec![Block::paragraph(vec![
            Inline::strikethrough(Inline::Text("hello".into()),)
        ])])])]
    );
}

#[test]
fn test_structure() {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    assert_eq!(
        markdown_to_ast(indoc!(
            "
            * hello

              world
            "
        )),
        vec![Block::List(vec![ListItem(vec![
            Block::paragraph(vec![Inline::Text("hello".into())]),
            Block::paragraph(vec![Inline::Text("world".into())])
        ])])]
    );

    #[rustfmt::skip]
    assert_eq!(
        markdown_to_ast(indoc!(
            "
            # Example

            * A
              - A.A

                hello world

                * *A.A.A*
            "
        )),
        vec![
            Block::Heading(
                HeadingLevel::H1,
                Inlines(vec![Inline::Text("Example".into())])
            ),
            Block::List(vec![
                ListItem(vec![
                    Block::paragraph(vec![Inline::Text("A".into())]),
                    Block::List(vec![
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.A".into())]),
                            Block::paragraph(vec![Inline::Text("hello world".into())]),
                            Block::List(vec![
                                ListItem(vec![
                                    Block::paragraph(vec![
                                        Inline::emphasis(
                                            Inline::Text(
                                            "A.A.A".into()),
                                        )
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    );

    #[rustfmt::skip]
    assert_eq!(
        markdown_to_ast(indoc!(
            "
            * A
              - A.A
                * A.A.A
              - A.B
              - A.C
            "
        )),
        vec![
            Block::List(vec![
                ListItem(vec![
                    Block::paragraph(vec![Inline::Text("A".into())]),
                    Block::List(vec![
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.A".into())]),
                            Block::List(vec![ListItem(vec![
                                Block::paragraph(vec![Inline::Text("A.A.A".into())]),
                            ])])
                        ]),
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.B".into())]),
                        ]),
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.C".into())]),
                        ])
                    ])
                ])
            ])
        ]
    );

    #[rustfmt::skip]
    assert_eq!(
        markdown_to_ast(indoc!(
            "
            # Example

            * A
              - A.A
              - A.B
              * A.C
            "
        )),
        vec![
            Block::Heading(
                HeadingLevel::H1,
                Inlines(vec![Inline::Text("Example".into())])
            ),
            Block::List(vec![
                ListItem(vec![
                    Block::paragraph(vec![Inline::Text("A".into())]),
                    Block::List(vec![
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.A".into())]),
                        ]),
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.B".into())]),
                        ]),
                    ]),
                    Block::List(vec![
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.C".into())])
                        ])
                    ]),
                ]),
            ])
        ]
    );

    #[rustfmt::skip]
    assert_eq!(
        markdown_to_ast(indoc!(
            "
            * A
              - A.A
              - A.B

                separate paragraph

              - A.C
            "
        )),
        vec![
            Block::List(vec![
                ListItem(vec![
                    Block::paragraph(vec![Inline::Text("A".into())]),
                    Block::List(vec![
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.A".into())]),
                        ]),
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.B".into())]),
                            Block::paragraph(vec![Inline::Text("separate paragraph".into())]),
                        ]),
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.C".into())]),
                        ])
                    ])
                ])
            ])
        ]
    );

    #[rustfmt::skip]
    assert_eq!(
        markdown_to_ast(indoc!(
            "
            # Example

            * A
              - A.A
                * A.A.A
                  **soft break**

              - A.B

                separate paragraph

              - A.C
            "
        )),
        vec![
            Block::Heading(
                HeadingLevel::H1,
                Inlines(vec![Inline::Text("Example".into())])
            ),
            Block::List(vec![
                ListItem(vec![
                    Block::paragraph(vec![Inline::Text("A".into())]),
                    Block::List(vec![
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.A".into())]),
                            Block::List(vec![
                                ListItem(vec![
                                    Block::paragraph(vec![
                                        Inline::Text("A.A.A".into()),
                                        Inline::SoftBreak,
                                        Inline::strong(
                                            Inline::Text("soft break".into()),
                                        )
                                    ]),
                                ])
                            ]),
                        ]),
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.B".into())]),
                            Block::paragraph(vec![Inline::Text("separate paragraph".into())]),
                        ]),
                        ListItem(vec![
                            Block::paragraph(vec![Inline::Text("A.C".into())]),
                        ]),
                    ])
                ])
            ])
        ]
    );
}
