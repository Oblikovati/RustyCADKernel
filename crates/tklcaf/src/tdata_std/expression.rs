// Rust port of ApplicationFramework/TKLCAF/TDataStd/TDataStd_Expression and
// TDataStd_Relation (.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataStd_Expression` (an expression string and the variables it uses) and its derived
//! class `TDataStd_Relation`, which embeds it: a relation handle casts to the expression,
//! and the members the relation inherits are the expression's.
use super::TDataStdVariable;
use super::support::cast_attribute;
use crate::guid_from_text;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable};
use rustycad_tkcdf::json_dump::{class_begin, field_dumped, field_string};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::ExtendedString,
};
use std::io;
use std::sync::RwLockWriteGuard;

/// `TDataStd_Expression::GetID()`.
const EXPRESSION_ID: Guid = guid_from_text("ce24146a-8e57-11d1-8953-080009dc4425");
/// `TDataStd_Relation::GetID()`.
const RELATION_ID: Guid = guid_from_text("ce24146b-8e57-11d1-8953-080009dc4425");

/// `TDataStd_Expression`; `TDataStdExpression::set(&label)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::TDataStdExpression, tdf::TdfData};
/// use rustycad_tkernel::tcollection::ExtendedString;
/// let data = TdfData::new();
/// let expression = TDataStdExpression::set(&data.root()).unwrap();
/// expression.set_expression(&ExtendedString::new("a+b")).unwrap();
/// assert!(expression.name().is_equal(ExtendedString::new("a+b")));
/// ```
#[derive(Default)]
pub struct TDataStdExpression {
    transient: StandardTransient,
    base: TdfAttributeBase,
    expression: SharedState<ExtendedString>,
    variables: SharedState<Vec<Handle<dyn TdfAttribute>>>,
}
impl_transient!(TDataStdExpression, "TDataStd_Expression", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TDataStdExpression {
    /// `GetID`.
    pub fn get_id() -> Guid {
        EXPRESSION_ID
    }
    /// `TDataStd_Expression()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label)`: the expression of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        find_or_add(label, &EXPRESSION_ID, Self::new)
    }
    /// `Name`: the expression string.
    pub fn name(&self) -> ExtendedString {
        self.get_expression()
    }
    /// `SetExpression(E)`: backed up when it changes (OCC2932).
    pub fn set_expression(&self, expression: &ExtendedString) -> Result<(), Failure> {
        if *self.expression.read() == *expression {
            return Ok(());
        }
        self.backup_as_dynamic()?;
        *self.expression.write() = expression.clone();
        Ok(())
    }
    /// `Backup()` through the attribute's own handle, so that the virtual `BackupCopy` is
    /// the derived class's (a `TDataStd_Relation` backs up as a relation, not as the
    /// embedded expression); an attribute without handle has no label and backs up nothing.
    fn backup_as_dynamic(&self) -> Result<(), Failure> {
        match self.base.this() {
            Ok(this) => live(&this, "TDataStd_Expression::Backup")?.backup(),
            Err(_) => self.backup(),
        }
    }
    /// `GetExpression`.
    pub fn get_expression(&self) -> ExtendedString {
        self.expression.read().clone()
    }
    /// `GetVariables`: the variables, lent through the write guard of the list.
    pub fn get_variables(&self) -> RwLockWriteGuard<'_, Vec<Handle<dyn TdfAttribute>>> {
        self.variables.write()
    }
    /// The shared body of `Restore`: the other expression's string, and its variables cast
    /// to `TDataStd_Variable` (an attribute of another class becomes null, as natively).
    fn restore_expression(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TDataStd_Expression::Restore")?;
        let other = live(&other, "TDataStd_Expression::Restore")?;
        let expression = other.get_expression();
        let variables = other
            .variables
            .read()
            .iter()
            .map(|v| v.cast::<TDataStdVariable>().cast())
            .collect();
        *self.expression.write() = expression;
        *self.variables.write() = variables;
        Ok(())
    }
    /// The shared body of `Paste`: `Into` takes the string and gains the relocation of each
    /// variable (null when the table has none); `Standard_NullObject` when `Into` is not an
    /// expression.
    fn paste_expression(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = into.cast::<Self>();
        let target = live(&target, "TDataStd_Expression::Paste")?;
        target.set_expression(&self.get_expression())?;
        let variables = self.variables.read().clone();
        for variable in variables {
            let mut relocated = Handle::default();
            live(table, "TDataStd_Expression::Paste")?.has_attribute_relocation(
                &variable.cast::<TDataStdVariable>().cast(),
                &mut relocated,
            );
            target.get_variables().push(relocated);
        }
        Ok(())
    }
    /// The variable dumps of `DumpJson`, each as `"<field>": {...}`.
    fn dump_variables(&self, stream: &mut DumpStream, depth: i32, field: &str) -> io::Result<()> {
        let variables = self.variables.read().clone();
        for variable in &variables {
            let child = variable
                .get()
                .map(|v| move |s: &mut DumpStream, d: i32| v.dump_json(s, d));
            field_dumped(stream, depth, field, child.as_ref().map(|c| c as _))?;
        }
        Ok(())
    }
}

/// The `Set(label)` of both classes: the attribute of `id`, added when absent.
fn find_or_add<T: TdfAttribute + rustycad_tkernel::standard::RuntimeTyped>(
    label: &TdfLabel,
    id: &Guid,
    make: impl FnOnce() -> T,
) -> Result<Handle<T>, Failure> {
    let mut found: Handle<T> = Handle::default();
    if label.find_attribute_as(id, &mut found)? {
        return Ok(found);
    }
    let created = Handle::new(make());
    label.add_attribute(&created.cast(), false)?;
    Ok(created)
}

impl TdfAttribute for TDataStdExpression {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        EXPRESSION_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        self.restore_expression(with)
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        self.paste_expression(into, table)
    }
    /// `Dump`: `"Expression"`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Expression")
    }
    /// `DumpJson`: the string and the dump of every variable.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_Expression")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_string(stream, "myExpression", &self.get_expression().to_utf8())?;
        self.dump_variables(stream, depth, "anAttribute.get()")
    }
}

/// `TDataStd_Relation`: an expression that states a relation; `TDataStdRelation::set(&label)`.
///
/// ```
/// use rustycad_tklcaf::{tdata_std::{TDataStdExpression, TDataStdRelation}, tdf::TdfData};
/// use rustycad_tkernel::tcollection::ExtendedString;
/// let data = TdfData::new();
/// let relation = TDataStdRelation::set(&data.root()).unwrap();
/// relation.set_relation(&ExtendedString::new("a=b")).unwrap();
/// assert!(!relation.cast::<TDataStdExpression>().is_null());
/// ```
#[derive(Default)]
pub struct TDataStdRelation {
    expression: TDataStdExpression,
}
impl_transient!(TDataStdRelation, "TDataStd_Relation", TDataStdExpression, expression, interfaces: [dyn TdfAttribute]);

impl TDataStdRelation {
    /// `GetID`.
    pub fn get_id() -> Guid {
        RELATION_ID
    }
    /// `TDataStd_Relation()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(label)`: the relation of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        find_or_add(label, &RELATION_ID, Self::new)
    }
    /// The embedded `TDataStd_Expression` (the native base class).
    pub fn expression(&self) -> &TDataStdExpression {
        &self.expression
    }
    /// `SetRelation(R)`: `SetExpression`.
    pub fn set_relation(&self, relation: &ExtendedString) -> Result<(), Failure> {
        self.expression.set_expression(relation)
    }
    /// `GetRelation`: `GetExpression`.
    pub fn get_relation(&self) -> ExtendedString {
        self.expression.get_expression()
    }
}

impl TdfAttribute for TDataStdRelation {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.expression.base
    }
    fn id(&self) -> Guid {
        RELATION_ID
    }
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        self.expression.restore_expression(with)
    }
    /// `NewEmpty`: the registered derived-attribute creator, a new relation.
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        self.expression.paste_expression(into, table)
    }
    /// `Dump`: `"Relation"`.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Relation")
    }
    /// `DumpJson`: the relation string and the dump of every variable.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDataStd_Relation")?;
        dump_attribute_json_base(self, stream, depth)?;
        field_string(stream, "GetRelation()", &self.get_relation().to_utf8())?;
        self.expression
            .dump_variables(stream, depth, "aVariable.get()")
    }
}
