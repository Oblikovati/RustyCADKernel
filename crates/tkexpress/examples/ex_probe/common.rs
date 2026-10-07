//! The token printers of tools/oracle/tkexpress_probe.cxx: `x`-prefixed hexadecimal
//! texts, doubles as bit patterns, guarded steps that print `!` and the exception type.
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::fmt::Write as _;

/// The tokens of the current answer line (`ex_out`).
#[derive(Default)]
pub struct ProbeLine {
    text: String,
}
impl ProbeLine {
    /// Append one token (`ex_token`).
    pub fn token(&mut self, token: &str) {
        self.text.push(' ');
        self.text.push_str(token);
    }
    /// `x` and the hexadecimal bytes (`ex_hex`).
    pub fn bytes(&mut self, bytes: &[u8]) {
        let mut text: String = String::from("x");
        for byte in bytes {
            let _ = write!(text, "{byte:02x}");
        }
        self.token(&text);
    }
    /// A text (`ex_text`).
    pub fn text(&mut self, text: &AsciiString) {
        self.bytes(text.as_bytes());
    }
    /// A double as its bit pattern (`ex_real`).
    pub fn real(&mut self, value: f64) {
        self.token(&value.to_bits().to_string());
    }
    /// An integer (`ex_int`).
    pub fn int(&mut self, value: i32) {
        self.token(&value.to_string());
    }
    /// A boolean as 1 or 0 (`ex_bool`).
    pub fn boolean(&mut self, value: bool) {
        self.token(if value { "1" } else { "0" });
    }
    /// One guarded step (`ex_try`): its tokens, or `!` and the exception type. A
    /// Standard_NullObject of the port is a null dereference natively, which the native
    /// probe (with OSD::SetSignal) reports as OSD_SIGSEGV.
    pub fn step(&mut self, step: impl FnOnce(&mut Self) -> Result<(), Failure>) {
        if let Err(failure) = step(self) {
            let name: &str = match failure.exception_type() {
                "Standard_NullObject" => "OSD_SIGSEGV",
                other => other,
            };
            self.token(&format!("!{name}"));
        }
    }
    /// Print the line and start a new one (`ex_end`).
    pub fn end(&mut self) {
        println!("{}", self.text);
        self.text.clear();
    }
}

/// The bytes of an `x`-prefixed hexadecimal token (`ex_unhex`).
pub fn unhex(token: &str) -> Vec<u8> {
    let digits: &[u8] = token.as_bytes().get(1..).unwrap_or_default();
    digits
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair.as_slice()).unwrap_or("00"), 16)
                .unwrap_or(0)
        })
        .collect()
}

/// `TCollection_AsciiString(ex_unhex(token).c_str())`: the text up to its first NUL.
pub fn c_text(token: &str) -> AsciiString {
    let bytes: Vec<u8> = unhex(token);
    let end: usize = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    AsciiString::from_bytes(&bytes[..end])
}

/// A double from its decimal bit pattern (`ex_bits`).
pub fn bits(token: &str) -> f64 {
    f64::from_bits(token.parse().unwrap_or(0))
}
