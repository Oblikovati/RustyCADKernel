//! C-string and char-array adapters for ASCII string assignment and insertion.
use super::AsciiString;
use crate::standard::Failure;
use crate::standard::c_string::c_str_prefix;

impl AsciiString {
    /// Copies a nullable C string through its first NUL; `text.copy_c_string(Some(b"a\0b"))`.
    pub fn copy_c_string(&mut self, input: Option<&[u8]>) {
        let bytes: &[u8] = input.map_or(&[], c_str_prefix);
        self.copy(bytes);
    }

    /// Copies a char array through its first NUL like `Copy(const char(&)[N])`.
    /// Example: `text.copy_literal(b"prefix\0ignored");`.
    pub fn copy_literal<const N: usize>(&mut self, input: &[u8; N]) {
        self.copy_c_string(Some(input));
    }

    /// Inserts a nullable C-string prefix before a one-based position; null is a no-op.
    /// Example: `text.insert_c_string(2, Some(b"x\0ignored"))?`.
    pub fn insert_c_string(&mut self, position: i32, input: Option<&[u8]>) -> Result<(), Failure> {
        let Some(bytes) = input else {
            return Ok(());
        };
        let prefix: &[u8] = c_str_prefix(bytes);
        self.insert(position, prefix)
    }

    /// Inserts a char-array C-string prefix before a one-based position.
    /// Example: `text.insert_literal(2, b"x\0ignored")?`.
    pub fn insert_literal<const N: usize>(
        &mut self,
        position: i32,
        input: &[u8; N],
    ) -> Result<(), Failure> {
        self.insert_c_string(position, Some(input))
    }

    /// Inserts a nullable C-string prefix after an index; null is a no-op without range checks.
    /// Example: `text.insert_after_c_string(1, Some(b"x\0ignored"))?`.
    pub fn insert_after_c_string(
        &mut self,
        index: i32,
        input: Option<&[u8]>,
    ) -> Result<(), Failure> {
        let Some(bytes) = input else {
            return Ok(());
        };
        let prefix: &[u8] = c_str_prefix(bytes);
        self.insert_after(index, prefix)
    }

    /// Inserts a char-array C-string prefix after an index.
    /// Example: `text.insert_after_literal(1, b"x\0ignored")?`.
    pub fn insert_after_literal<const N: usize>(
        &mut self,
        index: i32,
        input: &[u8; N],
    ) -> Result<(), Failure> {
        self.insert_after_c_string(index, Some(input))
    }

    /// Inserts a nullable C-string prefix before an index; null is a no-op without range checks.
    /// Example: `text.insert_before_c_string(2, Some(b"x\0ignored"))?`.
    pub fn insert_before_c_string(
        &mut self,
        index: i32,
        input: Option<&[u8]>,
    ) -> Result<(), Failure> {
        let Some(bytes) = input else {
            return Ok(());
        };
        let prefix: &[u8] = c_str_prefix(bytes);
        self.insert_before(index, prefix)
    }

    /// Inserts a char-array C-string prefix before an index.
    /// Example: `text.insert_before_literal(2, b"x\0ignored")?`.
    pub fn insert_before_literal<const N: usize>(
        &mut self,
        index: i32,
        input: &[u8; N],
    ) -> Result<(), Failure> {
        self.insert_before_c_string(index, Some(input))
    }
}
