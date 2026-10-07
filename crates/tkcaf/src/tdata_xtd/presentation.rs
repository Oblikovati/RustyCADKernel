// Rust port of ApplicationFramework/TKCAF/TDataXtd/TDataXtd_Presentation.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataXtd_Presentation`: the application-independent presentation settings of a label
//! (driver, display flag, color, material, transparency, width, modes).
use super::presentation_colors;
use rustycad_tkernel::quantity::NamedColor;
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::support::cast_attribute;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable,
};

/// `TDataXtd_Presentation::GetID()`.
const PRESENTATION_ID: Guid = guid_from_text("04fb4d00-5690-11d1-8940-080009dc3333");

/// The fields of `TDataXtd_Presentation`.
#[derive(Clone)]
struct PresentationState {
    driver: Guid,
    color: NamedColor,
    material_index: i32,
    mode: i32,
    selection_modes: Vec<i32>,
    transparency: f64,
    width: f64,
    displayed: bool,
    own_color: bool,
    own_material: bool,
    own_transparency: bool,
    own_width: bool,
    own_mode: bool,
    own_selection_mode: bool,
}

impl Default for PresentationState {
    fn default() -> Self {
        Self {
            driver: Guid::default(),
            color: NamedColor::WHITE,
            material_index: 0,
            mode: 0,
            selection_modes: Vec::new(),
            transparency: 0.0,
            width: 0.0,
            displayed: false,
            own_color: false,
            own_material: false,
            own_transparency: false,
            own_width: false,
            own_mode: false,
            own_selection_mode: false,
        }
    }
}

/// `TDataXtd_Presentation`.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::TDataXtdPresentation;
/// use rustycad_tkernel::quantity::NamedColor;
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let driver = TDataXtdPresentation::get_id();
/// let presentation = TDataXtdPresentation::set(&data.root(), &driver).unwrap();
/// presentation.set_color(NamedColor::RED).unwrap();
/// assert!(presentation.has_own_color() && presentation.color() == NamedColor::RED);
/// ```
pub struct TDataXtdPresentation {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<PresentationState>,
}
impl_transient!(TDataXtdPresentation, "TDataXtd_Presentation", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataXtdPresentation {
    fn default() -> Self {
        Self::new()
    }
}

impl TDataXtdPresentation {
    /// `TDataXtd_Presentation()`: null driver, white, nothing owned, not displayed.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(PresentationState::default()),
        }
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        PRESENTATION_ID
    }
    /// `Set(theLabel, theDriverId)`: finds or adds the presentation and sets its driver.
    pub fn set(label: &TdfLabel, driver: &Guid) -> Result<Handle<TDataXtdPresentation>, Failure> {
        let mut presentation: Handle<TDataXtdPresentation> = Handle::default();
        if !label.find_attribute_as(&Self::get_id(), &mut presentation)? {
            presentation = Handle::new(Self::new());
            label.add_attribute(&presentation.cast(), true)?;
        }
        rustycad_tklcaf::live(&presentation, "TDataXtd_Presentation::Set")?
            .set_driver_guid(driver)?;
        Ok(presentation)
    }
    /// `Unset(theLabel)`: forgets the presentation of the label, if any.
    pub fn unset(label: &TdfLabel) -> Result<(), Failure> {
        let mut presentation: Handle<dyn TdfAttribute> = Handle::default();
        if label.find_attribute(&Self::get_id(), &mut presentation)? {
            label.forget_attribute(&presentation)?;
        }
        Ok(())
    }
    /// `getColorNameFromOldEnum`.
    pub fn get_color_name_from_old_enum(old: i32) -> Result<NamedColor, Failure> {
        presentation_colors::color_from_old(old)
    }
    /// `getOldColorNameFromNewEnum`.
    pub fn get_old_color_name_from_new_enum(color: NamedColor) -> i32 {
        presentation_colors::old_from_color(color)
    }
    fn read(&self) -> PresentationState {
        self.state.read().clone()
    }
    /// Backs up (on request) and changes the state when `changed`.
    fn change(
        &self,
        changed: bool,
        backup: bool,
        apply: impl FnOnce(&mut PresentationState),
    ) -> Result<(), Failure> {
        if !changed {
            return Ok(());
        }
        if backup {
            self.backup()?;
        }
        apply(&mut self.state.write());
        Ok(())
    }
}

/// The getters (`GetDriverGUID`, `IsDisplayed`, `HasOwn...`, the values).
impl TDataXtdPresentation {
    /// `GetDriverGUID`.
    pub fn get_driver_guid(&self) -> Guid {
        self.state.read().driver
    }
    /// `IsDisplayed`.
    pub fn is_displayed(&self) -> bool {
        self.state.read().displayed
    }
    /// `HasOwnMaterial`.
    pub fn has_own_material(&self) -> bool {
        self.state.read().own_material
    }
    /// `HasOwnTransparency`.
    pub fn has_own_transparency(&self) -> bool {
        self.state.read().own_transparency
    }
    /// `HasOwnColor`.
    pub fn has_own_color(&self) -> bool {
        self.state.read().own_color
    }
    /// `HasOwnWidth`.
    pub fn has_own_width(&self) -> bool {
        self.state.read().own_width
    }
    /// `HasOwnMode`.
    pub fn has_own_mode(&self) -> bool {
        self.state.read().own_mode
    }
    /// `HasOwnSelectionMode`.
    pub fn has_own_selection_mode(&self) -> bool {
        self.state.read().own_selection_mode
    }
    /// `MaterialIndex`.
    pub fn material_index(&self) -> i32 {
        self.state.read().material_index
    }
    /// `Transparency`.
    pub fn transparency(&self) -> f64 {
        self.state.read().transparency
    }
    /// `Color`.
    pub fn color(&self) -> NamedColor {
        self.state.read().color
    }
    /// `Width`.
    pub fn width(&self) -> f64 {
        self.state.read().width
    }
    /// `Mode`.
    pub fn mode(&self) -> i32 {
        self.state.read().mode
    }
    /// `GetNbSelectionModes`.
    pub fn get_nb_selection_modes(&self) -> i32 {
        i32::try_from(self.state.read().selection_modes.len()).unwrap_or(i32::MAX)
    }
    /// `SelectionMode(index)`: the index-th (1-based) selection mode, 0 outside the list.
    pub fn selection_mode(&self, index: i32) -> i32 {
        let position = usize::try_from(index - 1).ok();
        position
            .and_then(|p| self.state.read().selection_modes.get(p).copied())
            .unwrap_or(0)
    }
}

/// The setters, each backed up when it changes something.
impl TDataXtdPresentation {
    /// `SetDriverGUID`.
    pub fn set_driver_guid(&self, driver: &Guid) -> Result<(), Failure> {
        self.change(self.get_driver_guid() != *driver, true, |s| {
            s.driver = *driver
        })
    }
    /// `SetDisplayed`.
    pub fn set_displayed(&self, displayed: bool) -> Result<(), Failure> {
        self.change(self.is_displayed() != displayed, true, |s| {
            s.displayed = displayed
        })
    }
    /// `SetMaterialIndex`: owned from now on.
    pub fn set_material_index(&self, index: i32) -> Result<(), Failure> {
        let s = self.read();
        self.change(!s.own_material || s.material_index != index, true, |s| {
            (s.material_index, s.own_material) = (index, true);
        })
    }
    /// `SetTransparency`: owned from now on.
    pub fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        let s = self.read();
        self.change(!s.own_transparency || s.transparency != value, true, |s| {
            (s.transparency, s.own_transparency) = (value, true);
        })
    }
    /// `SetColor`: owned from now on.
    pub fn set_color(&self, color: NamedColor) -> Result<(), Failure> {
        let s = self.read();
        self.change(!s.own_color || s.color != color, true, |s| {
            (s.color, s.own_color) = (color, true)
        })
    }
    /// `SetWidth`: owned from now on.
    pub fn set_width(&self, width: f64) -> Result<(), Failure> {
        let s = self.read();
        self.change(!s.own_width || s.width != width, true, |s| {
            (s.width, s.own_width) = (width, true)
        })
    }
    /// `SetMode`: owned from now on.
    pub fn set_mode(&self, mode: i32) -> Result<(), Failure> {
        let s = self.read();
        self.change(!s.own_mode || s.mode != mode, true, |s| {
            (s.mode, s.own_mode) = (mode, true)
        })
    }
    /// `SetSelectionMode(mode, theTransaction)`: the only selection mode, unless it already
    /// is the first of at most one (an owned empty list is kept, as natively).
    pub fn set_selection_mode(&self, mode: i32, transaction: bool) -> Result<(), Failure> {
        let s = self.read();
        let changed = !s.own_selection_mode
            || s.selection_modes.len() > 1
            || s.selection_modes
                .first()
                .is_some_and(|&first| first != mode);
        self.change(changed, transaction, |s| {
            (s.selection_modes, s.own_selection_mode) = (vec![mode], true);
        })
    }
    /// `AddSelectionMode(mode, theTransaction)`: appends a mode not in the list yet (always
    /// when no selection mode is owned).
    pub fn add_selection_mode(&self, mode: i32, transaction: bool) -> Result<(), Failure> {
        let s = self.read();
        let changed = !s.own_selection_mode || !s.selection_modes.contains(&mode);
        self.change(changed, transaction, |s| {
            s.selection_modes.push(mode);
            s.own_selection_mode = true;
        })
    }
}

/// The `Unset...` methods: each backed up when the value was owned.
impl TDataXtdPresentation {
    /// `UnsetMaterial`.
    pub fn unset_material(&self) -> Result<(), Failure> {
        self.change(self.has_own_material(), true, |s| s.own_material = false)
    }
    /// `UnsetTransparency`.
    pub fn unset_transparency(&self) -> Result<(), Failure> {
        self.change(self.has_own_transparency(), true, |s| {
            s.own_transparency = false
        })
    }
    /// `UnsetColor`.
    pub fn unset_color(&self) -> Result<(), Failure> {
        self.change(self.has_own_color(), true, |s| s.own_color = false)
    }
    /// `UnsetWidth`.
    pub fn unset_width(&self) -> Result<(), Failure> {
        self.change(self.has_own_width(), true, |s| s.own_width = false)
    }
    /// `UnsetMode`.
    pub fn unset_mode(&self) -> Result<(), Failure> {
        self.change(self.has_own_mode(), true, |s| s.own_mode = false)
    }
    /// `UnsetSelectionMode`: also clears the modes.
    pub fn unset_selection_mode(&self) -> Result<(), Failure> {
        self.change(self.has_own_selection_mode(), true, |s| {
            s.own_selection_mode = false;
            s.selection_modes.clear();
        })
    }
}

/// `Paste`: the owned values with their flags (a value not owned keeps the target's), the
/// display flag and the driver.
fn paste_state(source: &PresentationState, target: &mut PresentationState) {
    if source.own_material {
        target.material_index = source.material_index;
    }
    if source.own_color {
        target.color = source.color;
    }
    if source.own_width {
        target.width = source.width;
    }
    if source.own_mode {
        target.mode = source.mode;
    }
    if source.own_selection_mode {
        target.selection_modes.clone_from(&source.selection_modes);
    }
    if source.own_transparency {
        target.transparency = source.transparency;
    }
    target.own_material = source.own_material;
    target.own_color = source.own_color;
    target.own_width = source.own_width;
    target.own_mode = source.own_mode;
    target.own_selection_mode = source.own_selection_mode;
    target.own_transparency = source.own_transparency;
    (target.displayed, target.driver) = (source.displayed, source.driver);
}

impl TdfAttribute for TDataXtdPresentation {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        PRESENTATION_ID
    }
    /// `Restore`: every field.
    fn restore(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other =
            cast_attribute::<TDataXtdPresentation>(attribute, "TDataXtd_Presentation::Restore")?;
        let state = rustycad_tklcaf::live(&other, "TDataXtd_Presentation::Restore")?.read();
        *self.state.write() = state;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `BackupCopy`: a new presentation with every field.
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        let copy = Self::new();
        *copy.state.write() = self.read();
        Ok(Handle::new(copy).cast())
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TDataXtdPresentation>(into, "TDataXtd_Presentation::Paste")?;
        let target = rustycad_tklcaf::live(&target, "TDataXtd_Presentation::Paste")?;
        target.backup()?;
        paste_state(&self.read(), &mut target.state.write());
        Ok(())
    }
}
