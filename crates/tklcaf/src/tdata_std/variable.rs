// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_Variable.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_Variable`: a named, valued variable; its name, value and expression are the
//! `TDataStd_Name`, `TDataStd_Real` and `TDataStd_Expression` of the same label.
use super::support::cast_attribute;
use super::{
    TDataStdExpression, TDataStdGenericExtString, TDataStdName, TDataStdReal, TDataStdRealEnum,
};
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfDataSet, TdfLabel, TdfRelocationTable,
};
use rustycad_tkcdf::json_dump::{class_begin, field_integer, field_string};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream, exceptions,
    },
    tcollection::{AsciiString, ExtendedString},
};
use std::io;

/// `TDataStd_Variable::GetID()`.
const VARIABLE_ID: Guid = guid_from_text("ce241469-8e57-11d1-8953-080009dc4425");

/// The native fields `isConstant` and `myUnit`.
struct VariableState {
    is_constant: bool,
    unit: AsciiString,
}

/// `TDataStd_Variable`; `TDataStdVariable::set(&label)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdVariable, tdf::TdfData};
/// let data = TdfData::new();
/// let variable = TDataStdVariable::set(&data.root()).unwrap();
/// variable.set_value(2.5).unwrap();
/// assert!(variable.is_valued().unwrap() && TDataStdVariable::get(&variable).unwrap() == 2.5);
/// assert_eq!(variable.unit().as_bytes(), b"SCALAR");
/// ```
pub struct TDataStdVariable {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<VariableState>,
}
impl_transient!(TDataStdVariable, "TDataStd_Variable", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataStdVariable {
    /// `TDataStd_Variable()`: not constant, unit `"SCALAR"`.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(VariableState {
                is_constant: false,
                unit: AsciiString::new("SCALAR"),
            }),
        }
    }
}

/// The `Standard_DomainError` of the members that need an attribute the label lacks.
fn invalid_model(member: &str, label: &TdfLabel) -> Failure {
    exceptions::domain_error(&format!(
        "TDataStd_Variable::{member} : invalid model at {label:?}"
    ))
}

impl TDataStdVariable {
    /// `GetID`.
    pub fn get_id() -> Guid {
        VARIABLE_ID
    }
    /// `TDataStd_Variable()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label)`: the variable of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        let mut found: Handle<Self> = Handle::default();
        if label.find_attribute_as(&VARIABLE_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        label.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `Name(string)`: sets the `TDataStd_Name` of the label.
    pub fn set_name(&self, string: &ExtendedString) -> Result<(), Failure> {
        TDataStdName::set(&self.label(), string).map(|_| ())
    }
    /// `Name()`: the `TDataStd_Name` of the label; `Standard_DomainError` without one.
    pub fn name(&self) -> Result<ExtendedString, Failure> {
        let mut name: Handle<TDataStdName> = Handle::default();
        if !self
            .label()
            .find_attribute_as(&TDataStdName::get_id(), &mut name)?
        {
            return Err(invalid_model("Name", &self.label()));
        }
        Ok(TDataStdGenericExtString::get(live(
            &name,
            "TDataStd_Variable::Name",
        )?))
    }
    /// `Set(value)`: sets the `TDataStd_Real` of the label.
    pub fn set_value(&self, value: f64) -> Result<(), Failure> {
        TDataStdReal::set(&self.label(), value).map(|_| ())
    }
    /// `Set(value, dimension)`: as `Set(value)`; a new real also takes the dimension.
    pub fn set_value_with_dimension(
        &self,
        value: f64,
        dimension: TDataStdRealEnum,
    ) -> Result<(), Failure> {
        let was_valued = self.is_valued()?;
        let real = TDataStdReal::set(&self.label(), value)?;
        if !was_valued {
            live(&real, "TDataStd_Variable::Set")?.set_dimension(dimension)?;
        }
        Ok(())
    }
    /// `IsValued`: the label has a `TDataStd_Real`.
    pub fn is_valued(&self) -> Result<bool, Failure> {
        self.label().is_attribute(&TDataStdReal::get_id())
    }
    /// `Get`: the value of the `TDataStd_Real`; `Standard_DomainError` without one.
    pub fn get(&self) -> Result<f64, Failure> {
        let real = self.real()?;
        Ok(TDataStdReal::get(live(&real, "TDataStd_Variable::Get")?))
    }
    /// `Real`: the `TDataStd_Real` of the label; `Standard_DomainError` without one.
    pub fn real(&self) -> Result<Handle<TDataStdReal>, Failure> {
        let mut real: Handle<TDataStdReal> = Handle::default();
        if !self
            .label()
            .find_attribute_as(&TDataStdReal::get_id(), &mut real)?
        {
            return Err(invalid_model("Real", &self.label()));
        }
        Ok(real)
    }
    /// `IsAssigned`: the label has a `TDataStd_Expression`.
    pub fn is_assigned(&self) -> Result<bool, Failure> {
        self.label().is_attribute(&TDataStdExpression::get_id())
    }
    /// `Assign`: the `TDataStd_Expression` of the label, added when absent.
    pub fn assign(&self) -> Result<Handle<TDataStdExpression>, Failure> {
        TDataStdExpression::set(&self.label())
    }
    /// `Desassign`: forgets the expression; `Standard_DomainError` without one.
    pub fn desassign(&self) -> Result<(), Failure> {
        let expression = self.expression_or("Deassign")?;
        self.label().forget_attribute(&expression.cast())
    }
    /// `Expression`: the `TDataStd_Expression` of the label; `Standard_DomainError` without one.
    pub fn expression(&self) -> Result<Handle<TDataStdExpression>, Failure> {
        self.expression_or("GetExpression")
    }
    /// The expression of the label, or the `Standard_DomainError` the member raises.
    fn expression_or(&self, member: &str) -> Result<Handle<TDataStdExpression>, Failure> {
        let mut expression: Handle<TDataStdExpression> = Handle::default();
        if !self
            .label()
            .find_attribute_as(&TDataStdExpression::get_id(), &mut expression)?
        {
            let message = format!("TDataStd_Variable::{member} at {:?}", self.label());
            return Err(exceptions::domain_error(&message));
        }
        Ok(expression)
    }
    /// `IsCaptured`: the real is captured (`TDataStd_Real::IsCaptured`).
    pub fn is_captured(&self) -> Result<bool, Failure> {
        live(&self.real()?, "TDataStd_Variable::IsCaptured")?.is_captured()
    }
    /// `IsConstant`.
    pub fn is_constant(&self) -> bool {
        self.state.read().is_constant
    }
    /// `Constant(status)`: backed up when it changes (OCC2932).
    pub fn constant(&self, status: bool) -> Result<(), Failure> {
        if self.is_constant() == status {
            return Ok(());
        }
        self.backup()?;
        self.state.write().is_constant = status;
        Ok(())
    }
    /// `Unit(unit)`: backed up when it changes (OCC2932).
    pub fn set_unit(&self, unit: &AsciiString) -> Result<(), Failure> {
        if self.state.read().unit == *unit {
            return Ok(());
        }
        self.backup()?;
        self.state.write().unit = unit.clone();
        Ok(())
    }
    /// `Unit()`.
    pub fn unit(&self) -> AsciiString {
        self.state.read().unit.clone()
    }
}

impl TdfAttribute for TDataStdVariable {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        VARIABLE_ID
    }
    /// `Restore(With)`: the constant flag and the unit.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TDataStd_Variable::Restore")?;
        let other = live(&other, "TDataStd_Variable::Restore")?;
        let (is_constant, unit) = (other.is_constant(), other.unit());
        let mut state = self.state.write();
        (state.is_constant, state.unit) = (is_constant, unit);
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste(Into, RT)`: `Into` takes the constant flag and the unit.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<Self>(into, "TDataStd_Variable::Paste")?;
        let target = live(&target, "TDataStd_Variable::Paste")?;
        target.constant(self.is_constant())?;
        target.set_unit(&self.unit())
    }
    /// `References`: the `TDataStd_Name` of the label.
    fn references(&self, data_set: &Handle<TdfDataSet>) {
        let mut name: Handle<dyn TdfAttribute> = Handle::default();
        let found = self
            .label()
            .find_attribute(&TDataStdName::get_id(), &mut name)
            .unwrap_or(false);
        if let (true, Some(data_set)) = (found, data_set.get()) {
            data_set.add_attribute(&name);
        }
    }
    /// `Dump`: `"Variable"`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Variable")
    }
    /// `DumpJson`: the constant flag and the unit.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_Variable")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_integer(stream, "isConstant", i32::from(self.is_constant()))?;
        field_string(
            stream,
            "myUnit",
            &String::from_utf8_lossy(self.unit().as_bytes()),
        )
    }
}
