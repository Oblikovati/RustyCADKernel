// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_TypedValue.cxx (copy, definition,
// limits and enumerations) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The specification of a `MoniTool_TypedValue`: definition text, `AddDef` commands,
//! integer and real limits, enumerations, and the copy constructor.
use super::typed_value::{EnumTexts, TypedValueState, not_kind};
use super::{MoniToolTypedValue, MoniToolValueType};
use crate::c_text::atoi;
use rustycad_tkernel::{
    standard::{Failure, Handle, c_string::atof_bytes, exceptions},
    tcollection::{AsciiString, HAsciiString},
};

impl MoniToolTypedValue {
    /// `MoniTool_TypedValue(other)`: a duplicate with its own string. As natively, the
    /// definition is the computed one of `other` (now enforced), the object type is
    /// `ObjectType()` (`Standard_Transient` when not set), and the limits are read with
    /// `IntegerLimit` and `RealLimit`, which also give the natural limits
    /// (`IntegerFirst`, `RealLast`...) when not set.
    pub fn copied(other: &MoniToolTypedValue) -> Self {
        let mut state = other.state.read().clone();
        let source = state.clone();
        state.thedef = source.definition();
        state.theotyp = other.object_type();
        state.thelims = 0;
        state.theenums = None;
        state.copy_limits(&source);
        state.copy_enums(&source);
        if state.thehval.is_some() {
            let text = source.thehval.string().clone();
            state.thehval = Handle::new(HAsciiString::from_string(&text));
        }
        Self::from_state(state)
    }
    /// `Definition`: the enforced definition, else one computed from the specification.
    pub fn definition(&self) -> AsciiString {
        self.state.read().definition()
    }
    /// `SetDefinition`.
    pub fn set_definition(&self, deftext: &str) {
        self.state.write().thedef = AsciiString::new(deftext);
    }
    /// `AddDef(initext)`: interprets "imin i", "imax i", "rmin r", "rmax r", "unit u",
    /// "ematch i", "enum i", "eval text", "tmax l" (the argument follows the last
    /// space); false for a text it does not recognise. The setters it calls raise for a
    /// value of another type.
    pub fn add_def(&self, init: &str) -> Result<bool, Failure> {
        let bytes = init.as_bytes();
        let Some(space) = bytes.iter().rposition(|b| *b == b' ') else {
            return Ok(false);
        };
        let arg = &init[space + 1..];
        let at = |i: usize| bytes.get(i).copied().unwrap_or(0);
        match (at(0), at(1), at(2)) {
            (b'i', _, b'i') => self.set_integer_limit(false, atoi(arg.as_bytes()))?,
            (b'i', _, b'a') => self.set_integer_limit(true, atoi(arg.as_bytes()))?,
            (b'r', _, b'i') => self.set_real_limit(false, atof_bytes(arg.as_bytes()))?,
            (b'r', _, b'a') => self.set_real_limit(true, atof_bytes(arg.as_bytes()))?,
            (b'u', _, _) => self.set_unit_def(arg),
            (b'e', b'm', _) => self.start_enum(atoi(arg.as_bytes()), true)?,
            (b'e', b'n', _) => self.start_enum(atoi(arg.as_bytes()), false)?,
            (b'e', b'v', _) => self.add_enum(&[arg])?,
            (b't', b'm', _) => self.set_max_length(atoi(arg.as_bytes())),
            _ => return Ok(false),
        }
        Ok(true)
    }
    /// `SetIntegerLimit(max, val)`: `Standard_ConstructionError` unless an integer.
    pub fn set_integer_limit(&self, max: bool, val: i32) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.thetype != MoniToolValueType::ValueInteger {
            return Err(not_kind("SetIntegerLimit, not an Integer", state.thetype));
        }
        state.thelims |= if max { 2 } else { 1 };
        *(if max {
            &mut state.theintup
        } else {
            &mut state.theintlow
        }) = val;
        Ok(())
    }
    /// `IntegerLimit(max, val)`: whether the limit is set, and the limit or the natural
    /// one (`IntegerLast`, `IntegerFirst`).
    pub fn integer_limit(&self, max: bool) -> (bool, i32) {
        self.state.read().integer_limit(max)
    }
    /// `SetRealLimit(max, val)`: `Standard_ConstructionError` unless a real.
    pub fn set_real_limit(&self, max: bool, val: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.thetype != MoniToolValueType::ValueReal {
            return Err(not_kind("SetRealLimit, not a Real", state.thetype));
        }
        state.thelims |= if max { 2 } else { 1 };
        *(if max {
            &mut state.therealup
        } else {
            &mut state.therealow
        }) = val;
        Ok(())
    }
    /// `RealLimit(max, val)`: whether the limit is set, and the limit or the natural one
    /// (`RealLast`, `RealFirst`).
    pub fn real_limit(&self, max: bool) -> (bool, f64) {
        self.state.read().real_limit(max)
    }
    /// `StartEnum(start = 0, match = true)`: an empty enumeration from `start`;
    /// `Standard_ConstructionError` unless an enumeration.
    pub fn start_enum(&self, start: i32, match_required: bool) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.thetype != MoniToolValueType::ValueEnum {
            return Err(not_kind("StartEnum, Not an Enum", state.thetype));
        }
        state.thelims |= 4;
        if !match_required {
            state.thelims -= 4;
        }
        state.theintlow = start;
        state.theintup = start - 1;
        Ok(())
    }
    /// `AddEnum(v1, ..., v10)`: appends the non-empty texts as the next cases; the
    /// native call takes at most 10 texts, so longer slices are added 10 at a time.
    pub fn add_enum(&self, values: &[&str]) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.thetype != MoniToolValueType::ValueEnum {
            return Err(not_kind("AddEnum, Not an Enum", state.thetype));
        }
        for chunk in values.chunks(10) {
            state.grow_enums_for_ten();
            for text in chunk.iter().filter(|t| !t.is_empty()) {
                state.theintup += 1;
                let case = state.theintup;
                state.store_enum(case, text);
            }
        }
        Ok(())
    }
    /// `AddEnumValue(val, num)`: records `val` for case `num` (main text when the case
    /// has none, else an alternate text); `Standard_ConstructionError` unless an
    /// enumeration or when `num` is below the start.
    pub fn add_enum_value(&self, val: &str, num: i32) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.thetype != MoniToolValueType::ValueEnum {
            return Err(not_kind("AddEnum, Not an Enum", state.thetype));
        }
        if num < state.theintlow {
            return Err(exceptions::construction_error(&format!(
                "MoniTool_TypedValue : AddEnum, out of range: {num}; expected at least {}",
                state.theintlow
            )));
        }
        if !val.is_empty() {
            state.add_enum_value(val, num);
        }
        Ok(())
    }
    /// `EnumDef`: `Some((start, end, match))` for an enumeration, else `None`.
    pub fn enum_def(&self) -> Option<(i32, i32, bool)> {
        self.state.read().enum_def()
    }
    /// `EnumVal(num)`: the text of a case, empty out of range or for another type.
    pub fn enum_val(&self, num: i32) -> AsciiString {
        self.state.read().enum_val(num)
    }
    /// `EnumCase(val)`: the case of a main or additional text, else the integer the
    /// text spells; `StartEnum - 1` when not recognised or for another type.
    pub fn enum_case(&self, val: &str) -> i32 {
        self.state.read().enum_case(val.as_bytes())
    }
}

impl TypedValueState {
    pub(super) fn integer_limit(&self, max: bool) -> (bool, i32) {
        if max {
            let res = (self.thelims & 2) != 0;
            return (res, if res { self.theintup } else { i32::MAX });
        }
        let res = (self.thelims & 1) != 0;
        (res, if res { self.theintlow } else { i32::MIN })
    }
    pub(super) fn real_limit(&self, max: bool) -> (bool, f64) {
        if max {
            let res = (self.thelims & 2) != 0;
            return (res, if res { self.therealup } else { f64::MAX });
        }
        let res = (self.thelims & 1) != 0;
        (res, if res { self.therealow } else { -f64::MAX })
    }
    pub(super) fn enum_def(&self) -> Option<(i32, i32, bool)> {
        if self.thetype != MoniToolValueType::ValueEnum {
            return None;
        }
        Some((self.theintlow, self.theintup, (self.thelims & 4) != 0))
    }
    pub(super) fn enum_val(&self, num: i32) -> AsciiString {
        if self.thetype != MoniToolValueType::ValueEnum
            || num < self.theintlow
            || num > self.theintup
        {
            return AsciiString::default();
        }
        self.theenums
            .as_ref()
            .map(|e| e.value(num))
            .unwrap_or_default()
    }
    pub(super) fn enum_case(&self, val: &[u8]) -> i32 {
        if self.thetype != MoniToolValueType::ValueEnum {
            return self.theintlow - 1;
        }
        let enums = self.theenums.as_ref();
        let main = (self.theintlow..=self.theintup)
            .find(|i| enums.is_some_and(|e| e.value(*i).is_equal(val)));
        if let Some(case) = main {
            return case;
        }
        if let Some(case) = self.theeadds.seek(&AsciiString::from_bytes(val)) {
            return *case;
        }
        // gka S4054: an integer text (spaces and minus signs allowed) is its own case.
        if val
            .iter()
            .any(|c| *c != b' ' && *c != b'-' && !c.is_ascii_digit())
        {
            return self.theintlow - 1;
        }
        atoi(val)
    }
    /// The array growth at the start of `AddEnum`: room for 10 more cases.
    fn grow_enums_for_ten(&mut self) {
        match &self.theenums {
            None => self.theenums = Some(EnumTexts::new(self.theintlow, self.theintlow + 10)),
            Some(enums) if enums.upper() < self.theintup + 10 => {
                self.regrow_enums(self.theintup + 10)
            }
            Some(_) => {}
        }
    }
    /// A new array `theintlow..=upper` with the cases `theintlow..=theintup` copied.
    fn regrow_enums(&mut self, upper: i32) {
        let mut grown = EnumTexts::new(self.theintlow, upper);
        if let Some(old) = &self.theenums {
            (self.theintlow..=self.theintup).for_each(|i| grown.set_value(i, old.value(i)));
        }
        self.theenums = Some(grown);
    }
    fn store_enum(&mut self, case: i32, text: &str) {
        if let Some(enums) = &mut self.theenums {
            enums.set_value(case, AsciiString::new(text));
        }
        self.theeadds.bind(AsciiString::new(text), case);
    }
    fn add_enum_value(&mut self, val: &str, num: i32) {
        match &self.theenums {
            None => self.theenums = Some(EnumTexts::new(self.theintlow, num + 1)),
            Some(enums) if enums.upper() < num => self.regrow_enums(num + 1),
            Some(_) => {}
        }
        self.theintup = self.theintup.max(num);
        if let Some(enums) = &mut self.theenums
            && enums.value(num).length() == 0
        {
            enums.set_value(num, AsciiString::new(val));
        }
        // The text also goes to the dictionary, as natively.
        self.theeadds.bind(AsciiString::new(val), num);
    }
    /// The limit part of the copy constructor: both getters write their natural limit
    /// when not set, as the native calls do.
    fn copy_limits(&mut self, source: &TypedValueState) {
        let limits = [source.integer_limit(false), source.integer_limit(true)];
        let reals = [source.real_limit(false), source.real_limit(true)];
        (self.theintlow, self.theintup) = (limits[0].1, limits[1].1);
        (self.therealow, self.therealup) = (reals[0].1, reals[1].1);
        for (index, set) in [limits[0].0, limits[1].0, reals[0].0, reals[1].0]
            .into_iter()
            .enumerate()
        {
            if set {
                self.thelims |= if index % 2 == 0 { 1 } else { 2 };
            }
        }
    }
    /// The enumeration part of the copy constructor.
    fn copy_enums(&mut self, source: &TypedValueState) {
        let Some((start, end, match_required)) = source.enum_def() else {
            return;
        };
        self.theintlow = start;
        self.theintup = end;
        if match_required {
            self.thelims |= 4;
        }
        if end >= start {
            let mut enums = EnumTexts::new(start, end);
            (start..=end).for_each(|i| enums.set_value(i, source.enum_val(i)));
            self.theenums = Some(enums);
        }
    }
}
