//! Parse a Markdown input string into a sequence of Markdown abstract syntax
//! tree [`Block`]s.
//!
//! This crate is intentionally designed to interoperate well with the
//! [`pulldown-cmark`](https://crates.io/crate/pulldown-cmark) crate and the
//! ecosystem around it. See [Motivation and relation to pulldown-cmark](#motivation-and-relation-to-pulldown-cmark)
//! for more information.
//!
//! The AST types are designed to align with the structure defined
//! by the [CommonMark Specification](https://spec.commonmark.org/).
//!
//! ## Quick Examples
//!
//! Parse simple Markdown into an AST:
//!
//! ```
//! use markdown_ast::{markdown_to_ast, Block, Inline, Inlines};
//! # use pretty_assertions::assert_eq;
//!
//! let ast = markdown_to_ast("
//! Hello! This is a paragraph **with bold text**.
//! ");
//!
//! assert_eq!(ast, vec![
//!     Block::Paragraph(Inlines(vec![
//!         Inline::Text("Hello! This is a paragraph ".to_owned()),
//!         Inline::Strong(Inlines(vec![
//!             Inline::Text("with bold text".to_owned()),
//!         ])),
//!         Inline::Text(".".to_owned())
//!     ]))
//! ]);
//! ```
//!
//! ## API Overview
//!
//! | Function                           | Input      | Output       |
//! |------------------------------------|------------|--------------|
//! | [`markdown_to_ast()`]              | `&str`     | `Vec<Block>` |
//! | [`ast_to_markdown()`]              | `&[Block]` | `String`     |
//! | [`ast_to_events()`]                | `&[Block]` | `Vec<Event>` |
//! | [`events_to_ast()`]                | `&[Event]` | `Vec<Block>` |
//! | [`events_to_markdown()`]           | `&[Event]` | `String`     |
//! | [`markdown_to_events()`]           | `&str`     | `Vec<Event>` |
//! | [`canonicalize()`]                 | `&str`     | `String`     |
//!
//! ### Terminology
//!
//! This crate is able to process and manipulate Markdown in three different
//! representations:
//!
//! | Term     | Type                 | Description                         |
//! |----------|----------------------|-------------------------------------|
//! | Markdown | `String`             | Raw Markdown source / output string |
//! | Events   | `&[Event]`           | Markdown parsed by [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark) into a flat sequence of parser [`Event`]s |
//! | AST      | `Block` / `&[Block]` | Markdown parsed by `markdown-ast` into a hierarchical structure of [`Block`]s |
//!
//! ### Processing Steps
//!
//! ```text
//!     String => Events => Blocks => Events => String
//!     |_____ A ______|    |______ C _____|
//!               |______ B _____|    |______ D _____|
//!     |__________ E ___________|
//!                         |___________ F __________|
//!     |____________________ G _____________________|
//! ```
//!
//! - **A** — [`markdown_to_events()`]
//! - **B** — [`events_to_ast()`]
//! - **C** — [`ast_to_events()`]
//! - **D** — [`events_to_markdown()`]
//! - **E** — [`markdown_to_ast()`]
//! - **F** — [`ast_to_markdown()`]
//! - **G** — [`canonicalize()`]
//!
//! Note: **A** wraps [`pulldown_cmark::Parser`], and **D** wraps
//! [`pulldown_cmark_to_cmark::cmark()`].
//!
//! ## Detailed Examples
//!
//! Parse varied Markdown to an AST representation:
//!
//! ```
//! use markdown_ast::{
//!     markdown_to_ast, Block, HeadingLevel, Inline, Inlines, ListItem
//! };
//! # use pretty_assertions::assert_eq;
//!
//! let ast = markdown_to_ast("
//! ## An Example Document
//!
//! This is a paragraph that
//! is split across *multiple* lines.
//!
//! * This is a list item
//! ");
//!
//! assert_eq!(ast, vec![
//!     Block::Heading(
//!         HeadingLevel::H1,
//!         Inlines(vec![
//!              Inline::Text("An Example Document".to_owned())
//!         ])
//!     ),
//!     Block::Paragraph(Inlines(vec![
//!         Inline::Text("This is a paragraph that".to_owned()),
//!         Inline::SoftBreak,
//!         Inline::Text("is split across ".to_owned()),
//!         Inline::Emphasis(Inlines(vec![
//!             Inline::Text("multiple".to_owned()),
//!         ])),
//!         Inline::Text(" lines.".to_owned()),
//!     ])),
//!     Block::List(vec![
//!         ListItem(vec![
//!             Block::Paragraph(Inlines(vec![
//!                 Inline::Text("This is a list item".to_owned())
//!             ]))
//!         ])
//!     ])
//! ]);
//! ```
//!
//! ### Motivation and relation to `pulldown-cmark`
//!
//! [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark) is a popular
//! Markdown parser crate. It provides a streaming event (pull parsing) based
//! representation of a Markdown document. That representation is useful for
//! efficient transformation of a Markdown document into another format, often
//! HTML.
//!
//! However, a streaming parser representation is less amenable to programmatic
//! construction or human-understandable transformations of Markdown documents.
//!
//! `markdown-ast` provides a abstract syntax tree (AST) representation of
//! Markdown that is easy to construct and work with.
//!
//! Additionally, `pulldown-cmark` is widely used in the Rust crate ecosystem,
//! for example for [`mdbook`](https://crates.io/crates/mdbook) extensions.
//! Interoperability with `pulldown-cmark` is an intentional design choice for
//! the usability of `markdown-ast`; one could imagine `markdown-ast` instead
//! abstracting over the underlying parser implementation, but my view is that
//! would limit the utility of `markdown-ast`.
//!

mod unflatten;

/// Ensure that doc tests in the README.md file get run.
/// See: https://connorgray.com/reference/creating-a-new-rust-crate#test-readmemd-examples
#[doc(hidden)]
mod test_readme {
    #![doc = include_str!("../../../README.md")]
}

use std::mem;

use pulldown_cmark::{self as md, CowStr, Event, Tag};

use self::unflatten::UnflattenedEvent;

pub use pulldown_cmark::HeadingLevel;

//======================================
// AST Representation
//======================================

/// A piece of structural Markdown content.
/// (CommonMark: [blocks](https://spec.commonmark.org/0.30/#blocks),
/// [container blocks](https://spec.commonmark.org/0.30/#container-blocks))
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// CommonMark: [paragraphs](https://spec.commonmark.org/0.30/#paragraphs)
    Paragraph(Inlines),
    /// CommonMark: [lists](https://spec.commonmark.org/0.30/#lists)
    List(Vec<ListItem>),
    /// CommonMark: [ATX heading](https://spec.commonmark.org/0.30/#atx-heading)
    Heading(HeadingLevel, Inlines),
    /// An indented or fenced code block.
    ///
    /// CommonMark: [indented code blocks](https://spec.commonmark.org/0.30/#indented-code-blocks),
    /// [fenced code blocks](https://spec.commonmark.org/0.30/#fenced-code-blocks)
    CodeBlock {
        /// Indicates whether this is a fenced or indented code block.
        ///
        /// If this `CodeBlock` is a fenced code block, this contains its info
        /// string.
        ///
        /// CommonMark: [info string](https://spec.commonmark.org/0.30/#info-string)
        kind: CodeBlockKind,
        code: String,
    },
    /// CommonMark: [block quotes](https://spec.commonmark.org/0.30/#block-quotes)
    BlockQuote {
        // TODO: Document
        kind: Option<md::BlockQuoteKind>,
        blocks: Vec<Block>,
    },
    Table {
        alignments: Vec<md::Alignment>,
        headers: Vec<Inlines>,
        rows: Vec<Vec<Inlines>>,
    },
    /// CommonMark: [thematic breaks](https://spec.commonmark.org/0.30/#thematic-breaks)
    Rule,
}

/// A sequence of [`Inline`]s.
/// (CommonMark: [inlines](https://spec.commonmark.org/0.30/#inlines))
#[derive(Debug, Clone, PartialEq)]
pub struct Inlines(pub Vec<Inline>);

/// An item in a list. (CommonMark: [list items](https://spec.commonmark.org/0.30/#list-items))
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem(pub Vec<Block>);

/// An inline piece of Markdown content.
/// (CommonMark: [inlines](https://spec.commonmark.org/0.30/#inlines))
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    /// CommonMark: [emphasis](https://spec.commonmark.org/0.30/#emphasis-and-strong-emphasis)
    Emphasis(Inlines),
    /// CommonMark: [strong emphasis](https://spec.commonmark.org/0.30/#emphasis-and-strong-emphasis)
    Strong(Inlines),
    /// Strikethrough styled text. (Non-standard.)
    Strikethrough(Inlines),
    /// CommonMark: [code spans](https://spec.commonmark.org/0.30/#code-spans)
    Code(String),
    /// CommonMark: [links](https://spec.commonmark.org/0.30/#links)
    // TODO:
    //  Document every type of Inline::Link value and what its equivalent source
    //  is.
    Link {
        link_type: md::LinkType,
        /// CommonMark: [link destination](https://spec.commonmark.org/0.30/#link-destination)
        dest_url: String,
        /// CommonMark: [link title](https://spec.commonmark.org/0.30/#link-title)
        title: String,
        /// CommonMark: [link label](https://spec.commonmark.org/0.30/#link-label)
        id: String,
        /// CommonMark: [link text](https://spec.commonmark.org/0.30/#link-text)
        content_text: Inlines,
    },
    /// CommonMark: [soft line breaks](https://spec.commonmark.org/0.30/#soft-line-breaks)
    SoftBreak,
    /// CommonMark: [hard line breaks](https://spec.commonmark.org/0.30/#hard-line-breaks)
    HardBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockKind {
    Fenced(String),
    Indented,
}

//======================================
// Public API Functions
//======================================

/// Parse Markdown input string into AST [`Block`]s.
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

    let events = markdown_to_events(input);

    return events_to_ast(events);
}

/// Convert AST [`Block`]s into a Markdown string.
pub fn ast_to_markdown(blocks: &[Block]) -> String {
    let events = ast_to_events(blocks);

    return events_to_markdown(events);
}

/// Convert [`Event`]s into a Markdown string.
///
/// This is a thin wrapper around
/// [`pulldown_cmark_to_cmark::cmark_with_options`], provided in this crate for
/// consistency and ease of use.
pub fn events_to_markdown<'e, I: IntoIterator<Item = Event<'e>>>(events: I) -> String {
    let mut string = String::new();

    let options = default_to_markdown_options();

    let _: pulldown_cmark_to_cmark::State = pulldown_cmark_to_cmark::cmark_with_options(
        events.into_iter(),
        &mut string,
        options,
    )
    .expect("error converting Event sequent to Markdown string");

    string
}

/// Convert AST [`Block`]s into an [`Event`] sequence.
pub fn ast_to_events(blocks: &[Block]) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    for block in blocks {
        let events = &mut events;

        block_to_events(&block, events);
    }

    events
}

/// Parse [`Event`]s into AST [`Block`]s.
pub fn events_to_ast<'i, I: IntoIterator<Item = Event<'i>>>(events: I) -> Vec<Block> {
    let events = unflatten::parse_markdown_to_unflattened_events(events.into_iter());

    ast_events_to_ast(events)
}

/// Parse Markdown input string into [`Event`]s.
///
/// This is a thin wrapper around [`pulldown_cmark::Parser`], provided in this
/// crate for consistency and ease of use.
pub fn markdown_to_events<'i>(input: &'i str) -> impl Iterator<Item = Event<'i>> {
    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = md::Options::empty();
    options.insert(md::Options::ENABLE_STRIKETHROUGH);
    options.insert(md::Options::ENABLE_TABLES);
    md::Parser::new_ext(input, options)
}

/// Canonicalize (or format) a Markdown input by parsing and then converting
/// back to a string.
///
/// **⚠️ Warning ⚠️:** This function is **semver exempt**. The precise
/// canonicalization behavior may change in MINOR or PATCH versions of
/// markdown-ast. (Stabilizing the behavior of this function will require
/// additional options to configure the behavior of
/// [pulldown-cmark-to-cmark](https://crates.io/crates/pulldown-cmark-to-cmark).)
///
/// # Examples
///
/// List items using `-` (minus) are canonicalized to the `*` (asterisk) list
/// marker type:
///
/// ```
/// use markdown_ast::canonicalize;
/// assert_eq!(
/// canonicalize("\
/// - Foo
/// - Bar
/// "),
/// "\
/// * Foo
///
/// * Bar"
/// )
/// ```
///
/// Hard breaks ending in backslash are canonicalized to the "two spaces at the
/// end of the line" form:
///
/// ```
/// use markdown_ast::canonicalize;
/// assert_eq!(
/// canonicalize(r#"
/// This ends in a hard break.\
/// This is a new line."#),
/// // Note: The two spaces at the end of the first line below may not be
/// //       visible, but they're there.
/// "\
/// This ends in a hard break.  
/// This is a new line."
/// )
/// ```
pub fn canonicalize(input: &str) -> String {
    let ast = markdown_to_ast(input);

    return ast_to_markdown(&ast);
}

fn default_to_markdown_options() -> pulldown_cmark_to_cmark::Options<'static> {
    pulldown_cmark_to_cmark::Options {
        // newlines_after_paragraph: 2,
        // newlines_after_headline: 0,
        // newlines_after_codeblock: 0,
        // newlines_after_list: 1,
        // newlines_after_rest: 0,
        code_block_token_count: 3,
        ..pulldown_cmark_to_cmark::Options::default()
    }
}

//======================================
// AST Builder
//======================================

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

fn ast_events_to_ast(events: Vec<UnflattenedEvent>) -> Vec<Block> {
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
                                let item_blocks = ast_events_to_ast(item_events);
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
    pub fn text<S: Into<String>>(s: S) -> Self {
        Inline::Text(s.into())
    }

    pub fn emphasis(inline: Inline) -> Self {
        Inline::Emphasis(Inlines(vec![inline]))
    }

    pub fn strong(inline: Inline) -> Self {
        Inline::Strong(Inlines(vec![inline]))
    }

    pub fn strikethrough(inline: Inline) -> Self {
        Inline::Strikethrough(Inlines(vec![inline]))
    }

    pub fn code<S: Into<String>>(s: S) -> Self {
        Inline::Code(s.into())
    }
}

impl Block {
    fn paragraph(text: Vec<Inline>) -> Block {
        Block::Paragraph(Inlines(text))
    }
}

impl CodeBlockKind {
    pub fn info_string(&self) -> Option<&str> {
        match self {
            CodeBlockKind::Fenced(info_string) => Some(info_string.as_str()),
            CodeBlockKind::Indented => None,
        }
    }

    pub(crate) fn from_pulldown_cmark(kind: md::CodeBlockKind) -> Self {
        match kind {
            md::CodeBlockKind::Indented => CodeBlockKind::Indented,
            md::CodeBlockKind::Fenced(info_string) => {
                CodeBlockKind::Fenced(info_string.to_string())
            },
        }
    }

    pub(crate) fn to_pulldown_cmark<'s>(&'s self) -> md::CodeBlockKind<'s> {
        match self {
            CodeBlockKind::Fenced(info) => {
                md::CodeBlockKind::Fenced(CowStr::from(info.as_str()))
            },
            CodeBlockKind::Indented => md::CodeBlockKind::Indented,
        }
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
// AST blocks to Events
//======================================

fn block_to_events<'ast>(block: &'ast Block, events: &mut Vec<Event<'ast>>) {
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
                            [Block::Paragraph(inlines)] if list_items.len() == 1 => {
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

fn inlines_to_events<'ast>(inlines: &'ast Inlines, events: &mut Vec<Event<'ast>>) {
    let Inlines(inlines) = inlines;

    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                events.push(Event::Text(CowStr::from(text.as_str())));
            },
            Inline::Emphasis(inlines) => wrap(Tag::Emphasis, events, |events| {
                inlines_to_events(inlines, events)
            }),
            Inline::Strong(inlines) => wrap(Tag::Strong, events, |events| {
                inlines_to_events(inlines, events)
            }),
            Inline::Strikethrough(inlines) => {
                wrap(Tag::Strikethrough, events, |events| {
                    inlines_to_events(inlines, events)
                })
            },
            Inline::Code(code) => events.push(Event::Code(CowStr::from(code.as_str()))),
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
            Inline::SoftBreak => events.push(Event::SoftBreak),
            Inline::HardBreak => events.push(Event::HardBreak),
        }
    }
}

//======================================
// Tests: Markdown to AST parsing
//======================================

#[test]
fn test_markdown_to_ast() {
    use indoc::indoc;
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
            link_type: md::LinkType::Inline,
            dest_url: "example.com".into(),
            title: String::new(),
            id: String::new(),
            content_text: Inlines(vec![Inline::Text("example".into())]),
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

    //----------------------------------

    let input = "\
* And **bold** text.
  
  * With nested list items.
    
    * `md2nb` supports nested lists up to three levels deep.
";

    let ast = vec![Block::List(vec![ListItem(vec![
        Block::paragraph(vec![
            Inline::text("And "),
            Inline::strong(Inline::text("bold")),
            Inline::text(" text."),
        ]),
        Block::List(vec![ListItem(vec![
            Block::paragraph(vec![Inline::text("With nested list items.")]),
            Block::List(vec![ListItem(vec![Block::paragraph(vec![
                Inline::code("md2nb"),
                Inline::text(" supports nested lists up to three levels deep."),
            ])])]),
        ])]),
    ])])];

    assert_eq!(markdown_to_ast(input), ast);

    // Sanity check conversion to event stream.
    assert_eq!(
        markdown_to_events(input).collect::<Vec<_>>(),
        ast_to_events(&ast)
    );

    //----------------------------------
    // Test structures
    //----------------------------------

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

//======================================
// Tests: AST to Markdown string
//======================================

#[test]
fn test_ast_to_markdown() {
    use indoc::indoc;
    // use pretty_assertions::assert_eq;

    assert_eq!(
        ast_to_markdown(&[Block::paragraph(vec![Inline::Text("hello".into())])]),
        "hello"
    );

    assert_eq!(
        ast_to_markdown(&[Block::List(vec![ListItem(vec![
            Block::paragraph(vec![Inline::Text("hello".into())]),
            Block::paragraph(vec![Inline::Text("world".into())])
        ])])]),
        indoc!(
            "
            * hello
              
              world"
        ),
    )
}

/// Tests that some of the larger Markdown documents in this repository
/// all round-trip when processed:
#[test]
fn test_md_documents_roundtrip() {
    let kitchen_sink_md = include_str!("../../md2nb/docs/examples/kitchen-sink.md");

    // FIXME:
    //  Fix the bugs requiring these hacky removals from kitchen-sink.md
    //  that are needed to make the tests below pass.
    let kitchen_sink_md = kitchen_sink_md
        .replace("\n    \"This is an indented code block.\"\n", "")
        .replace("\nThis is a [shortcut] reference link.\n", "")
        .replace("\nThis is a [full reference][full reference] link.\n", "")
        .replace("\n[full reference]: https://example.org\n", "")
        .replace("[shortcut]: https://example.org\n", "");

    assert_roundtrip(&kitchen_sink_md);

    //==================================
    // README.md
    //==================================

    let readme = include_str!("../../../README.md");

    assert_roundtrip(readme);
}

#[cfg(test)]
fn assert_roundtrip(markdown: &str) {
    use pretty_assertions::assert_eq;

    // Recall:
    //
    //     String => Events => Blocks => Events => String
    //     |_____ A ______|    |______ C _____|
    //               |______ B _____|    |______ D _____|
    //     |__________ E ___________|
    //                         |___________ F __________|

    // Do A to get Events
    let original_events: Vec<Event> = markdown_to_events(markdown).collect();

    // Do B to get AST Blocks
    let ast: Vec<Block> = events_to_ast(original_events.clone());

    // println!("ast = {ast:#?}");

    // Do C to get Events again
    let processed_events: Vec<Event> = ast_to_events(&ast);

    // println!("original_events = {original_events:#?}");

    // Test that A => B => C is equivalent to just A.
    // I.e. that converting an Event stream to and from an AST is lossless.
    assert_eq!(processed_events, original_events);

    // Test that A => B => C => D produces Markdown equivalent to the original
    // Markdown string.
    assert_eq!(ast_to_markdown(&ast), markdown);
}
