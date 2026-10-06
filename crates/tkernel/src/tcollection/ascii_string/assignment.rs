//! OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 returns the destination from copy assignment.
use super::AsciiString;

impl AsciiString {
    /// Copies a complete string and returns the destination; `text.assign_from(&source).append(&tail)`.
    pub fn assign_from(&mut self, source: &Self) -> &mut Self {
        self.copy(source);
        self
    }

    /// Models native self-assignment without conflicting Rust borrows; `text.assign_self()`.
    /// The native Copy guard preserves the original buffer and all embedded zero bytes.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}
