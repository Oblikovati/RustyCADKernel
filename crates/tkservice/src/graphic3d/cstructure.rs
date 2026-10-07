//! `Graphic3d_CStructure`: the driver-side structure (visualization-design.md section 1). The
//! class is abstract, so it is the [`Graphic3dCStructure`] trait over the embedded
//! [`Graphic3dCStructureBase`], with the non-virtual members in [`Graphic3dCStructureMembers`].
use super::{
    GRAPHIC3D_ZLAYER_ID_DEFAULT, Graphic3dBndBox3d, Graphic3dDisplayPriority,
    Graphic3dGraphicDriver, Graphic3dGroup, Graphic3dPresentationAttributesSource,
    Graphic3dSequenceOfHClipPlane, Graphic3dStructureManager, Graphic3dStructureSource,
    Graphic3dTransformPersMembers as _, Graphic3dTransformPersSource, Graphic3dViewAffinity,
    Graphic3dZLayerId,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkmath::toploc::Datum3;
use std::io;

/// The fields of a driver structure: the protected members, the public bit fields and the
/// public view affinity.
#[derive(Clone)]
pub struct Graphic3dCStructureFields {
    /// `myGraphicDriver`.
    pub graphic_driver: Handle<dyn Graphic3dGraphicDriver>,
    /// `myGroups`.
    pub groups: Vec<Handle<dyn Graphic3dGroup>>,
    /// `myBndBox`.
    pub bnd_box: Graphic3dBndBox3d,
    /// `myTrsf`.
    pub trsf: Handle<Datum3>,
    /// `myTrsfPers`.
    pub trsf_pers: Handle<dyn Graphic3dTransformPersSource>,
    /// `myClipPlanes`.
    pub clip_planes: Handle<Graphic3dSequenceOfHClipPlane>,
    /// `myHighlightStyle`: set only while highlighted.
    pub highlight_style: Handle<dyn Graphic3dPresentationAttributesSource>,
    /// `myId`.
    pub id: i32,
    /// `myZLayer`.
    pub z_layer: Graphic3dZLayerId,
    /// `myPriority`.
    pub priority: Graphic3dDisplayPriority,
    /// `myPreviousPriority`.
    pub previous_priority: Graphic3dDisplayPriority,
    /// `myIsCulled`: whether the structure is skipped after the BVH traversal.
    pub is_culled: bool,
    /// `myBndBoxClipCheck`: whether the box is checked against the clipping before drawing.
    pub bnd_box_clip_check: bool,
    /// `myHasGroupTrsf`: whether some groups may have transform persistence.
    pub has_group_trsf: bool,
    /// `myHasGroupFlipping`: whether some groups may have flipping options.
    pub has_group_flipping: bool,
    /// `IsInfinite`.
    pub is_infinite: bool,
    /// `stick`: set once the structure is in the scene graph (maybe hidden).
    pub stick: bool,
    /// `highlight`.
    pub highlight: bool,
    /// `visible`: hides the structure while keeping it in the scene graph.
    pub visible: bool,
    /// `HLRValidation`.
    pub hlr_validation: bool,
    /// `IsForHighlight`.
    pub is_for_highlight: bool,
    /// `IsMutable`.
    pub is_mutable: bool,
    /// `Is2dText`.
    pub is_2d_text: bool,
    /// `ViewAffinity`: the view visibility mask (null for every view).
    pub view_affinity: Handle<Graphic3dViewAffinity>,
}

/// The embedded state of a driver structure.
pub struct Graphic3dCStructureBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dCStructureFields>,
}

impl Graphic3dCStructureBase {
    /// The protected constructor: the manager's driver, a new identification from it, the
    /// default layer, normal priority, visible and culled.
    /// A null manager or driver is refused (natively a null dereference).
    /// Example: `Graphic3dCStructureBase::new(&manager)?`.
    pub fn new(manager: &Handle<Graphic3dStructureManager>) -> Result<Self, Failure> {
        let driver = manager
            .get()
            .map(|m| m.graphic_driver())
            .unwrap_or_default();
        let Some(driver_ref) = driver.get() else {
            return Err(program_error(
                "Graphic3d_CStructure: null structure manager or graphic driver; expected a driver",
            ));
        };
        let id = driver_ref.graphic_driver_base().new_identification()?;
        Ok(Self {
            fields: SharedState::new(default_fields(driver, id)),
        })
    }
}

/// The member initializers of the native constructor.
fn default_fields(
    graphic_driver: Handle<dyn Graphic3dGraphicDriver>,
    id: i32,
) -> Graphic3dCStructureFields {
    Graphic3dCStructureFields {
        graphic_driver,
        groups: Vec::new(),
        bnd_box: Graphic3dBndBox3d::default(),
        trsf: Handle::default(),
        trsf_pers: Handle::default(),
        clip_planes: Handle::default(),
        highlight_style: Handle::default(),
        id,
        z_layer: GRAPHIC3D_ZLAYER_ID_DEFAULT,
        priority: Graphic3dDisplayPriority::Normal,
        previous_priority: Graphic3dDisplayPriority::Normal,
        is_culled: true,
        bnd_box_clip_check: true,
        has_group_trsf: false,
        has_group_flipping: false,
        is_infinite: false,
        stick: false,
        highlight: false,
        visible: true,
        hlr_validation: false,
        is_for_highlight: false,
        is_mutable: false,
        is_2d_text: false,
        view_affinity: Handle::default(),
    }
}

/// A driver structure; the graphic members are implemented by the driver.
pub trait Graphic3dCStructure: Transient {
    /// The embedded base; `cstructure.cstructure_base()`.
    fn cstructure_base(&self) -> &Graphic3dCStructureBase;
    /// `SetTransformation`; `cstructure.set_transformation(datum)`.
    fn set_transformation(&self, trsf: &Handle<Datum3>) {
        self.cstructure_base().fields.write().trsf = trsf.clone();
    }
    /// `SetTransformPersistence`; `cstructure.set_transform_persistence(&pers)`.
    fn set_transform_persistence(&self, trsf_pers: &Handle<dyn Graphic3dTransformPersSource>) {
        self.cstructure_base().fields.write().trsf_pers = trsf_pers.clone();
    }
    /// `SetZLayer`; `cstructure.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP)`.
    fn set_z_layer(&self, layer_id: Graphic3dZLayerId) {
        self.cstructure_base().fields.write().z_layer = layer_id;
    }
    /// `OnVisibilityChanged`: updates the driver state after a visibility change.
    fn on_visibility_changed(&self);
    /// `Clear`: clears the graphic data.
    fn clear(&self);
    /// `Connect`: connects a child structure.
    fn connect(&self, structure: &dyn Graphic3dCStructure);
    /// `Disconnect`: disconnects a child structure.
    fn disconnect(&self, structure: &dyn Graphic3dCStructure);
    /// `GraphicHighlight`: highlights with a style.
    fn graphic_highlight(&self, style: &Handle<dyn Graphic3dPresentationAttributesSource>);
    /// `GraphicUnhighlight`.
    fn graphic_unhighlight(&self);
    /// `ShadowLink`: a shadow structure for the manager.
    fn shadow_link(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure>;
    /// `NewGroup`: a new group of `structure`, appended to the group sequence.
    fn new_group(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure>;
    /// `RemoveGroup`: removes a group from the sequence.
    fn remove_group(&self, group: &Handle<dyn Graphic3dGroup>);
    /// `updateLayerTransformation`: nothing by default.
    fn update_layer_transformation(&self) {}
    /// `DumpJson`: the groups, identification, layer, priorities, flags and boxes;
    /// `cstructure.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        super::driver_dump::dump_cstructure(self, stream, depth)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dCStructure,
    "Graphic3d_CStructure",
    StandardTransient
);

/// The non-virtual members of every driver structure.
pub trait Graphic3dCStructureMembers: Graphic3dCStructure {
    /// `GraphicDriver`; `cstructure.graphic_driver()`.
    fn graphic_driver(&self) -> Handle<dyn Graphic3dGraphicDriver> {
        self.cstructure_base().fields.read().graphic_driver.clone()
    }
    /// `Groups` (a copy of the sequence); `cstructure.groups().len()`.
    fn groups(&self) -> Vec<Handle<dyn Graphic3dGroup>> {
        self.cstructure_base().fields.read().groups.clone()
    }
    /// `Transformation`; `cstructure.transformation().is_null()`.
    fn transformation(&self) -> Handle<Datum3> {
        self.cstructure_base().fields.read().trsf.clone()
    }
    /// `TransformPersistence`; `cstructure.transform_persistence()`.
    fn transform_persistence(&self) -> Handle<dyn Graphic3dTransformPersSource> {
        self.cstructure_base().fields.read().trsf_pers.clone()
    }
    /// `HasGroupTransformPersistence`; `cstructure.has_group_transform_persistence()`.
    fn has_group_transform_persistence(&self) -> bool {
        self.cstructure_base().fields.read().has_group_trsf
    }
    /// `SetGroupTransformPersistence`; `cstructure.set_group_transform_persistence(true)`.
    fn set_group_transform_persistence(&self, value: bool) {
        self.cstructure_base().fields.write().has_group_trsf = value;
    }
    /// `HasGroupFlipping`; `cstructure.has_group_flipping()`.
    fn has_group_flipping(&self) -> bool {
        self.cstructure_base().fields.read().has_group_flipping
    }
    /// `SetGroupFlipping`; `cstructure.set_group_flipping(true)`.
    fn set_group_flipping(&self, value: bool) {
        self.cstructure_base().fields.write().has_group_flipping = value;
    }
    /// `ClipPlanes`; `cstructure.clip_planes()`.
    fn clip_planes(&self) -> Handle<Graphic3dSequenceOfHClipPlane> {
        self.cstructure_base().fields.read().clip_planes.clone()
    }
    /// `SetClipPlanes`; `cstructure.set_clip_planes(&planes)`.
    fn set_clip_planes(&self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
        self.cstructure_base().fields.write().clip_planes = planes.clone();
    }
    /// `BoundingBox`; `cstructure.bounding_box().is_valid()`.
    fn bounding_box(&self) -> Graphic3dBndBox3d {
        self.cstructure_base().fields.read().bnd_box
    }
    /// `ChangeBoundingBox` as an assignment; `cstructure.set_bounding_box(bounds)`.
    fn set_bounding_box(&self, bounds: Graphic3dBndBox3d) {
        self.cstructure_base().fields.write().bnd_box = bounds;
    }
    /// `IsVisible()`; `cstructure.is_visible()`.
    fn is_visible(&self) -> bool {
        self.cstructure_base().fields.read().visible
    }
    /// `IsVisible(theViewId)`: visible and not masked for the view; `cstructure.is_visible_in_view(0)`.
    fn is_visible_in_view(&self, view_id: i32) -> bool {
        let fields = self.cstructure_base().fields.read();
        fields.visible
            && fields
                .view_affinity
                .get()
                .is_none_or(|affinity| affinity.is_visible(view_id))
    }
    /// `ZLayer`; `cstructure.z_layer()`.
    fn z_layer(&self) -> Graphic3dZLayerId {
        self.cstructure_base().fields.read().z_layer
    }
    /// `HighlightStyle`; `cstructure.highlight_style()`.
    fn highlight_style(&self) -> Handle<dyn Graphic3dPresentationAttributesSource> {
        self.cstructure_base().fields.read().highlight_style.clone()
    }
    /// `Identification`; `cstructure.identification()`.
    fn identification(&self) -> i32 {
        self.cstructure_base().fields.read().id
    }
    /// `Priority`; `cstructure.priority()`.
    fn priority(&self) -> Graphic3dDisplayPriority {
        self.cstructure_base().fields.read().priority
    }
    /// `SetPriority`; `cstructure.set_priority(Graphic3dDisplayPriority::Above)`.
    fn set_priority(&self, priority: Graphic3dDisplayPriority) {
        self.cstructure_base().fields.write().priority = priority;
    }
    /// `PreviousPriority`; `cstructure.previous_priority()`.
    fn previous_priority(&self) -> Graphic3dDisplayPriority {
        self.cstructure_base().fields.read().previous_priority
    }
    /// `SetPreviousPriority`; `cstructure.set_previous_priority(Graphic3dDisplayPriority::Normal)`.
    fn set_previous_priority(&self, priority: Graphic3dDisplayPriority) {
        self.cstructure_base().fields.write().previous_priority = priority;
    }
    /// `IsCulled`; `cstructure.is_culled()`.
    fn is_culled(&self) -> bool {
        self.cstructure_base().fields.read().is_culled
    }
    /// `SetCulled`; `cstructure.set_culled(false)`.
    fn set_culled(&self, is_culled: bool) {
        self.cstructure_base().fields.write().is_culled = is_culled;
    }
    /// `MarkAsNotCulled`; `cstructure.mark_as_not_culled()`.
    fn mark_as_not_culled(&self) {
        self.set_culled(false);
    }
    /// `BndBoxClipCheck`; `cstructure.bnd_box_clip_check()`.
    fn bnd_box_clip_check(&self) -> bool {
        self.cstructure_base().fields.read().bnd_box_clip_check
    }
    /// `SetBndBoxClipCheck`; `cstructure.set_bnd_box_clip_check(false)`.
    fn set_bnd_box_clip_check(&self, value: bool) {
        self.cstructure_base().fields.write().bnd_box_clip_check = value;
    }
    /// `IsAlwaysRendered`: infinite, highlight, mutable, 2D text or trihedron/2D persistence;
    /// `cstructure.is_always_rendered()`.
    fn is_always_rendered(&self) -> bool {
        let fields = self.cstructure_base().fields.read();
        fields.is_infinite
            || fields.is_for_highlight
            || fields.is_mutable
            || fields.is_2d_text
            || fields
                .trsf_pers
                .get()
                .is_some_and(|pers| pers.is_trihedron_or_2d())
    }
}
impl<T: Graphic3dCStructure + ?Sized> Graphic3dCStructureMembers for T {}
