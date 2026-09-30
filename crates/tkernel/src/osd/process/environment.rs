//! `OSD_Environment`: one process environment variable, read and written by name.
//!
//! The Unix build validates names and values (printable ASCII, no `$` in a name or in a
//! value given to `SetValue`) and raises `Standard_ConstructionError`; the Windows build
//! validates nothing. Unix `Remove` sets the variable to the empty string (it stays
//! defined); Windows `Remove` deletes it.
use crate::osd::{OsdError, OsdErrorAccess, WhoAmI};
use crate::{standard::Failure, tcollection::AsciiString};

mod native;
pub use native::NativeEnvironment;

/// The process environment behind `OSD_Environment`; implement for deterministic tests.
pub trait EnvironmentSource {
    /// Reads a variable (`getenv`); `None` when it is not defined; `source.get(b"HOME")`.
    fn get(&self, name: &[u8]) -> Option<Vec<u8>>;
    /// Defines `name=value` and checks it reads back, or returns the native error number;
    /// `source.put(b"NAME", b"value")`. On Windows an empty value deletes the variable.
    fn put(&self, name: &[u8], value: &[u8]) -> Result<(), i32>;
}

/// `OSD_Environment`; `Environment::with_value("NAME", "value")?.build()`.
pub struct Environment<S: EnvironmentSource = NativeEnvironment> {
    source: S,
    name: AsciiString,
    value: AsciiString,
    error: OsdError,
}
impl Default for Environment<NativeEnvironment> {
    fn default() -> Self {
        Self::new()
    }
}
impl Environment<NativeEnvironment> {
    /// `OSD_Environment()`: no name and no value; `Environment::new()`.
    pub fn new() -> Self {
        Self::bare(NativeEnvironment)
    }
    /// `OSD_Environment(Name)`; `Environment::named("HOME")?`.
    pub fn named(name: &str) -> Result<Self, Failure> {
        Self::named_in(NativeEnvironment, name)
    }
    /// `OSD_Environment(Name, Value)`; `Environment::with_value("NAME", "value")?`.
    pub fn with_value(name: &str, value: &str) -> Result<Self, Failure> {
        Self::with_value_in(NativeEnvironment, name, value)
    }
}
impl<S: EnvironmentSource> Environment<S> {
    /// The default constructor over an injected environment; `Environment::bare(fake)`.
    pub fn bare(source: S) -> Self {
        Self {
            source,
            name: AsciiString::default(),
            value: AsciiString::default(),
            error: OsdError::new(),
        }
    }
    /// `OSD_Environment(Name)` over an injected environment; `Environment::named_in(fake, "A")?`.
    ///
    /// Unix: a name that is not printable ASCII or holds `$` raises
    /// `Standard_ConstructionError`.
    pub fn named_in(source: S, name: &str) -> Result<Self, Failure> {
        check_name(name, "OSD_Environment::OSD_Environment: bad argument")?;
        let mut environment: Self = Self::bare(source);
        environment.name = AsciiString::from(name);
        Ok(environment)
    }
    /// `OSD_Environment(Name, Value)` over an injected environment;
    /// `Environment::with_value_in(fake, "A", "$B")?` (a value may hold `$`).
    pub fn with_value_in(source: S, name: &str, value: &str) -> Result<Self, Failure> {
        let message: &str = "OSD_Environment::OSD_Environment: bad argument";
        check_name(name, message)?;
        check_text(value, false, message)?;
        let mut environment: Self = Self::bare(source);
        environment.name = AsciiString::from(name);
        environment.value = AsciiString::from(value);
        Ok(environment)
    }
    /// `OSD_Environment::SetValue`: the value `build` will define; `env.set_value("v")?`.
    ///
    /// Unix refuses a value that is not printable ASCII or holds `$` (unlike the
    /// constructor) with the native "OSD_Environment::Change: bad argument".
    pub fn set_value(&mut self, value: &str) -> Result<(), Failure> {
        check_text(value, true, "OSD_Environment::Change: bad argument")?;
        self.value = AsciiString::from(value);
        Ok(())
    }
    /// `OSD_Environment::Value`: reads the variable and keeps the result as the stored
    /// value; an undefined variable reads as empty; `env.value()`.
    ///
    /// Windows additionally records ERROR_ENVVAR_NOT_FOUND (203) for an undefined
    /// variable and resets the error after a successful read.
    pub fn value(&mut self) -> AsciiString {
        let found: Option<Vec<u8>> = self.source.get(self.name.as_bytes());
        self.value = found
            .as_deref()
            .map_or_else(AsciiString::default, AsciiString::from_raw);
        if cfg!(windows) {
            self.record_windows_lookup(found.is_some());
        }
        self.value.clone()
    }
    /// `OSD_Environment::SetName`: Unix resets the error first, then refuses a bad name
    /// like the constructor; `env.set_name("OTHER")?`.
    pub fn set_name(&mut self, name: &str) -> Result<(), Failure> {
        if !cfg!(windows) {
            self.error.reset();
        }
        check_name(name, "OSD_Environment::SetName: bad argument")?;
        self.name = AsciiString::from(name);
        Ok(())
    }
    /// `OSD_Environment::Name`; `env.name()`.
    pub fn name(&self) -> AsciiString {
        self.name.clone()
    }
    /// `OSD_Environment::Build`: defines the variable with the stored value; Unix records
    /// "Set Environment" when it does not read back; `env.build()`.
    pub fn build(&mut self) {
        let status = self.source.put(self.name.as_bytes(), self.value.as_bytes());
        if let Err(errno) = status {
            self.error
                .set_value(errno, WhoAmI::Environment as i32, "Set Environment");
        }
    }
    /// `OSD_Environment::Remove`: Unix clears the stored value and builds (the variable
    /// stays defined as empty); Windows deletes the variable and keeps the stored value;
    /// `env.remove()`.
    pub fn remove(&mut self) {
        if cfg!(windows) {
            let _ = self.source.put(self.name.as_bytes(), b"");
            return;
        }
        self.value = AsciiString::default();
        self.build();
    }
    fn record_windows_lookup(&mut self, found: bool) {
        if found {
            self.error.reset();
            return;
        }
        const ENVVAR_NOT_FOUND: i32 = 203;
        let text: String = system_message(ENVVAR_NOT_FOUND);
        self.error
            .set_value(ENVVAR_NOT_FOUND, WhoAmI::Environment as i32, text.as_str());
    }
}
impl<S: EnvironmentSource> OsdErrorAccess for Environment<S> {
    fn osd_error(&self) -> &OsdError {
        &self.error
    }
    fn osd_error_mut(&mut self) -> &mut OsdError {
        &mut self.error
    }
}

/// The host's text for an OS error number, without the standard library's
/// " (os error N)" suffix (Windows: the FormatMessageW text native code stores).
fn system_message(code: i32) -> String {
    let text: String = std::io::Error::from_raw_os_error(code).to_string();
    let suffix: String = format!(" (os error {code})");
    text.strip_suffix(&suffix).unwrap_or(&text).to_string()
}

fn check_name(name: &str, message: &str) -> Result<(), Failure> {
    check_text(name, true, message)
}

/// Unix `IsAscii()` (every byte in 32..=126) and, when asked, no `$`; Windows accepts all.
fn check_text(text: &str, refuse_dollar: bool, message: &str) -> Result<(), Failure> {
    if cfg!(windows) {
        return Ok(());
    }
    let printable: bool = text.bytes().all(|byte| (b' '..=b'~').contains(&byte));
    if printable && !(refuse_dollar && text.contains('$')) {
        return Ok(());
    }
    Err(Failure::with_type(
        "Standard_ConstructionError",
        message,
        "",
    ))
}
