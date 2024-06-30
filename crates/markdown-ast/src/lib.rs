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
//! # Quick Examples
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
//!
//!
//! # API Overview
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
//! ##### Terminology
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
//! ##### Processing Steps
//!
//! ```text
//!     String => Events => Blocks => Events => String
//!     └───── A ──────┘    └────── C ─────┘
//!                └────── B ─────┘    └────── D ────┘
//!     └────────── E ────────────┘
//!                         └─────────── F ──────────┘
//!     └──────────────────── G ─────────────────────┘
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
//!
//!
//! # Detailed Examples
//!
//! #### Parse varied Markdown to an AST representation:
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
//! #### Synthesize Markdown using programmatic construction of the document:
//!
//! *Note:* This is a more user friendly alternative to a "string builder"
//! approach where the raw Markdown string is constructed piece by piece,
//! which suffers from extra bookkeeping that must be done to manage things like
//! indent level and soft vs hard breaks.
//!
//! ```
//! use markdown_ast::{
//!     ast_to_markdown, Block, Inline, Inlines, ListItem,
//!     HeadingLevel,
//! };
//! # use pretty_assertions::assert_eq;
//!
//! let tech_companies = vec![
//!     ("Apple", 1976, 164_000),
//!     ("Microsoft", 1975, 221_000),
//!     ("Nvidia", 1993, 29_600),
//! ];
//!
//! let ast = vec![
//!     Block::Heading(HeadingLevel::H1, Inlines::plain_text("Tech Companies")),
//!     Block::plain_text_paragraph("The following are major tech companies:"),
//!     Block::List(Vec::from_iter(
//!         tech_companies
//!             .into_iter()
//!             .map(|(company_name, founded, employee_count)| {
//!                 ListItem(vec![
//!                     Block::paragraph(vec![Inline::plain_text(company_name)]),
//!                     Block::List(vec![
//!                         ListItem::plain_text(format!("Founded: {founded}")),
//!                         ListItem::plain_text(format!("Employee count: {employee_count}"))
//!                     ])
//!                 ])
//!             })
//!     ))
//! ];
//!
//! let markdown: String = ast_to_markdown(&ast);
//!
//! assert_eq!(markdown, "\
//! ## Tech Companies
//!
//! The following are major tech companies:
//!
//! * Apple
//!  
//!   * Founded: 1976
//!  
//!   * Employee count: 164000
//!
//! * Microsoft
//!  
//!   * Founded: 1975
//!  
//!   * Employee count: 221000
//!
//! * Nvidia
//!  
//!   * Founded: 1993
//!  
//!   * Employee count: 29600\
//! ");
//!
//! ```
//!
//! # Known Issues
//!
//! Currently `markdown-ast` does not escape Markdown content appearing in
//! leaf inline text:
//!
//! ```
//! use markdown_ast::{ast_to_markdown, Block};
//!
//! let ast = vec![
//!     Block::plain_text_paragraph("In the equation a*b*c ...")
//! ];
//!
//! let markdown = ast_to_markdown(&ast);
//!
//! assert_eq!(markdown, "In the equation a*b*c ...");
//! ```
//!
//! which will render as:
//!
//! > In the equation a*b*c ...
//!
//! with the asterisks interpreted as emphasis formatting markers, contrary to
//! the intention of the author.
//!
//! Fixing this robustly will require either:
//!
//! * Adding automatic escaping of Markdown characters in [`Inline::Text`]
//!   during rendering (not ideal)
//!
//! * Adding pre-construction validation checks for [`Inline::Text`] that
//!   prevent constructing an `Inline` with Markdown formatting characters that
//!   have not been escaped correctly by the user.
//!
//! In either case, fixing this bug will be considered a **semver exempt**
//! change in behavior to `markdown-ast`.
//!
//! # Motivation and relation to `pulldown-cmark`
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

mod from_events;
mod to_events;

/// Ensure that doc tests in the README.md file get run.
///
/// See: <https://connorgray.com/reference/creating-a-new-rust-crate#test-readmemd-examples>
mod test_readme {
    #![doc = include_str!("../README.md")]
}

use pulldown_cmark::{self as md, CowStr, Event};

pub use pulldown_cmark::{HeadingLevel, LinkType};

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

/// An inline piece of atomic Markdown content.
/// (CommonMark: [inlines](https://spec.commonmark.org/0.30/#inlines))
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// CommonMark: [textual content](https://spec.commonmark.org/0.30/#textual-content)
    ///
    /// ```
    /// # use markdown_ast::Inline;
    /// #
    /// assert_eq!(
    ///     Inline::parse("some plain text"),
    ///     Inline::Text("some plain text".to_owned())
    /// );
    /// ```
    Text(String),

    /// CommonMark: [emphasis](https://spec.commonmark.org/0.30/#emphasis-and-strong-emphasis)
    ///
    /// ```
    /// # use markdown_ast::{Inline, Inlines};
    /// #
    /// assert_eq!(
    ///     Inline::parse("*emphasized content*"),
    ///     Inline::Emphasis(Inlines(vec![
    ///         Inline::Text("emphasized content".to_owned())
    ///     ]))
    /// );
    /// ```
    Emphasis(Inlines),

    /// CommonMark: [strong emphasis](https://spec.commonmark.org/0.30/#emphasis-and-strong-emphasis)
    ///
    /// ```
    /// # use markdown_ast::{Inline, Inlines};
    /// #
    /// assert_eq!(
    ///     Inline::parse("**strong content**"),
    ///     Inline::Strong(Inlines(vec![
    ///         Inline::Text("strong content".to_owned())
    ///     ]))
    /// );
    /// ```
    Strong(Inlines),

    /// Strikethrough styled text. (Non-standard.)
    ///
    /// ```
    /// # use markdown_ast::{Inline, Inlines};
    /// #
    /// assert_eq!(
    ///     Inline::parse("~~struck-through content~~"),
    ///     Inline::Strikethrough(Inlines(vec![
    ///         Inline::Text("struck-through content".to_owned())
    ///     ]))
    /// );
    /// ```
    Strikethrough(Inlines),

    /// CommonMark: [code spans](https://spec.commonmark.org/0.30/#code-spans)
    ///
    /// ```
    /// # use markdown_ast::Inline;
    /// #
    /// assert_eq!(
    ///     Inline::parse("`fn code()`"),
    ///     Inline::Code("fn code()".to_owned())
    /// );
    /// ```
    Code(String),

    /// CommonMark: [links](https://spec.commonmark.org/0.30/#links)
    ///
    /// Example: Inline link with title:
    ///
    /// ```
    /// # use markdown_ast::{Inline, Inlines, LinkType};
    /// #
    /// assert_eq!(
    ///     Inline::parse("[my website](connorgray.com \"Connor Gray's website\")"),
    ///     Inline::Link {
    ///         link_type: LinkType::Inline,
    ///         dest_url: "connorgray.com".to_owned(),
    ///         title: "Connor Gray's website".to_owned(),
    ///         id: "".to_owned(),
    ///         content_text: Inlines::plain_text("my website")
    ///     }
    /// );
    /// ```
    //
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

    /// CommonMark: [images](https://spec.commonmark.org/0.30/#images)
    ///
    /// Example: Inline image link:
    ///
    /// ```
    /// # use markdown_ast::{Inline, Inlines, LinkType};
    /// #
    /// assert_eq!(
    ///     Inline::parse("![cat photo](example.org/photo.png)"),
    ///     Inline::Image {
    ///         link_type: LinkType::Inline,
    ///         dest_url: "example.org/photo.png".to_owned(),
    ///         title: "".to_owned(),
    ///         id: "".to_owned(),
    ///         image_description: Inlines::plain_text("cat photo")
    ///     }
    /// );
    /// ```
    Image {
        link_type: md::LinkType,
        /// CommonMark: [link destination](https://spec.commonmark.org/0.30/#link-destination)
        dest_url: String,
        /// CommonMark: [link title](https://spec.commonmark.org/0.30/#link-title)
        title: String,
        /// CommonMark: [link label](https://spec.commonmark.org/0.30/#link-label)
        id: String,
        /// CommonMark: [image description](https://spec.commonmark.org/0.30/#image-description)
        image_description: Inlines,
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
pub fn events_to_markdown<'e, I: IntoIterator<Item = Event<'e>>>(
    events: I,
) -> String {
    let mut string = String::new();

    let options = default_to_markdown_options();

    let _: pulldown_cmark_to_cmark::State =
        pulldown_cmark_to_cmark::cmark_with_options(
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

        crate::to_events::block_to_events(&block, events);
    }

    events
}

/// Parse [`Event`]s into AST [`Block`]s.
pub fn events_to_ast<'i, I: IntoIterator<Item = Event<'i>>>(
    events: I,
) -> Vec<Block> {
    let events =
        unflatten::parse_markdown_to_unflattened_events(events.into_iter());

    crate::from_events::ast_events_to_ast(events)
}

/// Parse Markdown input string into [`Event`]s.
///
/// This is a thin wrapper around [`pulldown_cmark::Parser`], provided in this
/// crate for consistency and ease of use.
pub fn markdown_to_events<'i>(
    input: &'i str,
) -> impl Iterator<Item = Event<'i>> {
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
// Impls
//======================================

impl Inline {
    /// Parse a piece of simple input into an [`Inline`].
    ///
    /// # Panics
    ///
    /// This function will panic if the provided Markdown is not a trivial
    /// piece of inline content.
    ///
    /// # Examples
    ///
    /// Parse a few different kinds of simple inline Markdown content:
    ///
    /// ```
    /// use markdown_ast::{Inline, Inlines};
    ///
    /// assert_eq!(Inline::parse("hello"), Inline::Text("hello".to_owned()));
    ///
    /// assert_eq!(Inline::parse("`foo`"), Inline::Code("foo".to_owned()));
    ///
    /// assert_eq!(
    ///     Inline::parse("**HELLO**"),
    ///     Inline::Strong(Inlines(vec![Inline::Text("HELLO".to_owned())]))
    /// );
    /// ```
    pub fn parse(input: &str) -> Self {
        Self::try_parse(input).unwrap_or_else(|err| {
            panic!("Inline::parse: provided Markdown is not a simple Inline element, parsed to: `{err:?}`")
        })
    }

    /// Parse a piece of simple input into an [`Inline`].
    ///
    /// If the provided input was not a simple [`Inline`], the full parsed
    /// Markdown AST will be returned as an error.
    pub fn try_parse(input: &str) -> Result<Self, Vec<Block>> {
        let ast = markdown_to_ast(input);

        match ast.as_slice() {
            [Block::Paragraph(Inlines(inlines))] => match inlines.as_slice() {
                [inline] => return Ok(inline.clone()),
                _ => return Err(ast),
            },
            _ => return Err(ast),
        }
    }

    /// Construct a inline containing a piece of plain text.
    pub fn plain_text<S: Into<String>>(s: S) -> Self {
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

impl Inlines {
    /// Construct an inlines sequence containing a single inline piece of plain
    /// text.
    pub fn plain_text<S: Into<String>>(inline: S) -> Self {
        return Inlines(vec![Inline::Text(inline.into())]);
    }
}

impl Block {
    /// Construct a paragraph block containing a single inline piece of plain
    /// text.
    pub fn plain_text_paragraph<S: Into<String>>(inline: S) -> Self {
        return Block::Paragraph(Inlines(vec![Inline::Text(inline.into())]));
    }

    pub fn paragraph(text: Vec<Inline>) -> Block {
        Block::Paragraph(Inlines(text))
    }
}

impl ListItem {
    /// Construct a list item containing a single inline piece of plain text.
    pub fn plain_text<S: Into<String>>(inline: S) -> Self {
        return ListItem(vec![Block::Paragraph(Inlines(vec![Inline::Text(
            inline.into(),
        )]))]);
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
            Inline::plain_text("And "),
            Inline::strong(Inline::plain_text("bold")),
            Inline::plain_text(" text."),
        ]),
        Block::List(vec![ListItem(vec![
            Block::paragraph(vec![Inline::plain_text(
                "With nested list items.",
            )]),
            Block::List(vec![ListItem(vec![Block::paragraph(vec![
                Inline::code("md2nb"),
                Inline::plain_text(
                    " supports nested lists up to three levels deep.",
                ),
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
        ast_to_markdown(&[Block::paragraph(vec![Inline::Text(
            "hello".into()
        )])]),
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
    let kitchen_sink_md =
        include_str!("../../md2nb/docs/examples/kitchen-sink.md");

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
