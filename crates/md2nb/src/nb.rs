use markdown_ast::{Block, HeadingLevel, Inline, Inlines, ListItem};

use wolfram_expr::{Expr, Symbol};


#[derive(Debug, Clone)]
pub struct Options {
    pub create_external_language_cells: bool,
}

struct State {
    list_depth: u8,
}

pub fn block_to_cells(block: Block, opts: &Options) -> Vec<Expr> {
    let mut state = State { list_depth: 0 };

    block_to_cells_(&mut state, opts, block)
}

fn block_to_cells_(
    state: &mut State,
    opts: &Options,
    block: Block,
) -> Vec<Expr> {
    match block {
        Block::Heading(level, text) => {
            let style = match level {
                HeadingLevel::H1 => "Title",
                HeadingLevel::H2 => "Chapter",
                HeadingLevel::H3 => "Section",
                HeadingLevel::H4 => "Subsection",
                HeadingLevel::H5 => "Subsubsection",
                HeadingLevel::H6 => "Subsubsubsection",
            };

            vec![Expr::normal(
                Symbol::new("System`Cell"),
                vec![inlines_to_text_data(text), Expr::from(style)],
            )]
        },
        Block::Paragraph(text) => vec![Expr::normal(
            Symbol::new("System`Cell"),
            vec![inlines_to_text_data(text), Expr::from("Text")],
        )],
        Block::List(items) => {
            let mut list_cells = Vec::new();

            state.list_depth += 1;

            for item in items {
                list_cells.extend(list_item_to_cells(state, item));
            }

            state.list_depth -= 1;

            list_cells
        },
        Block::CodeBlock {
            kind,
            code: code_text,
        } => {
            let external_language: Option<&str> =
                // The languages listed here should be all of those currently supported
                // by ExternalEvaluate.
                match kind.info_string().map(|s| s.to_lowercase()).as_deref() {
                    Some("python") => Some("Python"),
                    Some("shell" | "bash" | "sh" | "zsh") => Some("Shell"),
                    Some("julia") => Some("Julia"),
                    Some("r") => Some("R"),
                    Some("octave") => Some("Octave"),
                    Some("java") => Some("Java"),
                    Some("node" | "nodejs" | "js" | "javascript") => Some("NodeJS"),
                    Some("jupyter") => Some("Jupyter"),
                    Some("sql") => Some("SQL"),
                    Some("sql-jdbc") => Some("SQL-JDBC"),
                    Some(_) => None,
                    None => None,
                };

            match external_language {
                // Only create "ExternalLanguage" cells if the option is set (enabled by
                // default).
                Some(lang) if opts.create_external_language_cells => {
                    vec![Expr::normal(
                        Symbol::new("System`Cell"),
                        vec![
                            Expr::string(code_text),
                            Expr::string("ExternalLanguage"),
                            Expr::rule(
                                Symbol::new("System`CellEvaluationLanguage"),
                                Expr::string(lang),
                            ),
                        ],
                    )]
                },
                _ => {
                    vec![Expr::normal(
                        Symbol::new("System`Cell"),
                        vec![Expr::string(code_text), Expr::string("Program")],
                    )]
                },
            }
        },
        Block::BlockQuote {
            kind: _,
            blocks: quote_blocks,
        } => {
            let quote_cells: Vec<Expr> = quote_blocks
                .into_iter()
                .flat_map(|block| block_to_cells(block, opts))
                .collect();

            // TODO: Use a dedicated "BlockQuote" cell style. There is no "BlockQuote"
            //       style in the default Wolfram notebook stylesheet, but we could add
            //       a StyleData definition to this notebook.
            let cell = Expr::normal(
                Symbol::new("System`Cell"),
                vec![
                    Expr::normal(
                        Symbol::new("System`BoxData"),
                        vec![Expr::list(quote_cells)],
                    ),
                    Expr::string("Text"),
                    // Only the left side should have a frame:
                    //   CellFrame -> {{4, 0}, {0, 0}}
                    Expr::rule(
                        Symbol::new("System`CellFrame"),
                        Expr::list(vec![
                            Expr::list(vec![Expr::from(4), Expr::from(0)]),
                            Expr::list(vec![Expr::from(0), Expr::from(0)]),
                        ]),
                    ),
                    // The cell frame should have a medium-light gray color:
                    //   CellFrameColor -> GrayLevel[0.8]
                    Expr::rule(
                        Symbol::new("System`CellFrameColor"),
                        Expr::normal(
                            Symbol::new("System`GrayLevel"),
                            vec![Expr::real(0.8)],
                        ),
                    ),
                    // The cell background should be a light gray color:
                    //   Background -> GrayLevel[0.95]
                    Expr::rule(
                        Symbol::new("System`Background"),
                        Expr::normal(
                            Symbol::new("System`GrayLevel"),
                            vec![Expr::real(0.95)],
                        ),
                    ),
                ],
            );
            vec![cell]
        },
        // FIXME: Process the `alignments`
        Block::Table {
            alignments: _,
            headers,
            rows,
        } => {
            let mut grid_rows: Vec<Expr> = Vec::new();

            let header_row = headers
                .into_iter()
                .map(|content: Inlines| {
                    Expr::normal(
                        Symbol::new("System`Cell"),
                        vec![
                            inlines_to_text_data(content),
                            Expr::from("Subsubsubsection"),
                        ],
                    )
                })
                .collect();

            grid_rows.push(Expr::list(header_row));

            for row_content in rows {
                let row: Vec<Expr> = row_content
                    .into_iter()
                    .map(|content: Inlines| {
                        Expr::normal(
                            Symbol::new("System`Cell"),
                            vec![
                                inlines_to_text_data(content),
                                Expr::from("Text"),
                            ],
                        )
                    })
                    .collect();

                grid_rows.push(Expr::list(row));
            }

            let grid_box = Expr::normal(
                Symbol::new("System`GridBox"),
                vec![
                    Expr::list(grid_rows),
                    // GridBoxItemSize -> {
                    //     "Columns" -> {{Automatic}},
                    //     "Rows" -> {{Automatic}}
                    // }
                    Expr::rule(
                        Symbol::new("System`GridBoxItemSize"),
                        Expr::list(vec![
                            Expr::rule(
                                Expr::from("Columns"),
                                Expr::list(vec![Expr::list(vec![Expr::from(
                                    Symbol::new("System`Automatic"),
                                )])]),
                            ),
                            Expr::rule(
                                Expr::from("Rows"),
                                Expr::list(vec![Expr::list(vec![Expr::from(
                                    Symbol::new("System`Automatic"),
                                )])]),
                            ),
                        ]),
                    ),
                ],
            );

            vec![Expr::normal(
                Symbol::new("System`Cell"),
                vec![
                    Expr::normal(Symbol::new("System`BoxData"), vec![grid_box]),
                    Expr::from("Text"),
                ],
            )]
        },
        Block::Rule => {
            // Note: This formatting is based on the menu item:
            //         Insert > Horizontal Line > Paste Thick Line Object
            // TODO:
            //     Support inserting lines of different thickness, and with different
            //     left-side margins based on context.
            //
            //     For example, `***` is supported within block quotes. Improve how it
            //     looks when used in that context.
            vec![Expr::normal(
                Symbol::new("System`Cell"),
                vec![
                    Expr::string(""),
                    Expr::string("Text"),
                    // Editable->False,
                    Expr::rule(
                        Symbol::new("System`Editable"),
                        Expr::symbol(Symbol::new("System`False")),
                    ),
                    // Selectable->False,
                    // CellFrame->{{0, 0}, {0, 3}},
                    Expr::rule(
                        Symbol::new("System`CellFrame"),
                        Expr::list(vec![
                            Expr::list(vec![Expr::from(0), Expr::from(0)]),
                            Expr::list(vec![Expr::from(0), Expr::from(3)]),
                        ]),
                    ),
                    // ShowCellBracket->False,
                    Expr::rule(
                        Symbol::new("System`ShowCellBracket"),
                        Expr::symbol(Symbol::new("System`False")),
                    ),
                    // CellMargins->{{0, 0}, {1, 1}},
                    Expr::rule(
                        Symbol::new("System`CellMargins"),
                        Expr::list(vec![
                            Expr::list(vec![Expr::from(0), Expr::from(0)]),
                            Expr::list(vec![Expr::from(1), Expr::from(1)]),
                        ]),
                    ),
                    // CellElementSpacings->{"CellMinHeight"->1},
                    Expr::rule(
                        Symbol::new("System`CellElementSpacings"),
                        Expr::list(vec![Expr::rule(
                            Expr::from("CellMinHeight"),
                            Expr::from(1),
                        )]),
                    ),
                    // CellFrameMargins->0,
                    Expr::rule(
                        Symbol::new("System`CellFrameMargins"),
                        Expr::from(0),
                    ),
                    // CellFrameColor->GrayLevel[0.75],
                    Expr::rule(
                        Symbol::new("System`CellFrameColor"),
                        Expr::normal(
                            Symbol::new("System`GrayLevel"),
                            vec![Expr::real(0.75)],
                        ),
                    ),
                    // CellSize->{Inherited, 5}
                    Expr::rule(
                        Symbol::new("System`CellSize"),
                        Expr::list(vec![
                            Expr::from(Symbol::new("System`Inherited")),
                            Expr::from(5),
                        ]),
                    ),
                ],
            )]
        },
    }
}

fn list_item_to_cells(
    state: &mut State,
    ListItem(blocks): ListItem,
) -> Vec<Expr> {
    let mut cells = vec![];

    for block in blocks {
        match block {
            Block::Paragraph(text) => {
                let style = match state.list_depth {
                    0 => panic!(),
                    1 => "Item",
                    2 => "Subitem",
                    3 => "Subsubitem",
                    _ => todo!("return list depth error"),
                };

                cells.push(Expr::normal(
                    Symbol::new("System`Cell"),
                    vec![inlines_to_text_data(text), Expr::from(style)],
                ));
            },
            Block::List(items) => {
                let mut list_cells = Vec::new();

                state.list_depth += 1;

                for item in items {
                    list_cells.extend(list_item_to_cells(state, item));
                }

                state.list_depth -= 1;

                cells.extend(list_cells);
            },
            Block::BlockQuote { kind: _, blocks: _ } => {
                todo!("handle markdown block quote inside list items")
            },
            Block::Heading(_, _) => {
                todo!("handle markdown headings inside list items")
            },
            Block::CodeBlock { .. } => {
                todo!("handle markdown code block inside list item")
            },
            Block::Table { .. } => {
                todo!("handle markdown table inside list item")
            },
            Block::Rule => todo!("handle markdown rule inside list item"),
        }
    }

    cells
}

/// Returns a `TextData[{...}]` expression.
fn inlines_to_text_data(inlines: Inlines) -> Expr {
    Expr::normal(Symbol::new("System`TextData"), vec![text_to_boxes(inlines)])
}

// Returns a `RowBox[{...}]` expression.
fn text_to_boxes(text: Inlines) -> Expr {
    let mut row = Vec::new();

    for span in text {
        let expr = match span {
            Inline::Text(text) => Expr::string(text),
            Inline::Emphasis(inlines) => Expr::normal(
                Symbol::new("System`StyleBox"),
                vec![
                    text_to_boxes(inlines),
                    Expr::rule(
                        Symbol::new("System`FontSlant"),
                        Expr::symbol(Symbol::new("System`Italic")),
                    ),
                ],
            ),
            Inline::Strong(inlines) => Expr::normal(
                Symbol::new("System`StyleBox"),
                vec![
                    text_to_boxes(inlines),
                    Expr::rule(
                        Symbol::new("System`FontWeight"),
                        Expr::symbol(Symbol::new("System`Bold")),
                    ),
                ],
            ),
            Inline::Strikethrough(_) => todo!("strikethrough text"),
            Inline::Code(code) => Expr::normal(
                Symbol::new("System`StyleBox"),
                vec![Expr::string(code), Expr::string("Code")],
            ),
            Inline::Link {
                // FIXME: Pass through this link type.
                link_type: _,
                // FIXME: Pass through this link title.
                title: _,
                dest_url,
                // FIXME: Pass through this link id.
                id: _,
                content_text,
            } => Expr::normal(
                Symbol::new("System`ButtonBox"),
                vec![
                    text_to_boxes(content_text),
                    Expr::normal(
                        Symbol::new("System`Rule"),
                        vec![
                            Expr::from(Symbol::new("System`BaseStyle")),
                            Expr::string("Hyperlink"),
                        ],
                    ),
                    Expr::normal(
                        Symbol::new("System`Rule"),
                        vec![
                            Expr::from(Symbol::new("System`ButtonData")),
                            Expr::normal(
                                Symbol::new("System`List"),
                                vec![
                                    Expr::normal(
                                        Symbol::new("System`URL"),
                                        vec![Expr::string(dest_url.clone())],
                                    ),
                                    Expr::from(Symbol::new("System`None")),
                                ],
                            ),
                        ],
                    ),
                    Expr::normal(
                        Symbol::new("System`Rule"),
                        vec![
                            Expr::from(Symbol::new("System`ButtonNote")),
                            Expr::string(dest_url),
                        ],
                    ),
                ],
            ),
            Inline::Image { .. } => {
                todo!("Support Image link conversion to notebook")
            },
            Inline::SoftBreak => Expr::string(" "),
            Inline::HardBreak => Expr::string("\n"),
        };

        row.push(expr);
    }

    Expr::normal(
        Symbol::new("System`RowBox"),
        vec![Expr::normal(Symbol::new("System`List"), row)],
    )
}
