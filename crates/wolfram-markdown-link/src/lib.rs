use wolfram_library_link::{
    export,
    expr::{Expr, Symbol},
};

use markdown_ast::{Block, Inline, Inlines, ListItem, TextStyle};

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

    let ast: Vec<Block> = markdown_ast::parse(s);

    let ast: Vec<Expr> = ast.iter().map(block_to_expr).collect();

    Expr::list(ast)
}

//======================================
// Expr Conversion
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
        Block::Heading(_, _) => todo!(),
        Block::CodeBlock {
            info_string: _,
            code: _,
        } => todo!(),
        Block::BlockQuote(_) => todo!(),
        Block::Table {
            headers: _,
            rows: _,
        } => todo!(),
        Block::Rule => todo!(),
    }
}

fn inlines_to_expr(Inlines(inlines): &Inlines) -> Expr {
    let spans = inlines.iter().map(inline_to_expr).collect();

    // {Markdown`Inline[__]...}
    Expr::list(spans)
}

fn inline_to_expr(span: &Inline) -> Expr {
    let inline_args = match span {
        Inline::Text(string, styles) => {
            let mut styles_exprs = styles.iter().map(text_style_to_expr);
            let style_expr = match styles.len() {
                0 => Expr::list(vec![]),
                1 => styles_exprs.next().unwrap(),
                _ => Expr::list(styles_exprs.collect()),
            };
            // let args: Vec<Expr> = std::iter::once(Expr::string(string))
            //     .chain(styles.iter().map(text_style_to_expr))
            //     .collect();

            // MarkdownElement["Text", string, styles]
            vec![Expr::string("Text"), Expr::string(string), style_expr]
        },
        // MarkdownElement["Code", code]
        Inline::Code(code) => vec![Expr::string("Code"), Expr::string(code)],
        // MarkdownElement["Hyperlink", label, destination]
        Inline::Link { label, destination } => vec![
            Expr::string("Hyperlink"),
            inlines_to_expr(label),
            Expr::string(destination),
        ],
        Inline::SoftBreak => vec![Expr::string("SoftBreak")],
        Inline::HardBreak => vec![Expr::string("HardBreak")],
    };

    Expr::normal(Symbol::new(MarkdownElement), inline_args)
}

// FIXME: This is the wrong level of abstraction, in particular the
//        strikethrough. Represent this symbolically, and convert to native
//        WL styles once we're in Wolfram, using some helper function.
fn text_style_to_expr(style: &TextStyle) -> Expr {
    match style {
        TextStyle::Emphasis => Expr::from(Symbol::new("System`Italic")),
        TextStyle::Strong => Expr::from(Symbol::new("System`Bold")),
        TextStyle::Strikethrough => Expr::string("StrikeThrough"),
    }
}

fn list_item_to_expr(ListItem(blocks): &ListItem) -> Expr {
    let blocks = blocks.iter().map(block_to_expr).collect();

    Expr::list(blocks)
}
