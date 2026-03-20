use std::path::Path;

use mahalaxmi_orchestration::context::{chunker::select_chunks, CodeChunk, CodeChunker};

#[test]
fn chunk_empty_file() {
    let chunker = CodeChunker::with_default_config();
    let chunks = chunker.chunk_file(Path::new("test.rs"), "", 1000);
    assert!(chunks.is_empty());
}

#[test]
fn chunk_small_file_single_chunk() {
    let chunker = CodeChunker::with_default_config();
    let content = "fn main() {\n    println!(\"hello\");\n}\n";
    let chunks = chunker.chunk_file(Path::new("main.rs"), content, 1000);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].start_line, 1);
    assert_eq!(chunks[0].end_line, 3);
}

#[test]
fn chunk_large_file_multiple_chunks() {
    let chunker = CodeChunker::with_default_config();
    // Create a file with ~200 chars per line, 20 lines = ~4000 chars = ~1000 tokens
    let line = "x".repeat(200);
    let content: String = (0..20)
        .map(|_| line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let chunks = chunker.chunk_file(Path::new("big.rs"), &content, 100);
    assert!(chunks.len() > 1);
}

#[test]
fn chunk_by_lines_splits_correctly() {
    let chunker = CodeChunker::with_default_config();
    // Each line is 40 chars = 10 tokens. Budget of 25 tokens fits ~2.5 lines.
    let lines: Vec<String> = (0..10)
        .map(|i| format!("// line {} {}", i, "x".repeat(30)))
        .collect();
    let content = lines.join("\n");
    let chunks = chunker.chunk_by_lines(Path::new("test.rs"), &content, 25);
    assert!(chunks.len() > 1);
    // All content should be covered
    let total_lines: usize = chunks.iter().map(|c| c.line_count()).sum();
    assert!(total_lines >= 10);
}

#[test]
fn code_chunk_line_count() {
    let chunk = CodeChunk::new("test.rs", "line1\nline2\nline3", 1, 3);
    assert_eq!(chunk.line_count(), 3);
}

#[test]
fn code_chunk_is_empty() {
    let chunk = CodeChunk::new("test.rs", "", 1, 0);
    assert!(chunk.is_empty());

    let chunk2 = CodeChunk::new("test.rs", "hello", 1, 1);
    assert!(!chunk2.is_empty());
}

#[test]
fn code_chunk_add_symbol() {
    let mut chunk = CodeChunk::new("test.rs", "fn foo() {}", 1, 1);
    chunk.add_symbol("foo");
    assert_eq!(chunk.symbols, vec!["foo"]);
}

#[test]
fn code_chunk_token_estimate() {
    let content = "a".repeat(100);
    let chunk = CodeChunk::new("test.rs", &content, 1, 1);
    assert_eq!(chunk.token_estimate, 25); // 100 / 4
}

#[test]
fn select_chunks_greedy() {
    let c1 = CodeChunk::new("a.rs", "a".repeat(40), 1, 1); // 10 tokens
    let c2 = CodeChunk::new("b.rs", "b".repeat(80), 1, 1); // 20 tokens
    let c3 = CodeChunk::new("c.rs", "c".repeat(40), 1, 1); // 10 tokens
    let chunks = vec![c1, c2, c3];

    let selected = select_chunks(&chunks, 25);
    // c1 (10) + c2 (20) = 30 > 25, so only c1 + c3
    // Actually greedy: c1 (10) fits, c2 (20) -> 30 > 25, skip; c3 (10) -> 20 <= 25, take
    assert_eq!(selected.len(), 2);
}

#[test]
fn select_chunks_empty_budget() {
    let c1 = CodeChunk::new("a.rs", "hello", 1, 1);
    let chunks = vec![c1];
    let selected = select_chunks(&chunks, 0);
    assert!(selected.is_empty());
}
