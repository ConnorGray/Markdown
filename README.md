# Markdown for Wolfram

This repository contains code and tools for processing Markdown from Wolfram.

## File Overview

* [`./crates/markdown-ast`](./crates/markdown-ast/): source code for the
  general-purpose `markdown-ast` crate.

* [`./crates/md2nb/`](./crates/md2nb): source code for the
  [`md2nb`](https://crates.io/crates/md2nb) command-line utility.

* [`./crates/wolfram-markdown-link`](./crates/wolfram-markdown-link/): source
  code for the LibraryLink library used by the Markdown paclet.

* [`./paclets/Markdown/`](./Markdown/): source code for the
  `ConnorGray/Markdown` paclet.

* [`third-party/commonmark-spec/`](./third-party/): git submodule of the
  [commonmark-spec](https://github.com/commonmark/commonmark-spec/) repository.
  Used by the `markdown-ast` conformance tests.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

* MIT license
  ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

### Developer Notes

See [**Development.md**](./docs/Development.md) for instructions on how to
perform common development tasks when contributing to this repository.