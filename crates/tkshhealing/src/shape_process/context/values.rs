//! The parameter getters of `ShapeProcess_Context`.
use super::ShapeProcessContext;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;

/// Which native numeric reading a `&alias` resolves to.
#[derive(Clone, Copy)]
enum Number {
    Real,
    Integer,
}

impl ShapeProcessContext {
    /// `IsParamSet(param)`: a manager is loaded and holds the scoped parameter.
    /// Example: `context.is_param_set("Tolerance3d")`.
    pub fn is_param_set(&self, param: &str) -> bool {
        let Some(resource) = self.resource_manager() else {
            return false;
        };
        let name = super::key(&self.scoped_name(param));
        resource.read().find(&name)
    }

    /// `GetString(param, str)`: the scoped value, `None` when no manager is loaded or the
    /// parameter is not defined; `context.get_string("exec.op")`.
    pub fn get_string(&self, param: &str) -> Option<AsciiString> {
        let resource = self.resource_manager()?;
        let name = super::key(&self.scoped_name(param));
        let manager = resource.read();
        if !manager.find(&name) {
            return None;
        }
        manager.value(&name).ok().cloned()
    }

    /// `GetReal(param, val)`: the scoped value read as a real, or the real of the
    /// (unscoped) parameter named after a leading `&`; `None` for the native false.
    /// Native quirk kept: a blank value raises Standard_OutOfRange (`Value(1)` of the empty
    /// adjusted string); `context.get_real("Tolerance3d")?`.
    pub fn get_real(&self, param: &str) -> Result<Option<f64>, Failure> {
        self.number(param, Number::Real)
    }

    /// `GetInteger(param, val)`: as `get_real` for integers; `context.get_integer("Mode")?`.
    pub fn get_integer(&self, param: &str) -> Result<Option<i32>, Failure> {
        Ok(self
            .number(param, Number::Integer)?
            .map(|value| value as i32))
    }

    // Native reads the value as a number, else follows one `&name` alias.
    fn number(&self, param: &str, kind: Number) -> Result<Option<f64>, Failure> {
        let Some(mut text) = self.get_string(param) else {
            return Ok(None);
        };
        if let Some(value) = read_number(&text, kind) {
            return Ok(Some(value));
        }
        text.left_adjust();
        if text.value(1)? != b'&' {
            return Ok(None);
        }
        let mut reference = text.split(1)?;
        reference.left_adjust();
        reference.right_adjust();
        let Some(resource) = self.resource_manager() else {
            return Ok(None);
        };
        let manager = resource.read();
        let name = super::key(&reference);
        if !manager.find(&name) {
            return Ok(None);
        }
        Ok(read_number(manager.value(&name)?, kind))
    }

    /// `GetBoolean(param, val)`: the scoped integer of the manager is nonzero; `None` when
    /// it raises (missing or not an integer); `context.get_boolean("FixShape")`.
    pub fn get_boolean(&self, param: &str) -> Option<bool> {
        let resource = self.resource_manager()?;
        let name = super::key(&self.scoped_name(param));
        let value = resource.read().integer(&name).ok()?;
        Some(value != 0)
    }

    /// `RealVal(param, def)`; `context.real_val("Tolerance3d", 1e-7)?`.
    pub fn real_val(&self, param: &str, default: f64) -> Result<f64, Failure> {
        Ok(self.get_real(param)?.unwrap_or(default))
    }

    /// `IntegerVal(param, def)`; `context.integer_val("Mode", -1)?`.
    pub fn integer_val(&self, param: &str, default: i32) -> Result<i32, Failure> {
        Ok(self.get_integer(param)?.unwrap_or(default))
    }

    /// `BooleanVal(param, def)`; `context.boolean_val("FixShape", true)`.
    pub fn boolean_val(&self, param: &str, default: bool) -> bool {
        self.get_boolean(param).unwrap_or(default)
    }

    /// `StringVal(param, def)`: the scoped value of the manager, `default` when no manager
    /// is loaded or the value raises; `context.string_val("exec.op", "")`.
    pub fn string_val(&self, param: &str, default: &str) -> AsciiString {
        let Some(resource) = self.resource_manager() else {
            return AsciiString::from(default);
        };
        let name = super::key(&self.scoped_name(param));
        let manager = resource.read();
        manager
            .value(&name)
            .cloned()
            .unwrap_or_else(|_| AsciiString::from(default))
    }
}

fn read_number(text: &AsciiString, kind: Number) -> Option<f64> {
    match kind {
        Number::Real => text
            .is_real_value(false)
            .then(|| text.real_value().ok())
            .flatten(),
        Number::Integer => text
            .is_integer_value()
            .then(|| text.integer_value().ok().map(f64::from))
            .flatten(),
    }
}
