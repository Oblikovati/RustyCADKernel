// Rust port of DataExchange/TKDE/DE/DE_ConfigurationContext.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_ConfigurationContext`: the parameters of a resource file or string, read from a
//! scope (`scope.param`).
use super::configuration_line::{
    ConfigurationLineKind, make_name, read_resource_line, what_kind_of_line,
};
use rustycad_tkernel::{
    impl_transient,
    message::Message,
    ncollection::{DataMap, List},
    osd::{CadPath, File, OpenMode, OsdErrorAccess, Protection},
    standard::{Failure, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use std::fmt::Write;
use std::sync::RwLockReadGuard;

/// The native `myResource`: parameter name to value.
pub type DeResourceMap = DataMap<AsciiString, AsciiString>;

/// `DE_ConfigurationContext`: loads `key : value` resources and reads them back by scope.
///
/// ```
/// use rustycad_tkde::de::DeConfigurationContext;
/// let context = DeConfigurationContext::new();
/// assert!(context.load_str(&"ToV4.exec.op : 12\n".into()).unwrap());
/// assert_eq!(context.integer_val("exec.op", 0, "ToV4"), 12);
/// ```
#[derive(Default)]
pub struct DeConfigurationContext {
    transient: StandardTransient,
    resource: SharedState<DeResourceMap>,
}
impl_transient!(
    DeConfigurationContext,
    "DE_ConfigurationContext",
    StandardTransient,
    transient
);

impl DeConfigurationContext {
    /// `DE_ConfigurationContext()`: an empty context; `DeConfigurationContext::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Load`: reads `configuration` as a file when such a file exists, else as the resource
    /// text itself; `context.load(&"a : 1".into())?`.
    pub fn load(&self, configuration: &AsciiString) -> Result<bool, Failure> {
        let file: File = File::with_path(CadPath::new(configuration.as_bytes()));
        if !file.exists() {
            return self.load_str(configuration);
        }
        self.load_file(configuration)
    }

    /// `LoadFile`: replaces the parameters with the resources of a file. A file that cannot
    /// be opened reports a failure message and, as natively, returns `true`; a syntax error
    /// stops the load and returns `false`; `context.load_file(&"conf.txt".into())?`.
    pub fn load_file(&self, file_name: &AsciiString) -> Result<bool, Failure> {
        self.resource.write().clear(false);
        let mut file: File = File::with_path(CadPath::new(file_name.as_bytes()));
        file.open(OpenMode::ReadOnly, &Protection::default())?;
        if file.failed() {
            Message::send_fail_text(&AsciiString::new(
                "Error: DE Context loading is stopped. Can't open the file",
            ));
            return Ok(true);
        }
        let mut line: AsciiString = AsciiString::default();
        while read_resource_line(&mut file, &mut line)? {
            if !self.load_line(&line)? {
                report_syntax_error(&line);
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `LoadStr`: replaces the parameters with the resources of a text, one per line;
    /// `context.load_str(&"a : 1\nb : 2".into())?`.
    pub fn load_str(&self, resource: &AsciiString) -> Result<bool, Failure> {
        self.resource.write().clear(false);
        let mut line: AsciiString = AsciiString::default();
        let length: i32 = resource.length();
        for index in 1..=length {
            let byte: u8 = resource.value(index)?;
            if byte != b'\n' {
                line += byte;
            }
            if (byte == b'\n' || index == length) && !line.is_empty() {
                if !self.load_line(&line)? {
                    report_syntax_error(&line);
                    return Ok(false);
                }
                line.clear();
            }
        }
        Ok(true)
    }

    /// The protected `load`: binds the resource of one line; comments, empty lines and
    /// lines without a colon are skipped. `false` only for an empty line;
    /// `context.load_line(&"a : 1".into())?`.
    pub fn load_line(&self, resource_line: &AsciiString) -> Result<bool, Failure> {
        if resource_line.is_empty() {
            return Ok(false);
        }
        if let ConfigurationLineKind::Resource(key, value) = what_kind_of_line(resource_line)? {
            self.resource.write().bind(key, value);
        }
        Ok(true)
    }

    /// `IsParamSet`: tells whether `scope.param` is defined; `context.is_param_set("op", "ToV4")`.
    pub fn is_param_set(&self, param: impl AsRef<[u8]>, scope: impl AsRef<[u8]>) -> bool {
        let name: AsciiString = make_name(scope.as_ref(), param.as_ref());
        self.resource.read().is_bound(&name)
    }

    /// `GetString`: copies the value of `scope.param` into `value`; `false` when it is not
    /// defined (then `value` is unchanged); `context.get_string("op", &mut text, "ToV4")`.
    pub fn get_string(
        &self,
        param: impl AsRef<[u8]>,
        value: &mut AsciiString,
        scope: impl AsRef<[u8]>,
    ) -> bool {
        let name: AsciiString = make_name(scope.as_ref(), param.as_ref());
        let resource = self.resource.read();
        let Some(found) = resource.seek(&name) else {
            return false;
        };
        *value = found.clone();
        true
    }

    /// `GetReal`: the value when it starts with a number; `context.get_real("tol", &mut v, "")`.
    pub fn get_real(
        &self,
        param: impl AsRef<[u8]>,
        value: &mut f64,
        scope: impl AsRef<[u8]>,
    ) -> bool {
        let mut text: AsciiString = AsciiString::default();
        if !self.get_string(param, &mut text, scope) || !text.is_real_value(false) {
            return false;
        }
        let Ok(number) = text.real_value() else {
            return false;
        };
        *value = number;
        true
    }

    /// `GetInteger`: the value when it is an integer (`IsIntegerValue`);
    /// `context.get_integer("mode", &mut v, "")`.
    pub fn get_integer(
        &self,
        param: impl AsRef<[u8]>,
        value: &mut i32,
        scope: impl AsRef<[u8]>,
    ) -> bool {
        let mut text: AsciiString = AsciiString::default();
        if !self.get_string(param, &mut text, scope) || !text.is_integer_value() {
            return false;
        }
        let Ok(number) = text.integer_value() else {
            return false;
        };
        *value = number;
        true
    }

    /// `GetBoolean`: an integer value, `true` when it is not 0;
    /// `context.get_boolean("flag", &mut v, "")`.
    pub fn get_boolean(
        &self,
        param: impl AsRef<[u8]>,
        value: &mut bool,
        scope: impl AsRef<[u8]>,
    ) -> bool {
        let mut number: i32 = 0;
        if !self.get_integer(param, &mut number, scope) {
            return false;
        }
        *value = number != 0;
        true
    }

    /// `GetStringSeq`: the value split at each space, every element trimmed (two spaces give
    /// an empty element, as natively); `context.get_string_seq("STEP", &mut list, "global.priority")`.
    pub fn get_string_seq(
        &self,
        param: impl AsRef<[u8]>,
        value: &mut List<AsciiString>,
        scope: impl AsRef<[u8]>,
    ) -> bool {
        let mut text: AsciiString = AsciiString::default();
        if !self.get_string(param, &mut text, scope) {
            return false;
        }
        value.clear();
        split_at_spaces(&text, value);
        true
    }

    /// `RealVal`: [`Self::get_real`] or `default`; `context.real_val("tol", 1.0, "")`.
    pub fn real_val(&self, param: impl AsRef<[u8]>, default: f64, scope: impl AsRef<[u8]>) -> f64 {
        let mut value: f64 = 0.0;
        if self.get_real(param, &mut value, scope) {
            value
        } else {
            default
        }
    }

    /// `IntegerVal`: [`Self::get_integer`] or `default`; `context.integer_val("mode", 0, "")`.
    pub fn integer_val(
        &self,
        param: impl AsRef<[u8]>,
        default: i32,
        scope: impl AsRef<[u8]>,
    ) -> i32 {
        let mut value: i32 = 0;
        if self.get_integer(param, &mut value, scope) {
            value
        } else {
            default
        }
    }

    /// `BooleanVal`: [`Self::get_boolean`] or `default`; `context.boolean_val("flag", true, "")`.
    pub fn boolean_val(
        &self,
        param: impl AsRef<[u8]>,
        default: bool,
        scope: impl AsRef<[u8]>,
    ) -> bool {
        let mut value: bool = false;
        if self.get_boolean(param, &mut value, scope) {
            value
        } else {
            default
        }
    }

    /// `StringVal`: [`Self::get_string`] or `default`; `context.string_val("name", &"x".into(), "")`.
    pub fn string_val(
        &self,
        param: impl AsRef<[u8]>,
        default: &AsciiString,
        scope: impl AsRef<[u8]>,
    ) -> AsciiString {
        let mut value: AsciiString = AsciiString::default();
        if self.get_string(param, &mut value, scope) {
            value
        } else {
            default.clone()
        }
    }

    /// `GetInternalMap`: the parameters, read-locked for the guard's lifetime;
    /// `context.get_internal_map().extent()`.
    pub fn get_internal_map(&self) -> RwLockReadGuard<'_, DeResourceMap> {
        self.resource.read()
    }
}

/// The element loop of `GetStringSeq`.
fn split_at_spaces(text: &AsciiString, elements: &mut List<AsciiString>) {
    let bytes: &[u8] = text.as_bytes();
    let mut element: AsciiString = AsciiString::default();
    for (index, &byte) in bytes.iter().enumerate() {
        element += byte;
        if (byte == b' ' || index + 1 == bytes.len()) && !element.is_empty() {
            element.right_adjust();
            element.left_adjust();
            elements.append(std::mem::take(&mut element));
        }
    }
}

/// The failure message of a stopped load.
fn report_syntax_error(line: &AsciiString) {
    let mut stream = Message::send_fail();
    let _ = write!(
        stream,
        "Error: DE Context loading is stopped. Syntax error: {}",
        String::from_utf8_lossy(line.as_bytes())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(text: &str) -> DeConfigurationContext {
        let context = DeConfigurationContext::new();
        assert!(
            context
                .load_str(&AsciiString::new(text))
                .expect("a valid resource")
        );
        context
    }

    #[test]
    fn typed_getters_follow_the_native_number_rules() {
        let context = context("s.r : 1e-3\ns.i : 0x10\ns.f : 3.5\ns.w : yes\n");
        assert_eq!(context.real_val("r", 2.5, "s"), 0.001);
        assert_eq!(context.integer_val("r", 5, "s"), 1);
        assert_eq!(context.real_val("i", 2.5, "s"), 16.0);
        assert_eq!(context.integer_val("i", 5, "s"), 0);
        assert_eq!(context.integer_val("f", 5, "s"), 5);
        assert!(!context.boolean_val("i", true, "s"));
        assert_eq!(
            context.string_val("w", &AsciiString::new("d"), "s"),
            AsciiString::new("yes")
        );
        assert_eq!(
            context.string_val("x", &AsciiString::new("d"), "s"),
            AsciiString::new("d")
        );
    }

    #[test]
    fn get_string_seq_keeps_empty_elements_of_double_spaces() {
        let context = context("q : a b  c\n");
        let mut list: List<AsciiString> = List::default();
        assert!(context.get_string_seq("q", &mut list, ""));
        let items: Vec<&[u8]> = list.iter().map(AsciiString::as_bytes).collect();
        assert_eq!(items, [&b"a"[..], b"b", b"", b"c"]);
        assert!(!context.get_string_seq("none", &mut list, ""));
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn a_failed_getter_leaves_the_output_unchanged() {
        let context = context("t : text\n");
        let mut value: f64 = -1.0;
        assert!(!context.get_real("t", &mut value, ""));
        assert_eq!(value, -1.0);
        assert!(context.is_param_set("t", ""));
        assert!(!context.is_param_set("t", "scope"));
    }

    #[test]
    fn load_str_replaces_and_stops_on_a_blank_line() {
        let context = context("a : 1\n");
        let failure = context
            .load_str(&AsciiString::new("b : 2\n   \nc : 3\n"))
            .unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
        assert!(!context.is_param_set("a", ""));
        assert!(context.is_param_set("b", ""));
        assert!(
            !context
                .load_line(&AsciiString::default())
                .expect("no failure")
        );
    }

    #[test]
    fn load_file_of_a_missing_file_reports_success_as_natively() {
        let context = DeConfigurationContext::new();
        assert!(
            context
                .load_file(&AsciiString::new("/no/such/dir/conf.txt"))
                .expect("no failure")
        );
        assert!(
            context
                .load(&AsciiString::new("x : 1"))
                .expect("no failure")
        );
        assert!(context.is_param_set("x", ""));
    }
}
