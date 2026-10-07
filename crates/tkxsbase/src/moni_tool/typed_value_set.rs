// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_TypedValue.cxx (Interpret,
// Satisfies and the value setters) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks and setters of the value of a `MoniTool_TypedValue`.
use super::typed_value::TypedValueState;
use super::{MoniToolTypedValue, MoniToolValueType};
use crate::c_text::atoi;
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::HAsciiString,
};

impl MoniToolTypedValue {
    /// `Interpret(hval, native)`: the interpret function when set; for an enumeration,
    /// the text (native) or the case number (coded) of a known value; else `hval` itself.
    pub fn interpret(&self, hval: &Handle<HAsciiString>, native: bool) -> Handle<HAsciiString> {
        if hval.is_null() {
            return hval.clone();
        }
        // The function reads this value: it runs without the lock held.
        let interp = self.state.read().theinterp;
        if let Some(func) = interp {
            return func(self, hval, native);
        }
        let state = self.state.read();
        if state.thetype != MoniToolValueType::ValueEnum {
            return hval.clone();
        }
        let bytes = hval.string().as_bytes().to_vec();
        let encas = state.enum_case(&bytes);
        if encas < state.theintlow {
            return hval.clone();
        }
        if native {
            return Handle::new(HAsciiString::from_string(&state.enum_val(encas)));
        }
        Handle::new(HAsciiString::from_integer(encas))
    }
    /// `Satisfies(val)`: false for a null value or when the satisfies function refuses
    /// it; an empty value satisfies; else the integer or real limits, the enumeration
    /// match or the text maximum length decide.
    pub fn satisfies(&self, val: &Handle<HAsciiString>) -> bool {
        let Some(text) = val.get() else { return false };
        // The function may read typed values: it runs without the lock held.
        let satisf = self.state.read().thesatisf;
        if satisf.is_some_and(|func| !func(val)) {
            return false;
        }
        if text.length() == 0 {
            return true;
        }
        self.state.read().satisfies_text(text)
    }
    /// `SetCStringValue(val)`: true when unchanged or set; false when it does not satisfy
    /// the specification or names no enumeration case. The shared string is edited in
    /// place; after `ClearValue` it is null, and the native call raises
    /// `Standard_NullObject` (from `IsSameString`).
    pub fn set_c_string_value(&self, val: &str) -> Result<bool, Failure> {
        let hval = Handle::new(HAsciiString::new(val));
        let current = self.live_string("SetCStringValue")?;
        if hval.is_same_string(&current)? {
            return Ok(true);
        }
        if !self.satisfies(&hval) {
            return Ok(false);
        }
        let mut state = self.state.write();
        match state.thetype {
            MoniToolValueType::ValueInteger => state.theival = atoi(val.as_bytes()),
            MoniToolValueType::ValueEnum => {
                let ival = state.enum_case(val.as_bytes());
                let cval = state.enum_val(ival);
                if cval.is_empty() {
                    return Ok(false);
                }
                state.theival = ival;
                drop(state);
                return Ok(replace_text(&current, cval.as_bytes()));
            }
            _ => {}
        }
        Ok(replace_text(&current, val.as_bytes()))
    }
    /// `SetHStringValue(hval)`: shares `hval` as the new string when it satisfies the
    /// specification; false for a null string.
    pub fn set_h_string_value(&self, hval: &Handle<HAsciiString>) -> bool {
        if hval.is_null() || !self.satisfies(hval) {
            return false;
        }
        let mut state = self.state.write();
        state.thehval = hval.clone();
        let bytes = hval.string().as_bytes().to_vec();
        match state.thetype {
            MoniToolValueType::ValueInteger => state.theival = atoi(&bytes),
            MoniToolValueType::ValueEnum => state.theival = state.enum_case(&bytes),
            _ => {}
        }
        true
    }
    /// `SetIntegerValue(ival)`: for an enumeration the text of the case is stored;
    /// `Standard_NullObject` after `ClearValue`, as natively.
    pub fn set_integer_value(&self, ival: i32) -> Result<bool, Failure> {
        let hval = Handle::new(HAsciiString::from_integer(ival));
        let current = self.live_string("SetIntegerValue")?;
        if hval.is_same_string(&current)? {
            return Ok(true);
        }
        if !self.satisfies(&hval) {
            return Ok(false);
        }
        let mut state = self.state.write();
        let text = match state.thetype {
            MoniToolValueType::ValueEnum => state.enum_val(ival),
            _ => hval.string().clone(),
        };
        state.theival = ival;
        drop(state);
        replace_text(&current, text.as_bytes());
        Ok(true)
    }
    /// `SetRealValue(rval)`: stores the text of the real (the integer value is kept);
    /// `Standard_NullObject` after `ClearValue`, as natively.
    pub fn set_real_value(&self, rval: f64) -> Result<bool, Failure> {
        let hval = Handle::new(HAsciiString::from_real(rval));
        let current = self.live_string("SetRealValue")?;
        if hval.is_same_string(&current)? {
            return Ok(true);
        }
        if !self.satisfies(&hval) {
            return Ok(false);
        }
        Ok(replace_text(&current, hval.string().as_bytes()))
    }
    /// The shared string, or the `Standard_NullObject` its native dereference raises.
    fn live_string(&self, member: &str) -> Result<Handle<HAsciiString>, Failure> {
        let current = self.h_string_value();
        if current.is_null() {
            return Err(exceptions::null_object(&format!(
                "MoniTool_TypedValue::{member}: the value string of {:?} is null after ClearValue; expected a set string",
                self.name()
            )));
        }
        Ok(current)
    }
}

/// `Clear` then `AssignCat` on the shared string; always true (the native return).
fn replace_text(current: &Handle<HAsciiString>, text: &[u8]) -> bool {
    current.clear();
    current.assign_cat(text);
    true
}

impl TypedValueState {
    /// The type-specific part of `Satisfies` for a non-empty value.
    pub(super) fn satisfies_text(&self, val: &HAsciiString) -> bool {
        match self.thetype {
            MoniToolValueType::ValueInteger => self.satisfies_integer(val),
            MoniToolValueType::ValueReal => self.satisfies_real(val),
            MoniToolValueType::ValueEnum => {
                let match_required = (self.thelims & 4) != 0;
                !match_required || self.enum_case(val.string().as_bytes()) >= self.theintlow
            }
            MoniToolValueType::ValueText => !(self.themaxlen > 0 && val.length() > self.themaxlen),
            _ => true,
        }
    }
    fn satisfies_integer(&self, val: &HAsciiString) -> bool {
        if !val.is_integer_value() {
            return false;
        }
        let ival = atoi(val.string().as_bytes());
        let (has_low, low) = self.integer_limit(false);
        let (has_up, up) = self.integer_limit(true);
        !(has_low && low > ival) && !(has_up && up < ival)
    }
    fn satisfies_real(&self, val: &HAsciiString) -> bool {
        if !val.is_real_value() {
            return false;
        }
        let rval = val.real_value().unwrap_or(0.0);
        let (has_low, low) = self.real_limit(false);
        let (has_up, up) = self.real_limit(true);
        !(has_low && low > rval) && !(has_up && up < rval)
    }
}
