//! Resource_Manager lookups and assignments.
use super::{ResourceManager, ResourceUnicode, c_string};
use crate::{
    ncollection::DataMap,
    standard::Failure,
    tcollection::{AsciiString, ExtendedString},
};
impl ResourceManager {
    /// True when the user or reference map holds the resource; `manager.find("language")`.
    pub fn find(&self, name: &str) -> bool {
        let key = c_string(name);
        self.user.seek(&key).is_some() || self.reference.seek(&key).is_some()
    }
    /// The user value, else the reference value (native `Find(theResource, theValue)`);
    /// `manager.find_value(&AsciiString::new("key"))`.
    pub fn find_value(&self, name: &AsciiString) -> Option<AsciiString> {
        self.user
            .seek(name)
            .or_else(|| self.reference.seek(name))
            .cloned()
    }
    /// The user value, else the reference value, else Resource_NoSuchResource carrying the
    /// name; `manager.value("language")?`.
    pub fn value(&self, name: &str) -> Result<&AsciiString, Failure> {
        let key = c_string(name);
        self.user
            .seek(&key)
            .or_else(|| self.reference.seek(&key))
            .ok_or_else(|| no_such_resource(&key))
    }
    /// The value parsed with the native integer-prefix rule, or Standard_TypeMismatch;
    /// `manager.integer("count")?`.
    pub fn integer(&self, name: &str) -> Result<i32, Failure> {
        let value = self.value(name)?;
        if !value.is_integer_value() {
            return Err(type_mismatch(name, "an integer"));
        }
        value.integer_value()
    }
    /// The value parsed with the native real-prefix rule, or Standard_TypeMismatch;
    /// `manager.real("tolerance")?`.
    pub fn real(&self, name: &str) -> Result<f64, Failure> {
        let value = self.value(name)?;
        if !value.is_real_value(false) {
            return Err(type_mismatch(name, "a real"));
        }
        value.real_value()
    }
    /// The value converted from the current Resource_Unicode format, cached per name
    /// (native `ExtValue`); `manager.ext_value("title")?`.
    ///
    /// Quirk preserved: the cache is never refreshed by a later byte-string `set_resource`.
    pub fn ext_value(&mut self, name: &str) -> Result<&ExtendedString, Failure> {
        let key = c_string(name);
        if self.extended.seek(&key).is_none() {
            let value = self.value(name)?.clone();
            let format = ResourceUnicode::get_format_with(&self.host)?;
            let mut converted = ExtendedString::default();
            ResourceUnicode::convert_format_to_unicode(format, value.as_bytes(), &mut converted)?;
            self.extended.bind(key.clone(), converted);
        }
        Ok(self.extended.seek(&key).expect("bound above"))
    }
    /// Creates or replaces a user value from a C string (bytes up to the first NUL);
    /// `manager.set_resource("key", b"value")`.
    pub fn set_resource(&mut self, name: &str, value: &[u8]) {
        let end = value
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(value.len());
        self.user
            .bind(c_string(name), AsciiString::from_raw(&value[..end]));
    }
    /// Stores the `%d` spelling of an integer; `manager.set_resource_integer("count", 3)`.
    pub fn set_resource_integer(&mut self, name: &str, value: i32) {
        self.set_resource(name, AsciiString::from_integer(value).as_bytes());
    }
    /// Stores the `%g` spelling of a real; `manager.set_resource_real("tolerance", 0.5)`.
    pub fn set_resource_real(&mut self, name: &str, value: f64) {
        self.set_resource(name, AsciiString::from_real(value).as_bytes());
    }
    /// Caches the UTF-16 value and stores its conversion to the current format (native
    /// `SetResource(const char*, const char16_t*)`); `manager.set_resource_ext("t", &[0x41, 0])?`.
    ///
    /// The value ends at its first zero unit. The conversion buffer holds `3 * length + 10`
    /// spaces; its result flag is ignored, as natively, and the stored text ends at the first
    /// NUL the conversion wrote. Conversions that raise (the code pages, whose loop reads past
    /// the source, and GBK/Big5) leave the cache updated, as the native exception does.
    pub fn set_resource_ext(&mut self, name: &str, value: &[u16]) -> Result<(), Failure> {
        let extended = ExtendedString::from_utf16(value);
        self.extended.bind(c_string(name), extended.clone());
        let mut buffer = vec![b' '; extended.len() * 3 + 10];
        let format = ResourceUnicode::get_format_with(&self.host)?;
        ResourceUnicode::convert_unicode_to_format(format, &extended, &mut buffer)?;
        self.set_resource(name, &buffer);
        Ok(())
    }
    /// The reference map (`true`) or the user map (native `GetMap`); `manager.get_map(true)`.
    pub fn get_map(&mut self, reference: bool) -> &mut DataMap<AsciiString, AsciiString> {
        if reference {
            &mut self.reference
        } else {
            &mut self.user
        }
    }
}
fn no_such_resource(name: &AsciiString) -> Failure {
    Failure::with_type(
        "Resource_NoSuchResource",
        &String::from_utf8_lossy(name.as_bytes()),
        "",
    )
}
fn type_mismatch(name: &str, expected: &str) -> Failure {
    let name = name.split('\0').next().unwrap_or_default();
    Failure::with_type(
        "Standard_TypeMismatch",
        &format!("Value of resource `{name}` is not {expected}"),
        "",
    )
}
