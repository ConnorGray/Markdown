mod from_expr_utils;

use wolfram_library_link::{
    export,
    expr::{Expr, Number, Symbol},
};

use markdown_ast::{Block, HeadingLevel, Inline, Inlines, ListItem};

use self::from_expr_utils::try_headed;

// TODO(cleanup):
//  Rename to MarkdownInline[..] and MarkdownBlock[..]?
//  Currently the "kind" of MarkdownElement is either an "inline" or "block"
//  element but that is not explicit to the caller.
#[allow(non_upper_case_globals)]
const MarkdownElement: &str = "ConnorGray`Markdown`MarkdownElement";

#[export(wstp, hidden)]
fn load_wolfram_markdown_link(args: Vec<Expr>) -> Expr {
    assert!(args.len() == 0);
    return wolfram_library_link::exported_library_functions_association(None);
}

#[export(wstp)]
fn parse_markdown(args: Vec<Expr>) -> Expr {
    if args.len() != 1 {
        panic!("incorrect argument count");
    }

    let s: &str = args[0].try_as_str().expect("expected String argument");

    let ast: Vec<Block> = markdown_ast::markdown_to_ast(s);

    let ast: Vec<Expr> = ast.iter().map(block_to_expr).collect();

    Expr::list(ast)
}

#[export(wstp)]
fn markdown_ast_to_markdown(args: Vec<Expr>) -> Expr {
    let [blocks]: [Expr; 1] = match args.try_into() {
        Ok(args) => args,
        Err(args) => panic!("expected one argument, got {}", args.len()),
    };

    let blocks =
        parse_expr_blocks(&blocks).expect("error converting expr to Markdown AST blocks");

    Expr::string(markdown_ast::ast_to_markdown(&blocks))
}

//======================================
// Block to Expr Conversion
//======================================

fn block_to_expr(block: &Block) -> Expr {
    match block {
        Block::Paragraph(inlines) => Expr::normal(
            Symbol::new(MarkdownElement),
            vec![Expr::string("Paragraph"), inlines_to_expr(inlines)],
        ),
        // FIXME: Should say whether the list is ordered or not
        Block::List(items) => {
            let exprs = items.iter().map(list_item_to_expr).collect();

            Expr::normal(
                Symbol::new(MarkdownElement),
                vec![Expr::string("List"), Expr::list(exprs)],
            )
        },
        Block::Heading(level, inlines) => {
            let level = match level {
                HeadingLevel::H1 => 1,
                HeadingLevel::H2 => 2,
                HeadingLevel::H3 => 3,
                HeadingLevel::H4 => 4,
                HeadingLevel::H5 => 5,
                HeadingLevel::H6 => 6,
            };

            // MarkdownElement["Heading", level, {...}]
            Expr::normal(
                Symbol::new(MarkdownElement),
                vec![
                    Expr::string("Heading"),
                    Expr::from(level),
                    inlines_to_expr(inlines),
                ],
            )
        },
        // MarkdownElement["CodeBlock", "info", "code"]
        Block::CodeBlock { kind, code } => Expr::normal(
            Symbol::new(MarkdownElement),
            vec![
                Expr::string("CodeBlock"),
                match kind.info_string() {
                    Some(info) => Expr::string(info),
                    None => Expr::symbol(Symbol::new("System`None")),
                },
                Expr::string(code),
            ],
        ),
        // MarkdownElement["BlockQuote", {...}]
        // FIXME: Convert the `kind` as well.
        Block::BlockQuote { kind: _, blocks } => {
            let blocks = blocks.into_iter().map(block_to_expr).collect();

            Expr::normal(
                Symbol::new(MarkdownElement),
                vec![Expr::string("BlockQuote"), Expr::list(blocks)],
            )
        },
        Block::Table {
            alignments: _,
            headers: _,
            rows: _,
        } => todo!(),
        Block::Rule => Expr::normal(
            Symbol::new(MarkdownElement),
            vec![Expr::string("ThematicBreak")],
        ),
    }
}

fn inlines_to_expr(Inlines(inlines): &Inlines) -> Expr {
    let spans = inlines.iter().map(inline_to_expr).collect();

    // {MarkdownElement[__]...}
    Expr::list(spans)
}

fn inline_to_expr(span: &Inline) -> Expr {
    let inline_args = match span {
        Inline::Text(string) => {
            // MarkdownElement["Text", "string"]
            vec![Expr::string("Text"), Expr::string(string)]
        },
        Inline::Emphasis(inlines) => {
            // MarkdownElement["Emphasis", {...}]
            vec![Expr::string("Emphasis"), inlines_to_expr(inlines)]
        },
        Inline::Strong(inlines) => {
            // MarkdownElement["Strong", {...}]
            vec![Expr::string("Strong"), inlines_to_expr(inlines)]
        },
        Inline::Strikethrough(inlines) => {
            // MarkdownElement["Strikethrough", {...}]
            vec![Expr::string("Strikethrough"), inlines_to_expr(inlines)]
        },
        // MarkdownElement["Code", "code"]
        Inline::Code(code) => vec![Expr::string("Code"), Expr::string(code)],
        // MarkdownElement["Hyperlink", label, destination]
        Inline::Link {
            // FIXME: Pass through this link type
            link_type: _,
            // FIXME: Pass through this link title as well
            title: _,
            dest_url,
            // FIXME: Pass through this link id
            id: _,
            content_text,
        } => vec![
            Expr::string("Hyperlink"),
            inlines_to_expr(content_text),
            Expr::string(dest_url),
        ],
        Inline::SoftBreak => vec![Expr::string("SoftBreak")],
        Inline::HardBreak => vec![Expr::string("HardBreak")],
    };

    Expr::normal(Symbol::new(MarkdownElement), inline_args)
}

fn list_item_to_expr(ListItem(blocks): &ListItem) -> Expr {
    let blocks = blocks.iter().map(block_to_expr).collect();

    Expr::normal(
        Symbol::new(MarkdownElement),
        vec![Expr::string("ListItem"), Expr::list(blocks)],
    )
}

//======================================
// Parse expressions to Blocks
//======================================

fn parse_expr_blocks(blocks: &Expr) -> Result<Vec<Block>, String> {
    let blocks = try_headed(blocks, Symbol::new("System`List"))
        .expect("expected 1st argument to be a list");

    let blocks: Vec<Block> = blocks
        .iter()
        .map(parse_expr_to_block)
        .collect::<Result<_, _>>()?;

    Ok(blocks)
}

fn parse_expr_to_block(expr: &Expr) -> Result<Block, String> {
    // TODO(cleanup): Require a standardized argument structure for
    //  MarkdownElement so that indexing into it is easier? (Like XMLElement.)
    let element_args = try_headed(expr, Symbol::new(MarkdownElement))?;

    if element_args.len() < 2 {
        return Err(format!(
            "expected MarkdownElement[..] to have at least 2 args: {expr}",
        ));
    }

    let kind = &element_args[0];

    let kind = kind.try_as_str().ok_or_else(|| {
        "expected MarkdownElement[...] first arg to be string".to_owned()
    })?;

    let ast = match (kind, &element_args[1..]) {
        ("Paragraph", [inlines]) => {
            let inlines = parse_expr_inlines(inlines)?;

            Block::Paragraph(inlines)
        },
        ("Heading", [level, inlines]) => {
            let level = match level.try_as_number() {
                Some(Number::Integer(1)) => HeadingLevel::H1,
                Some(Number::Integer(2)) => HeadingLevel::H1,
                Some(Number::Integer(3)) => HeadingLevel::H1,
                Some(Number::Integer(4)) => HeadingLevel::H1,
                Some(Number::Integer(5)) => HeadingLevel::H1,
                Some(Number::Integer(6)) => HeadingLevel::H1,
                _ => return Err(format!("unsupported heading level value: {level}")),
            };

            let inlines = parse_expr_inlines(inlines)?;

            Block::Heading(level, inlines)
        },
        (other, _) => panic!("unrecognized block MarkdownElement[{other:?}, ..] kind"),
    };

    Ok(ast)
}

fn parse_expr_to_inline(expr: &Expr) -> Result<Inline, String> {
    // TODO(polish): Support a "bare" string converting to
    //  Inline::Text(...)?
    // if let Some(str) = expr.try_as_str() {
    //     return Ok(Inline::Text(str.to_owned()));
    // }

    let element_args = try_headed(expr, Symbol::new(MarkdownElement))?;

    if element_args.len() < 2 {
        return Err(format!(
            "expected MarkdownElement[..] to have at least 2 args: {expr}",
        ));
    }

    let kind = &element_args[0];

    let kind = kind.try_as_str().ok_or_else(|| {
        "expected MarkdownElement[...] first arg to be string".to_owned()
    })?;

    let inline = match (kind, &element_args[1..]) {
        ("Text", [text]) => {
            let text: &str = text.try_as_str().ok_or_else(|| {
                "expected MarkdownElement[\"Text\", ..] 2nd argument to be a string"
                    .to_owned()
            })?;

            Inline::Text(text.to_owned())
        },
        ("Strong", [inlines]) => {
            let inlines = parse_expr_inlines(inlines)?;

            Inline::Strong(inlines)
        },
        (other, _) => panic!("unrecognized inline MarkdownElement[{other:?}, ..] form"),
    };

    Ok(inline)
}

fn parse_expr_inlines(inlines: &Expr) -> Result<Inlines, String> {
    let inlines = try_headed(inlines, Symbol::new("System`List"))
        .expect("expected 1st argument to be a list");

    let inlines: Vec<Inline> = inlines
        .iter()
        .map(parse_expr_to_inline)
        .collect::<Result<_, _>>()?;

    Ok(Inlines(inlines))
}
