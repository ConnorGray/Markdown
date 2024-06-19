# Markdown

#### [API Documentation](https://docs.rs/markdown-ast) | [Changelog](./docs/CHANGELOG-markdown-ast.md) | [Contributing](#contributing)

This repository contains two projects for working with Markdown
documents:

* `markdown-ast` — a Rust crate modeling Markdown
  syntax as an AST.

* `ConnorGray/Markdown` — a Wolfram paclet providing a
  symbolic representation of Markdown elements, and (**TODO**) notebook frontend
  support for opening and editing .md files.

## Quick Examples

Parse a Markdown document into an AST in Rust:

```rust
use markdown_ast::{markdown_to_ast, Block, Inline, Inlines};

let ast = markdown_to_ast("
Hello! This is a paragraph **with bold text**.
");

assert_eq!(ast, vec![
    Block::Paragraph(Inlines(vec![
        Inline::Text("Hello! This is a paragraph ".to_owned()),
        Inline::Strong(Inlines(vec![
            Inline::Text("with bold text".to_owned()),
        ])),
        Inline::Text(".".to_owned())
    ]))
]);
```

## File Overview

* [`./crates/markdown-ast`](./crates/markdown-ast/): source code for the
  general-purpose `markdown-ast` crate.

* [`./paclets/Markdown/`](./Markdown/): source code for the
  `ConnorGray/Markdown` paclet.

* [`./crates/md2nb/`](./crates/md2nb): source code for the
  [`md2nb`](https://crates.io/crates/md2nb) command-line utility.

* [`./crates/wolfram-markdown-link`](./crates/wolfram-markdown-link/): source
  code for the LibraryLink library used by the Markdown paclet.

* [`third-party/commonmark-spec/`](./third-party/): git submodule of the
  [commonmark-spec](https://github.com/commonmark/commonmark-spec/) repository.
  Used by the `markdown-ast` conformance tests.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

* MIT license
  ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

### Developer Notes

See [**Development.md**](./docs/Development.md) for instructions on how to
perform common development tasks when contributing to this repository.