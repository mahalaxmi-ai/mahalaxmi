use vte::{Params, Parser, Perform};

/// Internal performer that collects printable characters.
///
/// Separated from `VtCleaner` to avoid borrow conflicts with `Parser::advance`.
struct VtPerformer {
    output: Vec<u8>,
}

impl VtPerformer {
    fn new() -> Self {
        Self { output: Vec::new() }
    }
}

impl Perform for VtPerformer {
    fn print(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.output.extend_from_slice(s.as_bytes());
    }

    fn execute(&mut self, byte: u8) {
        if byte == b'\n' || byte == b'\r' {
            self.output.push(byte);
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn csi_dispatch(
        &mut self,
        _params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
    }
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

/// Strips ANSI/VT escape sequences from terminal output, producing clean text.
///
/// Uses the `vte` crate's state machine parser for correctness — handles all
/// CSI, OSC, ESC, and DCS sequences without fragile regex.
///
/// **Statefulness**: The VTE parser is stateful — it remembers when it's mid-escape-sequence
/// across calls to `clean()`. This means if a read boundary splits an escape sequence like
/// `\x1b[31` | `mhello`, the first call processes `\x1b[31` (enters CSI state), and the
/// second call continues with `m` (completes the CSI) then outputs `hello`.
///
/// **Partial UTF-8 handling**: PTY reads may split multibyte UTF-8 characters at arbitrary
/// byte boundaries. The `utf8_pending` buffer holds incomplete trailing bytes that don't
/// form a valid UTF-8 sequence. On the next `clean()` call, the pending bytes are prepended
/// to the new input before processing.
pub struct VtCleaner {
    parser: Parser,
    performer: VtPerformer,
    /// Incomplete UTF-8 bytes from the end of the previous `clean()` call.
    utf8_pending: Vec<u8>,
}

impl VtCleaner {
    /// Create a new VT cleaner.
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            performer: VtPerformer::new(),
            utf8_pending: Vec::new(),
        }
    }

    /// Strip escape sequences from the input bytes, returning clean UTF-8 text.
    ///
    /// Handles partial UTF-8 sequences split across calls.
    pub fn clean(&mut self, input: &[u8]) -> String {
        self.performer.output.clear();

        let data = if self.utf8_pending.is_empty() {
            input.to_vec()
        } else {
            let mut combined = std::mem::take(&mut self.utf8_pending);
            combined.extend_from_slice(input);
            combined
        };

        let split_point = find_utf8_safe_split(&data);
        let (safe, remainder) = data.split_at(split_point);

        self.parser.advance(&mut self.performer, safe);

        self.utf8_pending = remainder.to_vec();

        String::from_utf8_lossy(&self.performer.output).into_owned()
    }

    /// Strip escape sequences from a string.
    pub fn clean_str(&mut self, input: &str) -> String {
        self.performer.output.clear();
        self.parser.advance(&mut self.performer, input.as_bytes());
        String::from_utf8_lossy(&self.performer.output).into_owned()
    }

    /// Flush any pending UTF-8 bytes. Call this on EOF.
    pub fn flush(&mut self) -> String {
        if self.utf8_pending.is_empty() {
            return String::new();
        }
        let pending = std::mem::take(&mut self.utf8_pending);
        String::from_utf8_lossy(&pending).into_owned()
    }
}

/// Find the byte index at which it's safe to split the buffer for UTF-8 decoding.
fn find_utf8_safe_split(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }
    for i in 1..=3.min(data.len()) {
        let idx = data.len() - i;
        let byte = data[idx];
        if byte & 0x80 == 0 {
            return data.len();
        }
        if byte & 0xC0 == 0xC0 {
            let expected_len = if byte & 0xF8 == 0xF0 {
                4
            } else if byte & 0xF0 == 0xE0 {
                3
            } else if byte & 0xE0 == 0xC0 {
                2
            } else {
                1
            };
            let available = data.len() - idx;
            if available < expected_len {
                return idx;
            }
            return data.len();
        }
    }
    data.len()
}

impl Default for VtCleaner {
    fn default() -> Self {
        Self::new()
    }
}
