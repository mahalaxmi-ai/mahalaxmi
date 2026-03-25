use mahalaxmi_pty::VtCleaner;

// ---------------------------------------------------------------------------
// Basic Cleaning Tests
// ---------------------------------------------------------------------------

#[test]
fn plain_text_passthrough() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean(b"hello world");
    assert_eq!(result, "hello world");
}

#[test]
fn strips_color_codes() {
    let mut cleaner = VtCleaner::new();
    // Red text: \x1b[31m hello \x1b[0m
    let input = b"\x1b[31mhello\x1b[0m";
    let result = cleaner.clean(input);
    assert_eq!(result, "hello");
}

#[test]
fn strips_bold_underline() {
    let mut cleaner = VtCleaner::new();
    // Bold: \x1b[1m, Underline: \x1b[4m, Reset: \x1b[0m
    let input = b"\x1b[1mbold\x1b[0m \x1b[4munderline\x1b[0m";
    let result = cleaner.clean(input);
    assert_eq!(result, "bold underline");
}

#[test]
fn strips_cursor_movement() {
    let mut cleaner = VtCleaner::new();
    // Cursor up: \x1b[A, Cursor right: \x1b[C
    let input = b"\x1b[Ahello\x1b[C world";
    let result = cleaner.clean(input);
    assert_eq!(result, "hello world");
}

#[test]
fn preserves_newlines() {
    let mut cleaner = VtCleaner::new();
    let input = b"line1\nline2\nline3";
    let result = cleaner.clean(input);
    assert_eq!(result, "line1\nline2\nline3");
}

#[test]
fn preserves_carriage_return() {
    let mut cleaner = VtCleaner::new();
    let input = b"progress\r100%";
    let result = cleaner.clean(input);
    assert_eq!(result, "progress\r100%");
}

#[test]
fn empty_input() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean(b"");
    assert_eq!(result, "");
}

#[test]
fn only_escapes() {
    let mut cleaner = VtCleaner::new();
    let input = b"\x1b[31m\x1b[1m\x1b[0m";
    let result = cleaner.clean(input);
    assert_eq!(result, "");
}

#[test]
fn clean_str_works() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean_str("\x1b[32mgreen text\x1b[0m");
    assert_eq!(result, "green text");
}

#[test]
fn complex_prompt() {
    let mut cleaner = VtCleaner::new();
    // Simulated shell prompt with colors
    let input = b"\x1b[1;32muser@host\x1b[0m:\x1b[1;34m~/project\x1b[0m$ ";
    let result = cleaner.clean(input);
    assert_eq!(result, "user@host:~/project$ ");
}

#[test]
fn strips_osc_title() {
    let mut cleaner = VtCleaner::new();
    // OSC 0 (set title) terminated by BEL
    let input = b"\x1b]0;My Title\x07visible text";
    let result = cleaner.clean(input);
    assert_eq!(result, "visible text");
}

#[test]
fn strips_256_color() {
    let mut cleaner = VtCleaner::new();
    // 256-color: \x1b[38;5;196m (red)
    let input = b"\x1b[38;5;196mred text\x1b[0m";
    let result = cleaner.clean(input);
    assert_eq!(result, "red text");
}

#[test]
fn default_impl() {
    let mut cleaner = VtCleaner::default();
    let result = cleaner.clean(b"hello");
    assert_eq!(result, "hello");
}

// ---------------------------------------------------------------------------
// Partial UTF-8 Tests
// ---------------------------------------------------------------------------

#[test]
fn multibyte_intact() {
    let mut cleaner = VtCleaner::new();
    let result = cleaner.clean("こんにちは".as_bytes());
    assert_eq!(result, "こんにちは");
}

#[test]
fn split_2_byte() {
    let mut cleaner = VtCleaner::new();
    // 'ñ' is U+00F1, encoded as 0xC3 0xB1 (2 bytes)
    let first = cleaner.clean(&[0xC3]);
    // First byte alone can't produce the character
    assert_eq!(first, "");
    let second = cleaner.clean(&[0xB1]);
    assert_eq!(second, "ñ");
}

#[test]
fn split_3_byte() {
    let mut cleaner = VtCleaner::new();
    // '€' is U+20AC, encoded as 0xE2 0x82 0xAC (3 bytes)
    let first = cleaner.clean(&[0xE2, 0x82]);
    assert_eq!(first, "");
    let second = cleaner.clean(&[0xAC]);
    assert_eq!(second, "€");
}

#[test]
fn split_4_byte() {
    let mut cleaner = VtCleaner::new();
    // '𝄞' is U+1D11E, encoded as 0xF0 0x9D 0x84 0x9E (4 bytes)
    let first = cleaner.clean(&[0xF0, 0x9D, 0x84]);
    assert_eq!(first, "");
    let second = cleaner.clean(&[0x9E]);
    assert_eq!(second, "𝄞");
}

#[test]
fn mixed_multibyte_with_ascii() {
    let mut cleaner = VtCleaner::new();
    let input = "Hello 世界!".as_bytes();
    let result = cleaner.clean(input);
    assert_eq!(result, "Hello 世界!");
}

// ---------------------------------------------------------------------------
// Split Escape Sequence Tests
// ---------------------------------------------------------------------------

#[test]
fn split_mid_csi() {
    let mut cleaner = VtCleaner::new();
    // Split "\x1b[31m" across two calls: "\x1b[31" and "mhello"
    let first = cleaner.clean(b"\x1b[31");
    assert_eq!(first, "");
    let second = cleaner.clean(b"mhello");
    assert_eq!(second, "hello");
}

#[test]
fn split_between_esc_and_bracket() {
    let mut cleaner = VtCleaner::new();
    // Split between ESC and [: "\x1b" and "[32mtext"
    let first = cleaner.clean(b"\x1b");
    assert_eq!(first, "");
    let second = cleaner.clean(b"[32mtext");
    assert_eq!(second, "text");
}

// ---------------------------------------------------------------------------
// Flush Tests
// ---------------------------------------------------------------------------

#[test]
fn flush_on_eof() {
    let mut cleaner = VtCleaner::new();
    // Send incomplete UTF-8 then flush
    let _partial = cleaner.clean(&[0xC3]);
    let flushed = cleaner.flush();
    // Should get replacement character or the raw byte
    assert!(!flushed.is_empty());
}

#[test]
fn flush_when_empty() {
    let mut cleaner = VtCleaner::new();
    let flushed = cleaner.flush();
    assert_eq!(flushed, "");
}
