# Development

This document contains information useful to anyone wishing to
contribute to the development of this project.

This repository contains several sub-projects, and consequently it has several
constituent Development.md files:

* [crates/md2nb/docs/Development.md](../crates/md2nb/docs/Development.md)

## Setup

The [`cargo make`](https://github.com/sagiegurari/cargo-make) and unofficial
[`wolfram-cli`](https://github.com/ConnorGray/wolfram-cli) tools must be
installed to run the development commands below.

## Command Quick Reference

#### Run the crate tests:

```shell
$ cargo test
```

#### Build the Markdown paclet

```shell
$ cargo make paclet
```

#### Build and install the Markdown paclet

```shell
$ cargo make paclet-install
```

#### Run the Markdown paclet tests

```shell
$ cargo make paclet-test
```

#### markdown-ast: Run the CommonMark conformance tests

First ensure the `markdown-ast-conformance` test binary is installed:

```shell
$ cargo install --path ./crates/markdown-ast --bin markdown-ast-conformance
```

Then run the conformance tests:

```shell
# Must be in this this directory for spec_tests.py to work
$ cd ./third-party/commonmark-spec
$ python3 test/spec_tests.py --program markdown-ast-conformance
```