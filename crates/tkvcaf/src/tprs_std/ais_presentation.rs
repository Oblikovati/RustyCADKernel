// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_AISPresentation.hxx and .cxx
// (construction, data access and the attribute members) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TPrsStd_AISPresentation`: the attribute that shows a label in the interactive context of
//! the data framework's `TPrsStd_AISViewer`. Its settings live in the label's
//! `TDataXtd_Presentation`; the attribute holds the interactive object built by the driver
//! of that data (`TPrsStd_DriverTable`).
use super::TPrsStdAISViewer;
use rustycad_tkcaf::tdata_xtd::TDataXtdPresentation;
use rustycad_tkcdf::json_dump::{class_begin, field_dumped};
use rustycad_tkernel::{
    impl_transient,
    standard::{
        Failure, Guid, Handle, RuntimeTyped, SharedState, StandardTransient, dump::DumpStream,
        exceptions,
    },
};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeState,
    TdfDefaultDeltaOnModification, TdfDefaultDeltaOnRemoval, TdfDeltaOnAddition, TdfLabel,
    TdfRelocationTable,
};
use rustycad_tkv3d::ais::{AisInteractiveContext, AisInteractiveObject};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObject;
use std::io;

/// `TPrsStd_AISPresentation::GetID()`.
const AIS_PRESENTATION_ID: Guid = guid_from_text("3680ac6c-47ae-4366-bb94-26abb6e07341");

/// `NO_MORE_OBJECT`: the message of `Standard_NoMoreObject` when the label has no data.
const NO_MORE_OBJECT: &str = "TPrsStd_AISPresentation has no associated TDataXtd_PresentationData";

/// `TPrsStd_AISPresentation`.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::TDataXtdConstraint;
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkvcaf::tprs_std::TPrsStdAISPresentation;
/// let data = TdfData::new();
/// let label = data.root().find_child(1, true)?;
/// let presentation = TPrsStdAISPresentation::set(&label, &TDataXtdConstraint::get_id())?;
/// presentation.set_mode(3)?;
/// assert!(presentation.has_own_mode()? && presentation.mode()? == 3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct TPrsStdAISPresentation {
    transient: StandardTransient,
    base: TdfAttributeBase,
    /// `myAIS`: the interactive object built by the driver.
    pub(super) ais: SharedState<Handle<dyn AisInteractiveObject>>,
}
impl_transient!(TPrsStdAISPresentation, "TPrsStd_AISPresentation", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TPrsStdAISPresentation {
    fn default() -> Self {
        Self::new()
    }
}

impl TPrsStdAISPresentation {
    /// `TPrsStd_AISPresentation()`: no interactive object.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            ais: SharedState::new(Handle::default()),
        }
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        AIS_PRESENTATION_ID
    }
    /// `Set(theLabel, theDriver)`: finds or adds the label's `TDataXtd_Presentation` with
    /// the driver, then finds or adds the presentation attribute.
    pub fn set(label: &TdfLabel, driver: &Guid) -> Result<Handle<TPrsStdAISPresentation>, Failure> {
        let data = TDataXtdPresentation::set(label, driver)?;
        if data.is_null() {
            return Err(exceptions::no_more_object(NO_MORE_OBJECT));
        }
        let mut presentation: Handle<TPrsStdAISPresentation> = Handle::default();
        if !label.find_attribute_as(&Self::get_id(), &mut presentation)? {
            presentation = Handle::new(Self::new());
            label.add_attribute(&presentation.clone().cast(), true)?;
        }
        Ok(presentation)
    }
    /// `Set(theMaster)`: [`Self::set`] on the master's label with the master's ID as driver.
    pub fn set_from_master(
        master: &Handle<dyn TdfAttribute>,
    ) -> Result<Handle<TPrsStdAISPresentation>, Failure> {
        let master = rustycad_tklcaf::live(master, "TPrsStd_AISPresentation::Set")?;
        Self::set(&master.label(), &master.id())
    }
    /// `Unset(theLabel)`: forgets the presentation attribute, then the label's data.
    pub fn unset(label: &TdfLabel) -> Result<(), Failure> {
        let mut presentation: Handle<dyn TdfAttribute> = Handle::default();
        if label.find_attribute(&Self::get_id(), &mut presentation)? {
            label.forget_attribute(&presentation)?;
        }
        TDataXtdPresentation::unset(label)
    }
    /// `getData`: the label's `TDataXtd_Presentation`; `Standard_NoMoreObject` without it.
    pub fn get_data(&self) -> Result<Handle<TDataXtdPresentation>, Failure> {
        let mut data: Handle<TDataXtdPresentation> = Handle::default();
        if !self
            .label()
            .find_attribute_as(&TDataXtdPresentation::get_id(), &mut data)?
        {
            return Err(exceptions::no_more_object(NO_MORE_OBJECT));
        }
        Ok(data)
    }
    /// Reads one value of the data.
    pub(super) fn read_data<R>(
        &self,
        read: impl FnOnce(&TDataXtdPresentation) -> R,
    ) -> Result<R, Failure> {
        let data = self.get_data()?;
        Ok(read(rustycad_tklcaf::live(
            &data,
            "TPrsStd_AISPresentation",
        )?))
    }
    /// `GetAIS`: the interactive object (null before the first update).
    pub fn get_ais(&self) -> Handle<dyn AisInteractiveObject> {
        self.ais.read().clone()
    }
    /// Replaces the interactive object.
    pub(super) fn set_ais(&self, ais: Handle<dyn AisInteractiveObject>) {
        *self.ais.write() = ais;
    }
    /// `getAISContext`: the interactive context of the data framework's viewer, or null.
    pub(super) fn get_ais_context(&self) -> Result<Handle<AisInteractiveContext>, Failure> {
        let mut viewer: Handle<TPrsStdAISViewer> = Handle::default();
        if !TPrsStdAISViewer::find(&self.label(), &mut viewer)? {
            return Ok(Handle::default());
        }
        Ok(rustycad_tklcaf::live(&viewer, "TPrsStd_AISPresentation")?.get_interactive_context())
    }
}

/// The getters, read from the label's data (`Standard_NoMoreObject` without data).
impl TPrsStdAISPresentation {
    /// `IsDisplayed`.
    pub fn is_displayed(&self) -> Result<bool, Failure> {
        self.read_data(TDataXtdPresentation::is_displayed)
    }
    /// `GetDriverGUID`.
    pub fn get_driver_guid(&self) -> Result<Guid, Failure> {
        self.read_data(TDataXtdPresentation::get_driver_guid)
    }
    /// `Material`: the data's material index as a material name (`Standard_OutOfRange`
    /// for an index outside `Graphic3d_NameOfMaterial`, an invalid enumerator natively).
    pub fn material(
        &self,
    ) -> Result<rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial, Failure> {
        material_of_index(self.read_data(TDataXtdPresentation::material_index)?)
    }
    /// `HasOwnMaterial`.
    pub fn has_own_material(&self) -> Result<bool, Failure> {
        self.read_data(TDataXtdPresentation::has_own_material)
    }
    /// `Transparency`.
    pub fn transparency(&self) -> Result<f64, Failure> {
        self.read_data(TDataXtdPresentation::transparency)
    }
    /// `HasOwnTransparency`.
    pub fn has_own_transparency(&self) -> Result<bool, Failure> {
        self.read_data(TDataXtdPresentation::has_own_transparency)
    }
    /// `Color`.
    pub fn color(&self) -> Result<rustycad_tkernel::quantity::NamedColor, Failure> {
        self.read_data(TDataXtdPresentation::color)
    }
    /// `HasOwnColor`.
    pub fn has_own_color(&self) -> Result<bool, Failure> {
        self.read_data(TDataXtdPresentation::has_own_color)
    }
    /// `Width`.
    pub fn width(&self) -> Result<f64, Failure> {
        self.read_data(TDataXtdPresentation::width)
    }
    /// `HasOwnWidth`.
    pub fn has_own_width(&self) -> Result<bool, Failure> {
        self.read_data(TDataXtdPresentation::has_own_width)
    }
    /// `Mode`.
    pub fn mode(&self) -> Result<i32, Failure> {
        self.read_data(TDataXtdPresentation::mode)
    }
    /// `HasOwnMode`.
    pub fn has_own_mode(&self) -> Result<bool, Failure> {
        self.read_data(TDataXtdPresentation::has_own_mode)
    }
    /// `GetNbSelectionModes`.
    pub fn get_nb_selection_modes(&self) -> Result<i32, Failure> {
        self.read_data(TDataXtdPresentation::get_nb_selection_modes)
    }
    /// `SelectionMode(index)` (`index` is 1 by default natively).
    pub fn selection_mode(&self, index: i32) -> Result<i32, Failure> {
        self.read_data(|data| data.selection_mode(index))
    }
    /// `HasOwnSelectionMode`.
    pub fn has_own_selection_mode(&self) -> Result<bool, Failure> {
        self.read_data(TDataXtdPresentation::has_own_selection_mode)
    }
}

/// `(Graphic3d_NameOfMaterial)index`: the material of an index of the enumeration.
pub(super) fn material_of_index(
    index: i32,
) -> Result<rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial, Failure> {
    rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial::from_i32(index).ok_or_else(|| {
        exceptions::out_of_range(&format!(
            "TPrsStd_AISPresentation::Material: material index {index}; expected 0..=25"
        ))
    })
}

impl TPrsStdAISPresentation {
    /// `BeforeUndo` / `AfterUndo`: the presentation of the delta's label and the kind of
    /// delta (addition, default removal, default modification).
    fn undo_target(
        delta: &Handle<dyn TdfAttributeDelta>,
    ) -> Result<(Handle<TPrsStdAISPresentation>, UndoDeltaKind), Failure> {
        let delta_ref = rustycad_tklcaf::live(delta, "TPrsStd_AISPresentation::BeforeUndo")?;
        let mut presentation: Handle<TPrsStdAISPresentation> = Handle::default();
        delta_ref
            .label()
            .find_attribute_as(&Self::get_id(), &mut presentation)?;
        let kind = if delta.is_kind(TdfDeltaOnAddition::static_type()) {
            UndoDeltaKind::Addition
        } else if delta.is_kind(TdfDefaultDeltaOnRemoval::static_type()) {
            UndoDeltaKind::Removal
        } else if delta.is_kind(TdfDefaultDeltaOnModification::static_type()) {
            UndoDeltaKind::Modification
        } else {
            UndoDeltaKind::Other
        };
        Ok((presentation, kind))
    }
}

/// The delta kinds `BeforeUndo` and `AfterUndo` distinguish.
#[derive(Clone, Copy, PartialEq, Eq)]
enum UndoDeltaKind {
    Addition,
    Removal,
    Modification,
    Other,
}

impl TdfAttribute for TPrsStdAISPresentation {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        AIS_PRESENTATION_ID
    }
    /// `Restore`: drops the interactive object (rebuilt by the next update).
    fn restore(&self, _attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        self.set_ais(Handle::default());
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `BackupCopy`: a new empty presentation.
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        Ok(Handle::new(Self::new()).cast())
    }
    /// `Paste`: erases the target's interactive object (keeping its displayed flag) and
    /// drops it; the next update rebuilds it.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = into.cast::<TPrsStdAISPresentation>();
        let target = rustycad_tklcaf::live(&target, "TPrsStd_AISPresentation::Paste")?;
        target.backup()?;
        if !target.get_ais().is_null() {
            let displayed = target.is_displayed()?;
            target.ais_erase(true)?;
            if displayed {
                target.set_displayed(displayed)?;
            }
        }
        target.set_ais(Handle::default());
        Ok(())
    }
    /// `AfterAddition`: `AfterResume`.
    fn after_addition(&self) -> Result<(), Failure> {
        self.after_resume()
    }
    /// `BeforeRemoval`: `BeforeForget`.
    fn before_removal(&self) -> Result<(), Failure> {
        self.before_forget()
    }
    /// `BeforeForget`: removes the interactive object from its context and drops it.
    fn before_forget(&self) -> Result<(), Failure> {
        if !self.get_ais().is_null() {
            self.ais_erase(true)?;
            self.set_ais(Handle::default());
        }
        Ok(())
    }
    /// `AfterResume`: shows the label again when it is displayed, else erases it.
    fn after_resume(&self) -> Result<(), Failure> {
        if self.is_displayed()? {
            self.ais_update()?;
            self.ais_display()
        } else {
            self.ais_erase(false)
        }
    }
    /// `BeforeUndo`: an addition or a default modification is about to be undone: the
    /// presentation of the delta's label forgets its object.
    fn before_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        let (presentation, kind) = Self::undo_target(delta)?;
        let reset = matches!(kind, UndoDeltaKind::Addition | UndoDeltaKind::Modification);
        if let Some(presentation) = presentation.get().filter(|_| reset) {
            presentation.before_forget()?;
        }
        Ok(true)
    }
    /// `AfterUndo`: a removal was undone (the presentation is added again) or a default
    /// modification was undone (the presentation resumes).
    fn after_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        let (presentation, kind) = Self::undo_target(delta)?;
        let Some(presentation) = presentation.get() else {
            return Ok(true);
        };
        match kind {
            UndoDeltaKind::Removal => presentation.after_addition()?,
            UndoDeltaKind::Modification => presentation.after_resume()?,
            UndoDeltaKind::Addition | UndoDeltaKind::Other => {}
        }
        Ok(true)
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TPrsStd_AISPresentation")?;
        dump_attribute_json_base(self, stream, depth)?;
        let ais = self.get_ais();
        let Some(object) = ais.get() else {
            return Ok(());
        };
        let dump = |s: &mut DumpStream, d: i32| PrsMgrPresentableObject::dump_json(object, s, d);
        field_dumped(stream, depth, "myAIS.get()", Some(&dump))
    }
}
