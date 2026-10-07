// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_TypedValue (.hxx, .cxx),
// MoniTool_ValueInterpret.hxx and MoniTool_ValueSatisfies.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MoniTool_TypedValue`: a value with an alphanumeric expression and a specification
//! (integer with limits, real with limits, enumeration, text with a maximum length, or
//! object of a type). The definition and enumeration members are in `typed_value_def`,
//! the print members in `typed_value_print`, the library of types in `typed_value_lib`.
//!
//! No class of OCCT overrides the virtual members (`Print`, `HasInterpret`, `Interpret`,
//! `Satisfies`, the `Set*Value` setters), so they are inherent methods; the derived
//! `Interface_TypedValue` embeds this struct (xsbase-design.md section 1).
//!
//! The native readers dereference the shared string without a check, and it is null
//! after `ClearValue`; the port reads a null string as empty (`IsSetValue`, `RealValue`,
//! `CStringValue`), while the setters that edit it in place return `Standard_NullObject`.
use super::MoniToolValueType;
use rustycad_tkernel::{
    impl_transient,
    ncollection::DataMap,
    standard::{
        Failure, Handle, RuntimeType, RuntimeTyped, SharedState, StandardTransient, exceptions,
    },
    tcollection::{AsciiString, HAsciiString},
};

/// `MoniTool_ValueInterpret`: interprets a value (`native`: to the native form, else to
/// the coded form) for the typed value given as first argument.
pub type MoniToolValueInterpret =
    fn(&MoniToolTypedValue, &Handle<HAsciiString>, bool) -> Handle<HAsciiString>;
/// `MoniTool_ValueSatisfies`: an additional check of a value.
pub type MoniToolValueSatisfies = fn(&Handle<HAsciiString>) -> bool;

/// The bounds and texts of an enumeration (natively an `NCollection_HArray1` of strings).
#[derive(Clone, Debug, Default)]
pub(super) struct EnumTexts {
    pub(super) lower: i32,
    pub(super) values: Vec<AsciiString>,
}
impl EnumTexts {
    pub(super) fn new(lower: i32, upper: i32) -> Self {
        let len = usize::try_from(upper - lower + 1).unwrap_or(0);
        Self {
            lower,
            values: vec![AsciiString::default(); len],
        }
    }
    pub(super) fn upper(&self) -> i32 {
        self.lower + self.values.len() as i32 - 1
    }
    /// `Value(num)`: empty out of the bounds (unchecked natively).
    pub(super) fn value(&self, num: i32) -> AsciiString {
        let index = usize::try_from(num - self.lower).ok();
        index
            .and_then(|i| self.values.get(i))
            .cloned()
            .unwrap_or_default()
    }
    pub(super) fn set_value(&mut self, num: i32, text: AsciiString) {
        let index = usize::try_from(num - self.lower).ok();
        if let Some(slot) = index.and_then(|i| self.values.get_mut(i)) {
            *slot = text;
        }
    }
}

/// The fields of a typed value, in the native order.
#[derive(Clone)]
pub(super) struct TypedValueState {
    pub(super) thename: AsciiString,
    pub(super) thedef: AsciiString,
    pub(super) thelabel: AsciiString,
    pub(super) thetype: MoniToolValueType,
    pub(super) theotyp: Handle<RuntimeType>,
    pub(super) thelims: i32,
    pub(super) themaxlen: i32,
    pub(super) theintlow: i32,
    pub(super) theintup: i32,
    pub(super) therealow: f64,
    pub(super) therealup: f64,
    pub(super) theunidef: AsciiString,
    pub(super) theenums: Option<EnumTexts>,
    pub(super) theeadds: DataMap<AsciiString, i32>,
    pub(super) theinterp: Option<MoniToolValueInterpret>,
    pub(super) thesatisf: Option<MoniToolValueSatisfies>,
    pub(super) thesatisn: AsciiString,
    pub(super) theival: i32,
    pub(super) thehval: Handle<HAsciiString>,
    pub(super) theoval: Handle<StandardTransient>,
}

/// `MoniTool_TypedValue`.
///
/// ```
/// use rustycad_tkxsbase::moni_tool::{MoniToolTypedValue, MoniToolValueType};
/// let value = MoniToolTypedValue::new("count", MoniToolValueType::ValueInteger, "").unwrap();
/// value.set_integer_limit(false, 0).unwrap();
/// assert!(value.set_c_string_value("12").unwrap());
/// assert!(!value.set_c_string_value("-3").unwrap());
/// assert_eq!(value.integer_value(), 12);
/// assert_eq!(value.definition().as_bytes(), b"Integer  >= 0");
/// ```
pub struct MoniToolTypedValue {
    transient: StandardTransient,
    pub(super) state: SharedState<TypedValueState>,
}
impl_transient!(
    MoniToolTypedValue,
    "MoniTool_TypedValue",
    StandardTransient,
    transient
);

/// The types a typed value accepts (the constructor raises for the others).
fn supported_type(kind: MoniToolValueType) -> bool {
    use MoniToolValueType::{ValueEnum, ValueIdent, ValueInteger, ValueReal, ValueText};
    matches!(
        kind,
        ValueInteger | ValueReal | ValueEnum | ValueText | ValueIdent
    )
}

impl MoniToolTypedValue {
    /// `MoniTool_TypedValue(name, type = Text, init = "")`: `Standard_ConstructionError`
    /// for a type other than integer, real, enum, text or ident; `init` is set when it
    /// satisfies the specification.
    pub fn new(name: &str, kind: MoniToolValueType, init: &str) -> Result<Self, Failure> {
        if !supported_type(kind) {
            return Err(exceptions::construction_error(&format!(
                "MoniTool_TypedValue : Type not supported: {kind:?}; expected Integer, Real, Enum, Text or Ident"
            )));
        }
        let created = Self::from_state(TypedValueState::fresh(name, kind));
        if !init.is_empty() && created.satisfies(&Handle::new(HAsciiString::new(init))) {
            created.set_c_string_value(init)?;
        }
        Ok(created)
    }
    pub(super) fn from_state(state: TypedValueState) -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `Internals`: the interpret and satisfies functions, the satisfies name and a copy
    /// of the additional enumeration texts.
    pub fn internals(
        &self,
    ) -> (
        Option<MoniToolValueInterpret>,
        Option<MoniToolValueSatisfies>,
        AsciiString,
        DataMap<AsciiString, i32>,
    ) {
        let state = self.state.read();
        let adds = state.theeadds.clone();
        (
            state.theinterp,
            state.thesatisf,
            state.thesatisn.clone(),
            adds,
        )
    }
    /// `Name`.
    pub fn name(&self) -> AsciiString {
        self.state.read().thename.clone()
    }
    /// `ValueType`.
    pub fn value_type(&self) -> MoniToolValueType {
        self.state.read().thetype
    }
    /// `SetLabel`.
    pub fn set_label(&self, label: &str) {
        self.state.write().thelabel = AsciiString::new(label);
    }
    /// `Label`: empty when not set.
    pub fn label(&self) -> AsciiString {
        self.state.read().thelabel.clone()
    }
    /// `SetMaxLength`: a negative maximum is 0 (none).
    pub fn set_max_length(&self, max: i32) {
        self.state.write().themaxlen = max.max(0);
    }
    /// `MaxLength`: 0 when not set.
    pub fn max_length(&self) -> i32 {
        self.state.read().themaxlen
    }
    /// `SetUnitDef`.
    pub fn set_unit_def(&self, def: &str) {
        self.state.write().theunidef = AsciiString::new(def);
    }
    /// `UnitDef`: empty when not set.
    pub fn unit_def(&self) -> AsciiString {
        self.state.read().theunidef.clone()
    }
    /// `SetObjectType`: `Standard_ConstructionError` unless an object typed value.
    pub fn set_object_type(&self, typ: &Handle<RuntimeType>) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.thetype != MoniToolValueType::ValueIdent {
            return Err(not_kind("AddEnum, Not an Entity/Object", state.thetype));
        }
        state.theotyp = typ.clone();
        Ok(())
    }
    /// `ObjectType`: the type set, else `Standard_Transient`.
    pub fn object_type(&self) -> Handle<RuntimeType> {
        let typ = self.state.read().theotyp.clone();
        if typ.is_some() {
            return typ;
        }
        StandardTransient::static_type_handle()
    }
    /// `SetInterpret`.
    pub fn set_interpret(&self, func: Option<MoniToolValueInterpret>) {
        self.state.write().theinterp = func;
    }
    /// `HasInterpret`: an interpret function is set, or the value is an enumeration.
    pub fn has_interpret(&self) -> bool {
        let state = self.state.read();
        state.theinterp.is_some() || state.thetype == MoniToolValueType::ValueEnum
    }
    /// `SetSatisfies`: the name is kept only with a function.
    pub fn set_satisfies(&self, func: Option<MoniToolValueSatisfies>, name: &str) {
        let mut state = self.state.write();
        state.thesatisf = func;
        state.thesatisn = if func.is_some() {
            AsciiString::new(name)
        } else {
            AsciiString::default()
        };
    }
    /// `SatisfiesName`: empty when none.
    pub fn satisfies_name(&self) -> AsciiString {
        self.state.read().thesatisn.clone()
    }
    /// `IsSetValue`: an object is set (object typed value), or the text is not empty.
    pub fn is_set_value(&self) -> bool {
        let state = self.state.read();
        if state.thetype == MoniToolValueType::ValueIdent {
            return state.theoval.is_some();
        }
        let text_set = state.thehval.get().is_some_and(|h| h.length() > 0);
        text_set || state.theoval.is_some()
    }
    /// `CStringValue`: empty when not set.
    pub fn c_string_value(&self) -> AsciiString {
        let hval = self.state.read().thehval.clone();
        hval.get()
            .map_or_else(AsciiString::default, |h| h.string().clone())
    }
    /// `HStringValue`: the shared string (null after `ClearValue`).
    pub fn h_string_value(&self) -> Handle<HAsciiString> {
        self.state.read().thehval.clone()
    }
    /// `ClearValue`.
    pub fn clear_value(&self) {
        let mut state = self.state.write();
        state.thehval.nullify();
        state.theoval.nullify();
        state.theival = 0;
    }
    /// `IntegerValue`: the integer, or the case of an enumeration.
    pub fn integer_value(&self) -> i32 {
        self.state.read().theival
    }
    /// `RealValue`: the text as a real, 0.0 when empty or not a real.
    pub fn real_value(&self) -> f64 {
        let hval = self.h_string_value();
        let Some(text) = hval.get() else { return 0.0 };
        if text.length() == 0 || !text.is_real_value() {
            return 0.0;
        }
        text.real_value().unwrap_or(0.0)
    }
    /// `ObjectValue`.
    pub fn object_value(&self) -> Handle<StandardTransient> {
        self.state.read().theoval.clone()
    }
    /// `GetObjectValue`.
    pub fn get_object_value(&self, val: &mut Handle<StandardTransient>) {
        *val = self.object_value();
    }
    /// `SetObjectValue`: only for an object typed value; the object must be kind of the
    /// object type when one is set; a null object clears the value.
    pub fn set_object_value(&self, obj: &Handle<StandardTransient>) -> bool {
        let mut state = self.state.write();
        if state.thetype != MoniToolValueType::ValueIdent {
            return false;
        }
        if let Some(typ) = state.theotyp.get()
            && obj.is_some()
            && !obj.is_kind(typ)
        {
            return false;
        }
        state.theoval = obj.clone();
        true
    }
    /// `ObjectTypeName`: empty without object, the value type name of an element, else
    /// the dynamic type name.
    pub fn object_type_name(&self) -> String {
        let oval = self.object_value();
        let element = oval.cast::<dyn super::MoniToolElement>();
        if let Some(element) = element.get() {
            return element.value_type_name();
        }
        oval.dynamic_type()
            .map_or_else(String::new, |t| t.name().to_owned())
    }
}

/// The `Standard_ConstructionError` of a specification member called on another type.
pub(super) fn not_kind(what: &str, kind: MoniToolValueType) -> Failure {
    exceptions::construction_error(&format!(
        "MoniTool_TypedValue : {what}; the value type is {kind:?}"
    ))
}

impl TypedValueState {
    /// The fields `MoniTool_TypedValue(name, type)` initialises.
    fn fresh(name: &str, kind: MoniToolValueType) -> Self {
        Self {
            thename: AsciiString::new(name),
            thedef: AsciiString::default(),
            thelabel: AsciiString::default(),
            thetype: kind,
            theotyp: Handle::default(),
            thelims: 0,
            themaxlen: 0,
            theintlow: 0,
            theintup: -1,
            therealow: 0.0,
            therealup: 0.0,
            theunidef: AsciiString::default(),
            theenums: None,
            theeadds: DataMap::default(),
            theinterp: None,
            thesatisf: None,
            thesatisn: AsciiString::default(),
            theival: 0,
            thehval: Handle::new(HAsciiString::new("")),
            theoval: Handle::default(),
        }
    }
}
