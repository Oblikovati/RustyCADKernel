//! The non-virtual members of `PrsMgr_PresentableObject`, for every presentable object.
use super::presentable_defaults::is_non_identity;
use super::{
    PrsMgrDisplayStatus, PrsMgrPresentableObject, PrsMgrPresentableObjectBase, PrsMgrPresentation,
    PrsMgrTypeOfPresentation3d,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{GeneralTransform3, Transform3};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::aspect::AspectTypeOfFacingModel;
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOP, GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dGroup,
    Graphic3dPresentationAttributesMembers as _, Graphic3dPresentationAttributesSource as _,
    Graphic3dSequenceOfHClipPlane, Graphic3dStructureMembers as _, Graphic3dStructureSource as _,
    Graphic3dTransformPersSource, Graphic3dTypeOfStructure, Graphic3dViewAffinity,
    Graphic3dZLayerId,
};

/// A highlight drawer linked to `drawer` without auto triangulation (`SetHilightMode`).
fn highlight_drawer(
    drawer: &Handle<Prs3dDrawer>,
    color: NamedColor,
    layer: Graphic3dZLayerId,
) -> Handle<Prs3dDrawer> {
    let highlight = Prs3dDrawer::new();
    highlight.set_link(drawer);
    highlight.set_auto_triangulation(false);
    highlight.set_color(Color::named(color));
    highlight.set_z_layer(layer);
    Handle::new(highlight)
}

/// The non-virtual members of a presentable object.
pub trait PrsMgrPresentableObjectMembers: PrsMgrPresentableObject {
    /// `Presentations`: the presentations of the object; `object.presentations()`.
    fn presentations(&self) -> Vec<Handle<PrsMgrPresentation>> {
        self.presentable_base().read().presentations
    }
    /// `ZLayer`: the layer of the drawer; `object.z_layer()`.
    fn z_layer(&self) -> Graphic3dZLayerId {
        self.attributes().z_layer()
    }
    /// `IsMutable`; `object.is_mutable()`.
    fn is_mutable(&self) -> bool {
        self.presentable_base().read().is_mutable
    }
    /// `ViewAffinity`; `object.view_affinity()`.
    fn view_affinity(&self) -> Handle<Graphic3dViewAffinity> {
        self.presentable_base().read().view_affinity
    }
    /// `HasDisplayMode`: a display mode other than -1; `object.has_display_mode()`.
    fn has_display_mode(&self) -> bool {
        self.display_mode() != -1
    }
    /// `DisplayMode`: the drawer display mode; `object.display_mode()`.
    fn display_mode(&self) -> i32 {
        self.attributes().display_mode()
    }
    /// `SetDisplayMode`: the drawer display mode when the object accepts it; `object.set_display_mode(1)`.
    fn set_display_mode(&self, mode: i32) {
        if self.accept_display_mode(mode) {
            self.attributes().set_display_mode(mode);
        }
    }
    /// `UnsetDisplayMode`; `object.unset_display_mode()`.
    fn unset_display_mode(&self) {
        self.attributes().set_display_mode(-1);
    }
    /// `HasHilightMode`; `object.has_hilight_mode()`.
    fn has_hilight_mode(&self) -> bool {
        self.hilight_attributes()
            .get()
            .is_some_and(|d| d.display_mode() != -1)
    }
    /// `HilightMode`: -1 without highlight drawer; `object.hilight_mode()`.
    fn hilight_mode(&self) -> i32 {
        self.hilight_attributes()
            .get()
            .map_or(-1, |d| d.display_mode())
    }
    /// `SetHilightMode`: creates the gray-80 and cyan highlight drawers on demand.
    /// Example: `object.set_hilight_mode(1)`.
    fn set_hilight_mode(&self, mode: i32) {
        prs_mgr_initialize_hilight_mode(self.presentable_base(), mode);
    }
    /// `UnsetHilightMode`; `object.unset_hilight_mode()`.
    fn unset_hilight_mode(&self) {
        let fields = self.presentable_base().read();
        for drawer in [fields.hilight_drawer, fields.dyn_hilight_drawer]
            .iter()
            .filter_map(Handle::get)
        {
            drawer.set_display_mode(-1);
        }
    }
    /// `ToBeUpdated(toIncludeHidden)`: a presentation to update that is displayed or
    /// highlighted (or hidden too when asked). Example: `object.to_be_updated(false)`.
    fn to_be_updated(&self, to_include_hidden: bool) -> bool {
        let this = self.presentable_base().this();
        self.presentations()
            .iter()
            .filter_map(Handle::get)
            .any(|prs| {
                prs.must_be_updated() && (to_include_hidden || prs.is_shown_by_manager(&this))
            })
    }
    /// `SetToUpdate(mode)`: flags the presentations of a mode (-1 for all); `object.set_to_update(0)`.
    fn set_to_update(&self, mode: i32) {
        for prs in self.presentations().iter().filter_map(Handle::get) {
            if mode == -1 || prs.mode() == mode {
                prs.set_update_status(true);
            }
        }
    }
    /// `SetToUpdate()`: flags every presentation; `object.set_to_update_all()`.
    fn set_to_update_all(&self) {
        self.set_to_update(-1);
    }
    /// `IsInfinite`; `object.is_infinite()`.
    fn is_infinite(&self) -> bool {
        self.presentable_base().read().infinite_state
    }
    /// `SetInfiniteState`: the flag of the object and its presentations; `object.set_infinite_state(true)`.
    fn set_infinite_state(&self, flag: bool) {
        {
            let mut fields = self.presentable_base().fields.write();
            if fields.infinite_state == flag {
                return;
            }
            fields.infinite_state = flag;
        }
        self.presentations()
            .iter()
            .filter_map(Handle::get)
            .for_each(|p| p.set_infinite_state(flag));
    }
    /// `TypeOfPresentation3d`; `object.type_of_presentation3d()`.
    fn type_of_presentation3d(&self) -> PrsMgrTypeOfPresentation3d {
        self.presentable_base().read().type_of_presentation3d
    }
    /// `SetTypeOfPresentation`: the type and the visual of the presentations.
    /// Example: `object.set_type_of_presentation(PrsMgrTypeOfPresentation3d::ProjectorDependent)`.
    fn set_type_of_presentation(&self, kind: PrsMgrTypeOfPresentation3d) {
        self.presentable_base()
            .fields
            .write()
            .type_of_presentation3d = kind;
        let visual = if kind == PrsMgrTypeOfPresentation3d::ProjectorDependent {
            Graphic3dTypeOfStructure::Computed
        } else {
            Graphic3dTypeOfStructure::All
        };
        self.presentations()
            .iter()
            .filter_map(Handle::get)
            .for_each(|p| p.set_visual(visual));
    }
    /// `DisplayStatus`; `object.display_status()`.
    fn display_status(&self) -> PrsMgrDisplayStatus {
        self.presentable_base().read().display_status
    }
    /// `Attributes`: the drawer; `object.attributes()`.
    fn attributes(&self) -> Handle<Prs3dDrawer> {
        self.presentable_base().read().drawer
    }
    /// `HilightAttributes`; `object.hilight_attributes()`.
    fn hilight_attributes(&self) -> Handle<Prs3dDrawer> {
        self.presentable_base().read().hilight_drawer
    }
    /// `DynamicHilightAttributes`; `object.dynamic_hilight_attributes()`.
    fn dynamic_hilight_attributes(&self) -> Handle<Prs3dDrawer> {
        self.presentable_base().read().dyn_hilight_drawer
    }
    /// `SynchronizeAspects`: the groups of every presentation resynchronize their aspects.
    /// Example: `object.synchronize_aspects()`.
    fn synchronize_aspects(&self) {
        for group in self.all_groups() {
            group.synchronize_aspects();
        }
    }
    /// The groups of every presentation, in order (`SynchronizeAspects`, `replaceAspects`).
    fn all_groups(&self) -> Vec<Handle<dyn Graphic3dGroup>> {
        let presentations = self.presentations();
        presentations
            .iter()
            .filter_map(Handle::get)
            .flat_map(|p| p.groups())
            .filter(|g| !g.is_null())
            .collect()
    }
    /// `TransformPersistence`; `object.transform_persistence()`.
    fn transform_persistence(&self) -> Handle<dyn Graphic3dTransformPersSource> {
        self.presentable_base().read().transform_persistence
    }
    /// `LocalTransformationGeom`; `object.local_transformation_geom()`.
    fn local_transformation_geom(&self) -> Handle<Datum3> {
        self.presentable_base().read().local_transformation
    }
    /// `SetLocalTransformation(gp_Trsf)`; `object.set_local_transformation(&trsf)?`.
    fn set_local_transformation(&self, trsf: &Transform3) -> Result<(), Failure> {
        self.set_local_transformation_datum(&Handle::new(Datum3::new(*trsf)))
    }
    /// `HasTransformation`: a non-identity absolute transformation; `object.has_transformation()`.
    fn has_transformation(&self) -> bool {
        is_non_identity(&self.presentable_base().read().transformation)
    }
    /// `TransformationGeom`; `object.transformation_geom()`.
    fn transformation_geom(&self) -> Handle<Datum3> {
        self.presentable_base().read().transformation
    }
    /// `LocalTransformation`: the identity without local transformation; `object.local_transformation()`.
    fn local_transformation(&self) -> Transform3 {
        self.local_transformation_geom()
            .get()
            .map_or_else(Transform3::default, |t| *t.trsf())
    }
    /// `Transformation`: the identity without transformation; `object.transformation()`.
    fn transformation(&self) -> Transform3 {
        self.transformation_geom()
            .get()
            .map_or_else(Transform3::default, |t| *t.trsf())
    }
    /// `InversedTransformation`; `object.inversed_transformation()`.
    fn inversed_transformation(&self) -> GeneralTransform3 {
        self.presentable_base().read().inv_transformation
    }
    /// `CombinedParentTransformation`; `object.combined_parent_transformation()`.
    fn combined_parent_transformation(&self) -> Handle<Datum3> {
        self.presentable_base().read().combined_parent_transform
    }
    /// `ClipPlanes`; `object.clip_planes()`.
    fn clip_planes(&self) -> Handle<Graphic3dSequenceOfHClipPlane> {
        self.presentable_base().read().clip_planes
    }
    /// `Parent`: the parent, null without; `object.parent()`.
    fn parent(&self) -> Handle<dyn PrsMgrPresentableObject> {
        self.presentable_base().read().parent.upgrade()
    }
    /// `Children`; `object.children()`.
    fn children(&self) -> Vec<Handle<dyn PrsMgrPresentableObject>> {
        self.presentable_base().read().children
    }
    /// `AddChildWithCurrentTransformation`: keeps the child where it is.
    /// Example: `parent.add_child_with_current_transformation(&child)?`.
    fn add_child_with_current_transformation(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
    ) -> Result<(), Failure> {
        let Some(child) = object.get() else {
            return Ok(());
        };
        let trsf = self
            .transformation()
            .inverted()?
            .multiplied(child.transformation());
        child.set_local_transformation(&trsf)?;
        self.add_child(object)
    }
    /// `RemoveChildWithRestoreTransformation`: the child keeps its absolute transformation.
    /// Example: `parent.remove_child_with_restore_transformation(&child)?`.
    fn remove_child_with_restore_transformation(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
    ) -> Result<(), Failure> {
        let Some(child) = object.get() else {
            return Ok(());
        };
        let trsf = child.transformation();
        self.remove_child(object)?;
        child.set_local_transformation(&trsf)
    }
    /// `HasOwnPresentations`; `object.has_own_presentations()`.
    fn has_own_presentations(&self) -> bool {
        self.presentable_base().read().has_own_presentations
    }
    /// `CurrentFacingModel`; `object.current_facing_model()`.
    fn current_facing_model(&self) -> AspectTypeOfFacingModel {
        self.presentable_base().read().current_facing_model
    }
    /// `SetCurrentFacingModel`; `object.set_current_facing_model(AspectTypeOfFacingModel::BothSide)`.
    fn set_current_facing_model(&self, model: AspectTypeOfFacingModel) {
        self.presentable_base().fields.write().current_facing_model = model;
    }
    /// `SetIsoOnTriangulation`: the drawer flag; `object.set_iso_on_triangulation(true)`.
    fn set_iso_on_triangulation(&self, is_enabled: bool) {
        self.attributes().set_iso_on_triangulation(is_enabled);
    }
    /// `HasColor`; `object.has_color()`.
    fn has_color(&self) -> bool {
        self.presentable_base().read().has_own_color
    }
    /// `HasWidth`; `object.has_width()`.
    fn has_width(&self) -> bool {
        self.presentable_base().read().own_width != 0.0
    }
    /// `Width`; `object.width()`.
    fn width(&self) -> f64 {
        f64::from(self.presentable_base().read().own_width)
    }
    /// `HasMaterial`; `object.has_material()`.
    fn has_material(&self) -> bool {
        self.presentable_base().read().has_own_material
    }
    /// `IsTransparent`: a drawer transparency above 0.005; `object.is_transparent()`.
    fn is_transparent(&self) -> bool {
        self.attributes().transparency() > 0.005
    }
    /// `ToPropagateVisualState`; `object.to_propagate_visual_state()`.
    fn to_propagate_visual_state(&self) -> bool {
        self.presentable_base().read().to_propagate_visual_state
    }
    /// `SetPropagateVisualState`; `object.set_propagate_visual_state(false)`.
    fn set_propagate_visual_state(&self, flag: bool) {
        self.presentable_base()
            .fields
            .write()
            .to_propagate_visual_state = flag;
    }
}
impl<T: PrsMgrPresentableObject + ?Sized> PrsMgrPresentableObjectMembers for T {}

/// `getIdentityTrsf` (protected static): the identity transformation; `prs_mgr_identity_trsf()`.
pub fn prs_mgr_identity_trsf() -> Transform3 {
    Transform3::default()
}

/// Reuse native highlight initialization while constructing subclass base storage.
pub(crate) fn prs_mgr_initialize_hilight_mode(base: &PrsMgrPresentableObjectBase, mode: i32) {
    let fields = base.read();
    let mut highlight = fields.hilight_drawer.clone();
    let mut dynamic = fields.dyn_hilight_drawer.clone();
    if highlight.is_null() {
        highlight = highlight_drawer(
            &fields.drawer,
            NamedColor::GRAY80,
            GRAPHIC3D_ZLAYER_ID_UNKNOWN,
        );
    }
    if dynamic.is_null() {
        dynamic = highlight_drawer(&fields.drawer, NamedColor::CYAN1, GRAPHIC3D_ZLAYER_ID_TOP);
    }
    highlight.set_display_mode(mode);
    dynamic.set_display_mode(mode);
    let mut own = base.fields.write();
    (own.hilight_drawer, own.dyn_hilight_drawer) = (highlight, dynamic);
}
