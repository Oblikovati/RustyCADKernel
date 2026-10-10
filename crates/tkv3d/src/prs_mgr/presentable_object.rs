//! `PrsMgr_PresentableObject`: the abstract object a presentation manager displays (v3d-design.md
//! section 5). The class is abstract and its virtual members are overridden down to AIS, so it
//! is the [`PrsMgrPresentableObject`] trait over the embedded [`PrsMgrPresentableObjectBase`];
//! each virtual member has a default body that calls the native base body (the `prs_mgr_*`
//! functions of presentable_defaults.rs and presentable_style.rs), which an override may call
//! as the native `PrsMgr_PresentableObject::Member` call. The non-virtual members are in
//! [`super::PrsMgrPresentableObjectMembers`].
use super::presentable_defaults as base;
use super::presentable_style as style;
use super::{
    PrsMgrDisplayStatus, PrsMgrPresentation, PrsMgrPresentationManager, PrsMgrTypeOfPresentation3d,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle,
};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{GeneralTransform3, Transform3};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::aspect::AspectTypeOfFacingModel;
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dClipPlane, Graphic3dMaterialAspect, Graphic3dNameOfMaterial,
    Graphic3dPresentationAttributesMembers as _, Graphic3dPresentationAttributesSource as _,
    Graphic3dSequenceOfHClipPlane, Graphic3dStructureSource, Graphic3dTransformPersSource,
    Graphic3dViewAffinity, Graphic3dZLayerId,
};
use std::io;

/// The protected fields of `PrsMgr_PresentableObject`; a subclass edits them through
/// [`PrsMgrPresentableObjectBase::fields`].
#[derive(Clone)]
pub struct PrsMgrPresentableObjectFields {
    /// `myParent` (a raw pointer natively).
    pub parent: WeakHandle<dyn PrsMgrPresentableObject>,
    /// `myPresentations`.
    pub presentations: Vec<Handle<PrsMgrPresentation>>,
    /// `myViewAffinity`.
    pub view_affinity: Handle<Graphic3dViewAffinity>,
    /// `myClipPlanes`.
    pub clip_planes: Handle<Graphic3dSequenceOfHClipPlane>,
    /// `myDrawer`.
    pub drawer: Handle<Prs3dDrawer>,
    /// `myHilightDrawer`.
    pub hilight_drawer: Handle<Prs3dDrawer>,
    /// `myDynHilightDrawer`.
    pub dyn_hilight_drawer: Handle<Prs3dDrawer>,
    /// `myTransformPersistence`.
    pub transform_persistence: Handle<dyn Graphic3dTransformPersSource>,
    /// `myLocalTransformation`.
    pub local_transformation: Handle<Datum3>,
    /// `myTransformation`.
    pub transformation: Handle<Datum3>,
    /// `myCombinedParentTransform`.
    pub combined_parent_transform: Handle<Datum3>,
    /// `myChildren`.
    pub children: Vec<Handle<dyn PrsMgrPresentableObject>>,
    /// `myInvTransformation`.
    pub inv_transformation: GeneralTransform3,
    /// `myTypeOfPresentation3d`.
    pub type_of_presentation3d: PrsMgrTypeOfPresentation3d,
    /// `myDisplayStatus`.
    pub display_status: PrsMgrDisplayStatus,
    /// `myCurrentFacingModel`.
    pub current_facing_model: AspectTypeOfFacingModel,
    /// `myOwnWidth` (a float natively).
    pub own_width: f32,
    /// `hasOwnColor`.
    pub has_own_color: bool,
    /// `hasOwnMaterial`.
    pub has_own_material: bool,
    /// `myInfiniteState`.
    pub infinite_state: bool,
    /// `myIsMutable`.
    pub is_mutable: bool,
    /// `myHasOwnPresentations`.
    pub has_own_presentations: bool,
    /// `myToPropagateVisualState`.
    pub to_propagate_visual_state: bool,
}

/// The state every presentable object embeds.
pub struct PrsMgrPresentableObjectBase {
    /// The protected fields.
    pub fields: SharedState<PrsMgrPresentableObjectFields>,
    this: SharedState<WeakHandle<dyn PrsMgrPresentableObject>>,
}

impl PrsMgrPresentableObjectBase {
    /// The constructor state: a new drawer without display mode, no parent and no
    /// presentation. Example: `PrsMgrPresentableObjectBase::new(PrsMgrTypeOfPresentation3d::AllView)`.
    pub fn new(kind: PrsMgrTypeOfPresentation3d) -> Self {
        let drawer = Prs3dDrawer::new();
        drawer.set_display_mode(-1);
        let fields = PrsMgrPresentableObjectFields {
            parent: WeakHandle::default(),
            presentations: Vec::new(),
            view_affinity: Handle::new(Graphic3dViewAffinity::default()),
            clip_planes: Handle::default(),
            drawer: Handle::new(drawer),
            hilight_drawer: Handle::default(),
            dyn_hilight_drawer: Handle::default(),
            transform_persistence: Handle::default(),
            local_transformation: Handle::default(),
            transformation: Handle::default(),
            combined_parent_transform: Handle::default(),
            children: Vec::new(),
            inv_transformation: GeneralTransform3::from(Transform3::default()),
            type_of_presentation3d: kind,
            display_status: PrsMgrDisplayStatus::None,
            current_facing_model: AspectTypeOfFacingModel::BothSide,
            own_width: 0.0,
            has_own_color: false,
            has_own_material: false,
            infinite_state: false,
            is_mutable: false,
            has_own_presentations: true,
            to_propagate_visual_state: true,
        };
        Self {
            fields: SharedState::new(fields),
            this: SharedState::new(WeakHandle::default()),
        }
    }
    /// A copy of the fields; `base.read()`.
    pub fn read(&self) -> PrsMgrPresentableObjectFields {
        self.fields.read().clone()
    }
    /// The handle of the object, null before [`prs_mgr_presentable_bind`]; `base.this()`.
    pub fn this(&self) -> Handle<dyn PrsMgrPresentableObject> {
        self.this.read().upgrade()
    }
}

impl Drop for PrsMgrPresentableObjectBase {
    /// The destructor erases the presentations and detaches the children.
    fn drop(&mut self) {
        let fields = self.fields.read().clone();
        for prs in fields.presentations.iter().filter_map(Handle::get) {
            prs.erase();
            prs.forget_presentable_object();
        }
        for child in fields.children.iter().filter_map(Handle::get) {
            // An identity combined transformation cannot fail to invert.
            let _ = child.set_combined_parent_transform(&Handle::default());
            child.presentable_base().fields.write().parent = WeakHandle::default();
        }
    }
}

/// Records `object` as its own handle, which its presentations and children receive; every
/// constructor of a presentable class calls it once the handle exists.
/// Example: `prs_mgr_presentable_bind(&handle.clone().cast())`.
pub fn prs_mgr_presentable_bind(object: &Handle<dyn PrsMgrPresentableObject>) {
    if let Some(value) = object.get() {
        *value.presentable_base().this.write() = object.downgrade();
    }
}

/// An object that a presentation manager displays; the default bodies are the native ones.
pub trait PrsMgrPresentableObject: Transient {
    /// The embedded base; `object.presentable_base()`.
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase;
    /// `Compute` (pure virtual): fills the presentation of a display mode.
    /// Example: `object.compute(&manager, &prs, 0)?`.
    fn compute(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure>;
    /// `SetZLayer`: the drawer layer and the layer of every presentation; `object.set_z_layer(id)`.
    fn set_z_layer(&self, layer_id: Graphic3dZLayerId) {
        base::prs_mgr_set_z_layer(self, layer_id);
    }
    /// `SetMutable`: the mutable flag of the object and its presentations; `object.set_mutable(true)`.
    fn set_mutable(&self, is_mutable: bool) {
        base::prs_mgr_set_mutable(self, is_mutable);
    }
    /// `AcceptDisplayMode`: every mode by default; `object.accept_display_mode(1)`.
    fn accept_display_mode(&self, mode: i32) -> bool {
        let _ = mode;
        true
    }
    /// `DefaultDisplayMode`: 0 by default; `object.default_display_mode()`.
    fn default_display_mode(&self) -> i32 {
        0
    }
    /// `SetAttributes`: replaces the drawer; `object.set_attributes(&drawer)`.
    fn set_attributes(&self, drawer: &Handle<Prs3dDrawer>) {
        self.presentable_base().fields.write().drawer = drawer.clone();
    }
    /// `SetHilightAttributes`; `object.set_hilight_attributes(&drawer)`.
    fn set_hilight_attributes(&self, drawer: &Handle<Prs3dDrawer>) {
        self.presentable_base().fields.write().hilight_drawer = drawer.clone();
    }
    /// `SetDynamicHilightAttributes`; `object.set_dynamic_hilight_attributes(&drawer)`.
    fn set_dynamic_hilight_attributes(&self, drawer: &Handle<Prs3dDrawer>) {
        self.presentable_base().fields.write().dyn_hilight_drawer = drawer.clone();
    }
    /// `UnsetHilightAttributes`: forgets the highlight drawer; `object.unset_hilight_attributes()`.
    fn unset_hilight_attributes(&self) {
        self.presentable_base().fields.write().hilight_drawer = Handle::default();
    }
    /// `SetTransformPersistence`: the persistence of the object and of its presentations,
    /// recomputed. Example: `object.set_transform_persistence(&pers)`.
    fn set_transform_persistence(&self, trsf_pers: &Handle<dyn Graphic3dTransformPersSource>) {
        base::prs_mgr_set_transform_persistence(self, trsf_pers);
    }
    /// `ResetTransformation`: removes the local transformation; `object.reset_transformation()?`.
    fn reset_transformation(&self) -> Result<(), Failure> {
        self.set_local_transformation_datum(&Handle::default())
    }
    /// `UpdateTransformation`: combines the parent and local transformations and passes the
    /// result to the presentations and children; a singular transformation raises
    /// `Standard_ConstructionError`. Example: `object.update_transformation()?`.
    fn update_transformation(&self) -> Result<(), Failure> {
        base::prs_mgr_update_transformation(self)
    }
    /// `RecomputeTransformation`: nothing by default; `object.recompute_transformation(&camera)`.
    fn recompute_transformation(&self, projector: &Handle<Graphic3dCamera>) {
        let _ = projector;
    }
    /// `SetClipPlanes`: the object planes, passed to the presentations; `object.set_clip_planes(&planes)`.
    fn set_clip_planes(&self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
        self.presentable_base().fields.write().clip_planes = planes.clone();
        self.update_clipping();
    }
    /// `AddClipPlane`; `object.add_clip_plane(&plane)`.
    fn add_clip_plane(&self, plane: &Handle<Graphic3dClipPlane>) {
        base::prs_mgr_add_clip_plane(self, plane);
    }
    /// `RemoveClipPlane`; `object.remove_clip_plane(&plane)`.
    fn remove_clip_plane(&self, plane: &Handle<Graphic3dClipPlane>) {
        base::prs_mgr_remove_clip_plane(self, plane);
    }
    /// `AddChild`: detaches the child from its parent and attaches it here.
    /// Example: `parent.add_child(&child)?`.
    fn add_child(&self, object: &Handle<dyn PrsMgrPresentableObject>) -> Result<(), Failure> {
        base::prs_mgr_add_child(self, object)
    }
    /// `RemoveChild`: detaches a child; `parent.remove_child(&child)?`.
    fn remove_child(&self, object: &Handle<dyn PrsMgrPresentableObject>) -> Result<(), Failure> {
        base::prs_mgr_remove_child(self, object)
    }
    /// `BoundingBox`: the box of the presentation of the display mode, or of the first
    /// presentation or the children without mode. Example: `object.bounding_box(&mut bounds)`.
    fn bounding_box(&self, bounds: &mut BoundingBox3) {
        style::prs_mgr_bounding_box(self, bounds);
    }
    /// `Fill`: computes the presentation and gives it the transformation, clipping and
    /// persistence of the object. Example: `object.fill(&manager, &prs, 0)?`.
    fn fill(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<PrsMgrPresentation>,
        mode: i32,
    ) -> Result<(), Failure> {
        base::prs_mgr_fill(self, manager, prs, mode)
    }
    /// `computeHLR`: raises `Standard_NotImplemented` by default.
    /// Example: `object.compute_hlr(&camera, &trsf, &prs)?`.
    fn compute_hlr(
        &self,
        projector: &Handle<Graphic3dCamera>,
        trsf: &Handle<Datum3>,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let _ = (projector, trsf, prs);
        Err(rustycad_tkernel::standard::exceptions::not_implemented(
            "cannot compute under a specific projector",
        ))
    }
    /// `UpdateClipping`: passes the object planes to the presentations; `object.update_clipping()`.
    fn update_clipping(&self) {
        base::prs_mgr_update_clipping(self);
    }
    /// `SetCombinedParentTransform`; `object.set_combined_parent_transform(&trsf)?`.
    fn set_combined_parent_transform(&self, trsf: &Handle<Datum3>) -> Result<(), Failure> {
        self.presentable_base()
            .fields
            .write()
            .combined_parent_transform = trsf.clone();
        self.update_transformation()
    }
    /// `setLocalTransformation`: the local transformation, then `UpdateTransformation`.
    /// Example: `object.set_local_transformation_datum(&trsf)?`.
    fn set_local_transformation_datum(&self, trsf: &Handle<Datum3>) -> Result<(), Failure> {
        self.presentable_base().fields.write().local_transformation = trsf.clone();
        self.update_transformation()
    }
    /// `Color`: the drawer color; `object.color()`.
    fn color(&self) -> Color {
        self.presentable_base().read().drawer.color()
    }
    /// `SetColor`: the drawer color, now own; `object.set_color(red)?`.
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        let drawer = self.presentable_base().read().drawer;
        drawer.set_color(color);
        self.presentable_base().fields.write().has_own_color = true;
        Ok(())
    }
    /// `UnsetColor`; `object.unset_color()?`.
    fn unset_color(&self) -> Result<(), Failure> {
        self.presentable_base().fields.write().has_own_color = false;
        Ok(())
    }
    /// `SetWidth`: the own width (stored as a float); `object.set_width(2.0)?`.
    fn set_width(&self, width: f64) -> Result<(), Failure> {
        self.presentable_base().fields.write().own_width = width as f32;
        Ok(())
    }
    /// `UnsetWidth`; `object.unset_width()?`.
    fn unset_width(&self) -> Result<(), Failure> {
        self.presentable_base().fields.write().own_width = 0.0;
        Ok(())
    }
    /// `Material`: the name of the front shading material; `object.material()`.
    fn material(&self) -> Graphic3dNameOfMaterial {
        style::prs_mgr_material(self)
    }
    /// `SetMaterial`: an own shading aspect with the material; `object.set_material(&material)?`.
    fn set_material(&self, material: &Graphic3dMaterialAspect) -> Result<(), Failure> {
        style::prs_mgr_set_material(self, material)
    }
    /// `UnsetMaterial`; `object.unset_material()?`.
    fn unset_material(&self) -> Result<(), Failure> {
        style::prs_mgr_unset_material(self)
    }
    /// `Transparency`: the drawer transparency, 0 at or below 0.005; `object.transparency()`.
    fn transparency(&self) -> f64 {
        let value = self.presentable_base().read().drawer.transparency();
        if value <= 0.005 {
            0.0
        } else {
            f64::from(value)
        }
    }
    /// `SetTransparency`: an own shading aspect with both materials transparent.
    /// Example: `object.set_transparency(0.6)?`.
    fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        style::prs_mgr_set_transparency(self, value)
    }
    /// `UnsetTransparency`; `object.unset_transparency()?`.
    fn unset_transparency(&self) -> Result<(), Failure> {
        style::prs_mgr_unset_transparency(self)
    }
    /// `HasPolygonOffsets`; `object.has_polygon_offsets()`.
    fn has_polygon_offsets(&self) -> bool {
        style::prs_mgr_has_polygon_offsets(self)
    }
    /// `PolygonOffsets`: `(mode, factor, units)` when the object has polygon offsets (the
    /// native out arguments are untouched otherwise). Example: `object.polygon_offsets()`.
    fn polygon_offsets(&self) -> Option<(i32, f32, f32)> {
        style::prs_mgr_polygon_offsets(self)
    }
    /// `SetPolygonOffsets`; `object.set_polygon_offsets(1, 1.0, 0.0)?`.
    fn set_polygon_offsets(&self, mode: i32, factor: f32, units: f32) -> Result<(), Failure> {
        style::prs_mgr_set_polygon_offsets(self, mode, factor, units)
    }
    /// `UnsetAttributes`: a new drawer keeping the link; `object.unset_attributes()`.
    fn unset_attributes(&self) {
        style::prs_mgr_unset_attributes(self);
    }
    /// `DumpJson`; `object.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        super::presentable_dump::prs_mgr_presentable_dump_json(self, stream, depth)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn PrsMgrPresentableObject,
    "PrsMgr_PresentableObject",
    StandardTransient
);
