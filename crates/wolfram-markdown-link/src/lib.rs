use wolfram_library_link::{
    export,
    expr::{Expr, Symbol},
};

use markdown_ast::{Block, Inline, Inlines, ListItem};

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
        // MarkdownElement["BlockQuote", {...}]
        Block::BlockQuote(blocks) => {
            let blocks = blocks.into_iter().map(block_to_expr).collect();

            Expr::normal(
                Symbol::new(MarkdownElement),
                vec![Expr::string("BlockQuote"), Expr::list(blocks)],
            )
        },
        Block::Table {
            headers: _,
            rows: _,
        } => todo!(),
        Block::Rule => todo!(),
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

fn list_item_to_expr(ListItem(blocks): &ListItem) -> Expr {
    let blocks = blocks.iter().map(block_to_expr).collect();

    Expr::normal(
        Symbol::new(MarkdownElement),
        vec![Expr::string("ListItem"), Expr::list(blocks)],
    )
}
