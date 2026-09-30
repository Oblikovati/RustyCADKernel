//! Read-only members: comparison, search, numeric values and printing.
use super::HAsciiString;
use crate::standard::{Failure, Handle, exceptions};
use crate::tcollection::AsciiString;
use crate::tcollection::ascii_string::c_string_prefix;

impl HAsciiString {
    /// `IsLess`; `object.is_less(&other)`.
    pub fn is_less(&self, other: &HAsciiString) -> bool {
        let other = other.snapshot();
        self.string().is_less(&other)
    }
    /// `IsGreater`; `object.is_greater(&other)`.
    pub fn is_greater(&self, other: &HAsciiString) -> bool {
        let other = other.snapshot();
        self.string().is_greater(&other)
    }
    /// `IsDifferent`: different lengths, or a difference before the first NUL (`strncmp`);
    /// `Standard_NullObject` for a null handle; `object.is_different(&handle)?`.
    pub fn is_different(&self, other: &Handle<HAsciiString>) -> Result<bool, Failure> {
        let other = non_null(other, "IsDifferent")?.snapshot();
        Ok(!c_strings_match(&self.string(), &other))
    }
    /// `IsSameString(handle)`: equal lengths and no difference before the first NUL;
    /// `object.is_same_string(&handle)?`.
    pub fn is_same_string(&self, other: &Handle<HAsciiString>) -> Result<bool, Failure> {
        let other = non_null(other, "IsSameString")?.snapshot();
        Ok(c_strings_match(&self.string(), &other))
    }
    /// `IsSameString(handle, bool)`: every byte equal, optionally ignoring ASCII case;
    /// `object.is_same_string_with_case(&handle, false)?`.
    pub fn is_same_string_with_case(
        &self,
        other: &Handle<HAsciiString>,
        case_sensitive: bool,
    ) -> Result<bool, Failure> {
        let other = non_null(other, "IsSameString")?.snapshot();
        Ok(AsciiString::same_bytes(
            &*self.string(),
            &other,
            case_sensitive,
        ))
    }
    /// `IsSameState`: the `strncmp` comparison of `IsSameString(handle)`; `object.is_same_state(&other)`.
    pub fn is_same_state(&self, other: &HAsciiString) -> bool {
        let other = other.snapshot();
        c_strings_match(&self.string(), &other)
    }
    /// `FirstLocationInSet`; `object.first_location_in_set(&set, 1, object.length())?`.
    pub fn first_location_in_set(
        &self,
        set: &HAsciiString,
        from: i32,
        to: i32,
    ) -> Result<i32, Failure> {
        let set = set.snapshot();
        self.string().first_location_in_set(&set, from, to)
    }
    /// `FirstLocationNotInSet`; `object.first_location_not_in_set(&set, 1, object.length())?`.
    pub fn first_location_not_in_set(
        &self,
        set: &HAsciiString,
        from: i32,
        to: i32,
    ) -> Result<i32, Failure> {
        let set = set.snapshot();
        self.string().first_location_not_in_set(&set, from, to)
    }
    /// `Location(handle, int, int)`; `object.location(&what, 1, object.length())?`.
    pub fn location(&self, what: &HAsciiString, from: i32, to: i32) -> Result<i32, Failure> {
        let what = what.snapshot();
        self.string().location(&what, from, to)
    }
    /// `Location(int N, char C, int, int)`; `object.location_char(1, b'a', 1, object.length())?`.
    pub fn location_char(&self, count: i32, byte: u8, from: i32, to: i32) -> Result<i32, Failure> {
        self.string().location_char(count, byte, from, to)
    }
    /// `Search(const char*)`; `object.search("ab")`.
    pub fn search(&self, what: impl AsRef<[u8]>) -> i32 {
        self.string().search(c_string_prefix(what.as_ref()))
    }
    /// `Search(handle)`; `object.search_string(&what)`.
    pub fn search_string(&self, what: &HAsciiString) -> i32 {
        let what = what.snapshot();
        self.string().search(&what)
    }
    /// `SearchFromEnd(const char*)`; `object.search_from_end("ab")`.
    pub fn search_from_end(&self, what: impl AsRef<[u8]>) -> i32 {
        self.string()
            .search_from_end(c_string_prefix(what.as_ref()))
    }
    /// `SearchFromEnd(handle)`; `object.search_from_end_string(&what)`.
    pub fn search_from_end_string(&self, what: &HAsciiString) -> i32 {
        let what = what.snapshot();
        self.string().search_from_end(&what)
    }
}

impl HAsciiString {
    /// `IntegerValue`; `HAsciiString::new("12").integer_value()? == 12`.
    pub fn integer_value(&self) -> Result<i32, Failure> {
        self.string().integer_value()
    }
    /// `IsIntegerValue`; `object.is_integer_value()`.
    pub fn is_integer_value(&self) -> bool {
        self.string().is_integer_value()
    }
    /// `RealValue`; `HAsciiString::new("0.5").real_value()? == 0.5`.
    pub fn real_value(&self) -> Result<f64, Failure> {
        self.string().real_value()
    }
    /// `IsRealValue()`: the prefix check, `IsRealValue(false)`; `object.is_real_value()`.
    pub fn is_real_value(&self) -> bool {
        self.string().is_real_value(false)
    }
    /// `IsAscii`; `object.is_ascii()`.
    pub fn is_ascii(&self) -> bool {
        self.string().is_ascii()
    }
    /// `UsefullLength`; `object.useful_length()`.
    pub fn useful_length(&self) -> i32 {
        self.string().useful_length()
    }
    /// `Value`; `object.value(1)?`.
    pub fn value(&self, position: i32) -> Result<u8, Failure> {
        self.string().value(position)
    }
    /// `Print`; `object.print(&mut output)?`.
    pub fn print(&self, output: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.string().print(output)
    }
}

/// The object behind `handle`, or OCCT's `Standard_NullObject` naming `member`.
fn non_null<'a>(
    handle: &'a Handle<HAsciiString>,
    member: &str,
) -> Result<&'a HAsciiString, Failure> {
    handle.get().ok_or_else(|| {
        exceptions::null_object(&format!(
            "TCollection_HAsciiString::{member}: null handle; expected a live TCollection_HAsciiString"
        ))
    })
}

/// Equal lengths and `strncmp(first, second, length) == 0`: bytes after a shared NUL are
/// not compared.
fn c_strings_match(first: &AsciiString, second: &AsciiString) -> bool {
    first.len() == second.len()
        && c_string_prefix(first.as_bytes()) == c_string_prefix(second.as_bytes())
}
