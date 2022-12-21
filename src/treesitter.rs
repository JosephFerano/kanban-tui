use tree_sitter::{Parser, Language};
use std::path::PathBuf;

pub fn compile_md_grammar() {
    let dir: PathBuf = ["tree-sitter-javascript", "src"].iter().collect();

    cc::Build::new()
        .include(&dir)
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile("tree-sitter-javascript")
}

