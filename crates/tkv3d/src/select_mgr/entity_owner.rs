//! `SelectMgr_EntityOwner`: what a detected sensitive entity stands for, usually a selectable
//! object or a part of it (v3d-design.md section 6). The class is concrete but derived (the
//! BRep and AIS owners override its virtual members), so it is the
//! [`SelectMgrEntityOwnerSource`] trait over [`SelectMgrEntityOwnerBase`], with the concrete
//! class [`SelectMgrEntityOwner`].
use super::SelectMgrSelectableObject;
use crate::ais::AisSelectionScheme;
use crate::prs_mgr::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectMembers as _, PrsMgrPresentationManager,
};
use crate::prs3d::Prs3dDrawer;
use crate::v3d::V3dViewer;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle,
};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::aspect::{AspectVKeyFlags, AspectVKeyMouse};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dPresentationAttributesMembers as _, Graphic3dZLayerId,
};
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_pointer};
use std::io;

/// The protected fields of `SelectMgr_EntityOwner`.
#[derive(Clone)]
pub struct SelectMgrEntityOwnerFields {
    /// `mySelectable` (a raw pointer natively).
    pub selectable: WeakHandle<dyn SelectMgrSelectableObject>,
    /// `mypriority`.
    pub priority: i32,
    /// `myIsSelected`.
    pub is_selected: bool,
    /// `myFromDecomposition`.
    pub from_decomposition: bool,
}

/// The state every entity owner embeds.
pub struct SelectMgrEntityOwnerBase {
    /// The protected fields.
    pub fields: SharedState<SelectMgrEntityOwnerFields>,
    this: SharedState<WeakHandle<dyn SelectMgrEntityOwnerSource>>,
}

impl SelectMgrEntityOwnerBase {
    /// An owner of `selectable` (null for none) with a priority.
    /// Example: `SelectMgrEntityOwnerBase::new(&object, 0)`.
    pub fn new(selectable: &Handle<dyn SelectMgrSelectableObject>, priority: i32) -> Self {
        let fields = SelectMgrEntityOwnerFields {
            selectable: selectable.downgrade(),
            priority,
            is_selected: false,
            from_decomposition: false,
        };
        Self {
            fields: SharedState::new(fields),
            this: SharedState::new(WeakHandle::default()),
        }
    }
    /// A copy of the fields; `base.read()`.
    pub fn read(&self) -> SelectMgrEntityOwnerFields {
        self.fields.read().clone()
    }
    /// The handle of the owner, null before [`select_mgr_entity_owner_bind`]; `base.this()`.
    pub fn this(&self) -> Handle<dyn SelectMgrEntityOwnerSource> {
        self.this.read().upgrade()
    }
}

/// Records `owner` as its own handle (passed to `HilightOwnerWithColor`); every constructor of
/// an owner class calls it once the handle exists. Example: `select_mgr_entity_owner_bind(&owner)`.
pub fn select_mgr_entity_owner_bind(owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
    if let Some(value) = owner.get() {
        *value.owner_base().this.write() = owner.downgrade();
    }
}

/// The selectable object of an owner as a presentable object.
fn presentable_of(
    selectable: &Handle<dyn SelectMgrSelectableObject>,
) -> Handle<dyn PrsMgrPresentableObject> {
    selectable.cast()
}

/// An owner of detected entities; the default bodies are the native ones.
pub trait SelectMgrEntityOwnerSource: Transient {
    /// The embedded base; `owner.owner_base()`.
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase;
    /// `Selectable`: the selectable object, null without; `owner.selectable()`.
    fn selectable(&self) -> Handle<dyn SelectMgrSelectableObject> {
        self.owner_base().read().selectable.upgrade()
    }
    /// `SetSelectable`; `owner.set_selectable(&object)`.
    fn set_selectable(&self, selectable: &Handle<dyn SelectMgrSelectableObject>) {
        self.owner_base().fields.write().selectable = selectable.downgrade();
    }
    /// `HandleMouseClick`: false (not handled) by default.
    /// Example: `owner.handle_mouse_click(Vec2::new(1, 2), button, flags, false)`.
    fn handle_mouse_click(
        &self,
        point: Vec2<i32>,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        is_double_click: bool,
    ) -> bool {
        let _ = (point, button, modifiers, is_double_click);
        false
    }
    /// `IsHilighted`: the manager highlights the selectable object in the mode.
    /// Example: `owner.is_hilighted(&manager, 0)`.
    fn is_hilighted(&self, manager: &Handle<PrsMgrPresentationManager>, mode: i32) -> bool {
        let selectable = self.selectable();
        !selectable.is_null()
            && manager
                .get()
                .is_some_and(|m| m.is_highlighted(&presentable_of(&selectable), mode))
    }
    /// `HilightWithColor`: highlights the selectable object (in the style layer, else its
    /// own), or asks the object to highlight this owner when it is not auto-highlighted.
    /// Example: `owner.hilight_with_color(&manager, &style, 0)?`.
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        mode: i32,
    ) -> Result<(), Failure> {
        select_mgr_owner_hilight_with_color(self, manager, style, mode)
    }
    /// `Unhilight`: unhighlights the selectable object; `owner.unhilight(&manager, 0)`.
    fn unhilight(&self, manager: &Handle<PrsMgrPresentationManager>, mode: i32) {
        let _ = mode;
        let selectable = self.selectable();
        if let (false, Some(manager)) = (selectable.is_null(), manager.get()) {
            manager.unhighlight(&presentable_of(&selectable));
        }
    }
    /// `Clear`: nothing by default; `owner.clear(&manager, 0)`.
    fn clear(&self, manager: &Handle<PrsMgrPresentationManager>, mode: i32) {
        let _ = (manager, mode);
    }
    /// `HasLocation`: the selectable object has a transformation; `owner.has_location()`.
    fn has_location(&self) -> bool {
        self.selectable()
            .get()
            .is_some_and(|s| s.has_transformation())
    }
    /// `Location`: the transformation of the selectable object; `owner.location()`.
    fn location(&self) -> Location {
        match self.selectable().get().filter(|s| s.has_transformation()) {
            Some(object) => Location::new(object.transformation()),
            None => Location::default(),
        }
    }
    /// `SetLocation`: nothing by default; `owner.set_location(&location)`.
    fn set_location(&self, location: &Location) {
        let _ = location;
    }
    /// `IsAutoHilight`: true without selectable object, else the object's flag.
    /// Example: `owner.is_auto_hilight()`.
    fn is_auto_hilight(&self) -> bool {
        self.selectable().get().is_none_or(|s| s.is_auto_hilight())
    }
    /// `IsForcedHilight`: false by default; `owner.is_forced_hilight()`.
    fn is_forced_hilight(&self) -> bool {
        false
    }
    /// `SetZLayer`: nothing by default; `owner.set_z_layer(layer)`.
    fn set_z_layer(&self, layer_id: Graphic3dZLayerId) {
        let _ = layer_id;
    }
    /// `UpdateHighlightTrsf`: the manager updates the highlight transformation of the object.
    /// Example: `owner.update_highlight_trsf(&viewer, &manager, 0)?`.
    fn update_highlight_trsf(
        &self,
        viewer: &Handle<V3dViewer>,
        manager: &Handle<PrsMgrPresentationManager>,
        disp_mode: i32,
    ) -> Result<(), Failure> {
        let selectable = self.selectable();
        match (selectable.is_null(), manager.get()) {
            (false, Some(m)) => m.update_highlight_trsf(
                viewer,
                &presentable_of(&selectable),
                disp_mode,
                &Handle::default(),
            ),
            _ => Ok(()),
        }
    }
    /// `DumpJson`; `owner.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let _ = depth;
        select_mgr_entity_owner_dump_json(self, stream)
    }
}
impl rustycad_tkernel::standard::RuntimeTyped for dyn SelectMgrEntityOwnerSource {
    const TYPE_NAME: &'static str = "SelectMgr_EntityOwner";
    fn static_type() -> &'static rustycad_tkernel::standard::RuntimeType {
        // OCCT 3d097a0 has one concrete parent descriptor; the Rust dispatch seam is no second native class.
        <SelectMgrEntityOwner as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        // Runtime interface projection still returns the genuine inherited owner fields.
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}

/// The native body of `SelectMgr_EntityOwner::HilightWithColor`.
/// Example: `select_mgr_owner_hilight_with_color(&*owner, &manager, &style, 0)?`.
pub fn select_mgr_owner_hilight_with_color<O: SelectMgrEntityOwnerSource + ?Sized>(
    owner: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    mode: i32,
) -> Result<(), Failure> {
    let selectable = owner.selectable();
    let Some(object) = selectable.get() else {
        return Ok(());
    };
    if !owner.is_auto_hilight() {
        return object.hilight_owner_with_color(manager, style, &owner.owner_base().this());
    }
    let style_layer = style
        .get()
        .map_or(GRAPHIC3D_ZLAYER_ID_UNKNOWN, |s| s.z_layer());
    let layer = if style_layer != GRAPHIC3D_ZLAYER_ID_UNKNOWN {
        style_layer
    } else {
        object.z_layer()
    };
    match manager.get() {
        Some(m) => m.color(
            &presentable_of(&selectable),
            style,
            mode,
            &Handle::default(),
            layer,
        ),
        None => Ok(()),
    }
}

/// The native body of `SelectMgr_EntityOwner::DumpJson`.
/// Example: `select_mgr_entity_owner_dump_json(&*owner, &mut stream)?`.
pub fn select_mgr_entity_owner_dump_json<O: SelectMgrEntityOwnerSource + ?Sized>(
    owner: &O,
    stream: &mut DumpStream,
) -> io::Result<()> {
    class_begin(stream, "SelectMgr_EntityOwner")?;
    let fields = owner.owner_base().read();
    field_pointer(stream, "this", (owner as *const O).cast::<()>() as usize)?;
    field_pointer(
        stream,
        "mySelectable",
        fields.selectable.object_address() as usize,
    )?;
    field_integer(stream, "mypriority", fields.priority)?;
    field_integer(stream, "myIsSelected", i32::from(fields.is_selected))?;
    field_integer(
        stream,
        "myFromDecomposition",
        i32::from(fields.from_decomposition),
    )
}

/// The non-virtual members of every entity owner.
pub trait SelectMgrEntityOwnerMembers: SelectMgrEntityOwnerSource {
    /// `Priority`; `owner.priority()`.
    fn priority(&self) -> i32 {
        self.owner_base().read().priority
    }
    /// `SetPriority`; `owner.set_priority(2)`.
    fn set_priority(&self, priority: i32) {
        self.owner_base().fields.write().priority = priority;
    }
    /// `HasSelectable`; `owner.has_selectable()`.
    fn has_selectable(&self) -> bool {
        !self.owner_base().read().selectable.upgrade().is_null()
    }
    /// `IsSelected`; `owner.is_selected()`.
    fn is_selected(&self) -> bool {
        self.owner_base().read().is_selected
    }
    /// `SetSelected`; `owner.set_selected(true)`.
    fn set_selected(&self, is_selected: bool) {
        self.owner_base().fields.write().is_selected = is_selected;
    }
    /// `Select`: the selected state the scheme gives for a detected or undetected owner.
    /// Example: `owner.select(AisSelectionScheme::Add, true)`.
    fn select(&self, scheme: AisSelectionScheme, is_detected: bool) -> bool {
        let selected = self.is_selected();
        match scheme {
            AisSelectionScheme::Unknown => selected,
            AisSelectionScheme::Replace | AisSelectionScheme::ReplaceExtra => is_detected,
            AisSelectionScheme::Add => !selected || is_detected || self.is_forced_hilight(),
            AisSelectionScheme::Remove => selected && !is_detected,
            AisSelectionScheme::Xor if is_detected => !selected && !self.is_forced_hilight(),
            AisSelectionScheme::Xor => selected,
            AisSelectionScheme::Clear => false,
        }
    }
    /// `State()` (deprecated): 1 when selected, else 0; `owner.state()`.
    fn state(&self) -> i32 {
        i32::from(self.is_selected())
    }
    /// `State(status)` (deprecated): selected when the status is 1; `owner.set_state(1)`.
    fn set_state(&self, status: i32) {
        self.set_selected(status == 1);
    }
    /// `IsSameSelectable`: the same selectable object; `owner.is_same_selectable(&object)`.
    fn is_same_selectable(&self, other: &Handle<dyn SelectMgrSelectableObject>) -> bool {
        self.owner_base().read().selectable.object_address() == other.object_address()
    }
    /// `ComesFromDecomposition`; `owner.comes_from_decomposition()`.
    fn comes_from_decomposition(&self) -> bool {
        self.owner_base().read().from_decomposition
    }
    /// `SetComesFromDecomposition`; `owner.set_comes_from_decomposition(true)`.
    fn set_comes_from_decomposition(&self, is_from_decomposition: bool) {
        self.owner_base().fields.write().from_decomposition = is_from_decomposition;
    }
    /// `Set(selectable)` (deprecated): `SetSelectable`; `owner.set_selectable_deprecated(&object)`.
    fn set_selectable_deprecated(&self, selectable: &Handle<dyn SelectMgrSelectableObject>) {
        self.set_selectable(selectable);
    }
    /// `Set(priority)` (deprecated): `SetPriority`; `owner.set_priority_deprecated(1)`.
    fn set_priority_deprecated(&self, priority: i32) {
        self.set_priority(priority);
    }
}
impl<T: SelectMgrEntityOwnerSource + ?Sized> SelectMgrEntityOwnerMembers for T {}

/// The concrete `SelectMgr_EntityOwner`.
///
/// Example: `let owner = SelectMgrEntityOwner::new(0); owner.set_priority(3);`.
pub struct SelectMgrEntityOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
}
rustycad_tkernel::impl_transient!(SelectMgrEntityOwner, "SelectMgr_EntityOwner", StandardTransient, transient,
    interfaces: [dyn SelectMgrEntityOwnerSource]);

impl SelectMgrEntityOwnerSource for SelectMgrEntityOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
}

impl SelectMgrEntityOwner {
    /// An owner of `selectable` (null for none); the bound handle.
    fn bound(selectable: &Handle<dyn SelectMgrSelectableObject>, priority: i32) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(selectable, priority),
        });
        select_mgr_entity_owner_bind(&owner.clone().cast());
        owner
    }
    /// `SelectMgr_EntityOwner(priority)`: an owner without selectable object.
    /// Example: `SelectMgrEntityOwner::new(0)`.
    pub fn new(priority: i32) -> Handle<Self> {
        Self::bound(&Handle::default(), priority)
    }
    /// `SelectMgr_EntityOwner(selectable, priority)`; `SelectMgrEntityOwner::with_selectable(&object, 0)`.
    pub fn with_selectable(
        selectable: &Handle<dyn SelectMgrSelectableObject>,
        priority: i32,
    ) -> Handle<Self> {
        Self::bound(selectable, priority)
    }
    /// `SelectMgr_EntityOwner(owner, priority)`: the selectable object of another owner.
    /// Example: `SelectMgrEntityOwner::from_owner(&other, 0)`.
    pub fn from_owner(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        priority: i32,
    ) -> Handle<Self> {
        let selectable = owner
            .get()
            .map(|o| o.owner_base().read().selectable.upgrade())
            .unwrap_or_default();
        Self::bound(&selectable, priority)
    }
}
