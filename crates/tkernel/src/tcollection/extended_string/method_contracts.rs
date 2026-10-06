//! Native pointer and returning ownership methods.
use super::ExtendedString;
impl ExtendedString {
    /// Returns native ToExtString pointer identity, including shared empty storage; `text.to_ext_ptr()`.
    pub fn to_ext_ptr(&self) -> *const u16 {
        self.as_utf16_with_nul().as_ptr()
    }
    /// Transfers a native buffer and leaves the source at the shared terminator; `ExtendedString::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Returns the native move-assignment destination; `text.move_assign_returning(&mut source)`.
    pub fn move_assign_returning(&mut self, source: &mut Self) -> &mut Self {
        self.move_assign(source);
        self
    }
    /// Preserves native self-move assignment and returns its destination; `text.move_assign_self()`.
    pub fn move_assign_self(&mut self) -> &mut Self {
        self
    }
    /// Copies a UTF-16 string view including embedded zeros and returns the destination; `text.assign_view(&[65, 0, 66])`.
    pub fn assign_view(&mut self, units: &[u16]) -> &mut Self {
        self.copy(units);
        self
    }
    /// Returns Windows' UTF-16 wchar_t pointer alias; `text.to_wide_ptr()`.
    #[cfg(windows)]
    pub fn to_wide_ptr(&self) -> *const u16 {
        self.to_ext_ptr()
    }
}
