use std::io::Read;

use markdown_ast::markdown_to_ast;

fn main() {
    let mut stdin = std::io::stdin().lock();

    let mut content = String::new();
    stdin.read_to_string(&mut content).unwrap();

    // If this doesn't fail with a panic, that implies that the flat
    // pulldown-cmark event stream was successfully converted into a sequence
    // of structured markdown_ast::Block's.
    let _ = markdown_to_ast(&content);
}
