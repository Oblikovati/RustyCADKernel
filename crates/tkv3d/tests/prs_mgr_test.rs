//! PrsMgr behaviour on the TKService null driver: presentations per mode, display and erase,
//! highlight and immediate mode, transformations of children, clipping, style members and
//! update flags. `SegmentObject` is the named fake presentable object.
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectWindow};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOPMOST, Graphic3dArrayOfSegments, Graphic3dClipPlane,
    Graphic3dGroupMembers as _, Graphic3dMaterialAspect, Graphic3dNameOfMaterial,
    Graphic3dStructureManager, Graphic3dStructureMembers as _, Graphic3dStructureSource,
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::prs_mgr::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectBase, PrsMgrPresentableObjectMembers as _,
    PrsMgrPresentableObjectUpdates as _, PrsMgrPresentationManager, PrsMgrTypeOfPresentation3d,
    prs_mgr_presentable_bind,
};
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::v3d::V3dViewSource;
use rustycad_tkv3d::v3d::V3dViewer;

/// A presentable object drawing one segment from the origin to (length, 0, 0) per mode; it
/// accepts the modes 0 and 1 only.
struct SegmentObject {
    transient: StandardTransient,
    base: PrsMgrPresentableObjectBase,
    length: f64,
}
rustycad_tkernel::impl_transient!(SegmentObject, "Test_SegmentObject", StandardTransient, transient,
    interfaces: [dyn PrsMgrPresentableObject]);

impl PrsMgrPresentableObject for SegmentObject {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base
    }
    fn compute(
        &self,
        _manager: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        let Some(prs) = prs.get() else {
            return Ok(());
        };
        let segments = Graphic3dArrayOfSegments::new(2, 0, 0)?;
        segments.add_vertex_f64(0.0, 0.0, 0.0)?;
        segments.add_vertex_f64(self.length * f64::from(mode + 1), 0.0, 0.0)?;
        prs.new_group()?
            .add_primitive_array(&Handle::new(segments).cast(), true);
        Ok(())
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0 || mode == 1
    }
}

fn segment(length: f64) -> Handle<dyn PrsMgrPresentableObject> {
    let object: Handle<dyn PrsMgrPresentableObject> = Handle::new(SegmentObject {
        transient: StandardTransient::default(),
        base: PrsMgrPresentableObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
        length,
    })
    .cast();
    prs_mgr_presentable_bind(&object);
    object
}

fn manager() -> Handle<PrsMgrPresentationManager> {
    let structures =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    PrsMgrPresentationManager::new(&structures)
}

#[test]
fn display_creates_one_presentation_per_mode_and_erase_forgets_it() {
    let (manager, object) = (manager(), segment(2.0));
    manager.display(&object, 0).unwrap();
    manager.display(&object, 1).unwrap();
    assert_eq!(object.presentations().len(), 2);
    assert!(manager.is_displayed(&object, 0) && manager.has_presentation(&object, 1));
    manager.erase(&object, 0);
    assert!(!manager.is_displayed(&object, 0));
    assert_eq!(object.presentations().len(), 1);
    manager.erase(&object, -1);
    assert!(object.presentations().is_empty());
}

#[test]
fn display_mode_is_set_only_when_accepted() {
    let object = segment(1.0);
    assert!(!object.has_display_mode());
    object.set_display_mode(5);
    assert_eq!(object.display_mode(), -1);
    object.set_display_mode(1);
    assert_eq!(object.display_mode(), 1);
    object.unset_display_mode();
    assert!(!object.has_display_mode());
}

#[test]
fn hilight_mode_creates_both_highlight_drawers() {
    let object = segment(1.0);
    assert_eq!(object.hilight_mode(), -1);
    object.set_hilight_mode(1);
    assert!(object.has_hilight_mode());
    let gray = Color::named(NamedColor::GRAY80);
    assert_eq!(
        object
            .hilight_attributes()
            .get()
            .map(rustycad_tkservice::graphic3d::Graphic3dPresentationAttributesMembers::color),
        Some(gray)
    );
    assert!(!object.dynamic_hilight_attributes().is_null());
    object.unset_hilight_mode();
    assert!(!object.has_hilight_mode());
}

/// A manager of a viewer with one active view, so displayed structures get their bounds
/// (natively the view computes the box when it displays a structure).
fn manager_with_view() -> (
    Handle<V3dViewer>,
    Handle<dyn V3dViewSource>,
    Handle<PrsMgrPresentationManager>,
) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let window = AspectNeutralWindow::default();
    window.set_size(300, 200);
    let window: Handle<dyn AspectWindow> = Handle::new(window).cast();
    view.set_window(&window, 0).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    (viewer, view, manager)
}

#[test]
fn bounding_box_is_void_without_a_view_displaying_the_structure() {
    let (manager, object) = (manager(), segment(3.0));
    object.set_display_mode(1);
    manager.display(&object, 1).unwrap();
    let mut bounds = BoundingBox3::default();
    object.bounding_box(&mut bounds);
    assert!(bounds.is_void());
}

#[test]
fn bounding_box_is_the_box_of_the_display_mode_presentation() {
    let (_viewer, _view, manager) = manager_with_view();
    let object = segment(3.0);
    object.set_display_mode(1);
    manager.display(&object, 1).unwrap();
    let mut bounds = BoundingBox3::default();
    object.bounding_box(&mut bounds);
    let [x0, _, _, x1, _, _] = bounds.get().unwrap();
    assert_eq!((x0, x1), (0.0, 6.0));
}

#[test]
fn children_combine_the_parent_transformation() {
    let (parent, child) = (segment(1.0), segment(1.0));
    let mut shift = Transform3::default();
    shift.set_translation(Vector3::new(10.0, 0.0, 0.0));
    parent.set_local_transformation(&shift).unwrap();
    parent.add_child(&child).unwrap();
    assert!(child.has_transformation());
    assert_eq!(child.transformation().translation_part().x(), 10.0);
    assert!(child.parent().object_address() == parent.object_address());
    parent
        .remove_child_with_restore_transformation(&child)
        .unwrap();
    assert!(child.parent().is_null());
    assert_eq!(child.local_transformation().translation_part().x(), 10.0);
    let point = Point3::new(1.0, 0.0, 0.0).transformed(child.transformation());
    assert_eq!(point.x(), 11.0);
}

#[test]
fn clip_planes_reach_the_presentations() {
    let (manager, object) = (manager(), segment(1.0));
    manager.display(&object, 0).unwrap();
    let plane = Handle::new(Graphic3dClipPlane::default());
    object.add_clip_plane(&plane);
    let prs = object.presentations()[0].clone();
    assert_eq!(prs.clip_planes().get().map(|p| p.items().len()), Some(1));
    object.remove_clip_plane(&plane);
    assert_eq!(prs.clip_planes().get().map(|p| p.items().len()), Some(0));
}

#[test]
fn color_highlights_and_unhighlight_restores_the_hidden_state() {
    let (manager, object) = (manager(), segment(1.0));
    let style = Handle::new(Prs3dDrawer::new());
    manager
        .color(
            &object,
            &style,
            0,
            &Handle::default(),
            GRAPHIC3D_ZLAYER_ID_TOPMOST,
        )
        .unwrap();
    assert!(manager.is_highlighted(&object, 0));
    manager.unhighlight(&object);
    assert!(!manager.is_highlighted(&object, 0));
    assert!(!manager.is_displayed(&object, 0));
}

#[test]
fn immediate_mode_queues_highlight_shadows() {
    let (manager, object) = (manager(), segment(1.0));
    manager.begin_immediate_draw();
    assert!(manager.is_immediate_mode_on());
    let style = Handle::new(Prs3dDrawer::new());
    manager
        .color(
            &object,
            &style,
            0,
            &Handle::default(),
            GRAPHIC3D_ZLAYER_ID_TOPMOST,
        )
        .unwrap();
    assert!(!manager.is_highlighted(&object, 0));
    manager.clear_immediate_draw();
    assert!(manager.is_immediate_mode_on());
}

#[test]
fn transparency_and_material_follow_the_native_flags() {
    let object = segment(1.0);
    object.set_transparency(0.6).unwrap();
    assert!(object.is_transparent());
    assert!((object.transparency() - 0.6).abs() < 1e-6);
    object.unset_transparency().unwrap();
    assert!(!object.is_transparent());
    object
        .set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))
        .unwrap();
    assert!(object.has_material());
    assert_eq!(object.material(), Graphic3dNameOfMaterial::Gold);
    object.unset_material().unwrap();
    assert!(!object.has_material());
    // No own shading aspect and no link: natively HasPolygonOffsets is true, and
    // PolygonOffsets would read a null shading aspect.
    assert!(object.has_polygon_offsets());
    assert_eq!(object.polygon_offsets(), None);
    object.set_polygon_offsets(1, 2.0, 3.0).unwrap();
    assert!(!object.has_polygon_offsets());
}

#[test]
fn update_flags_and_update_presentations() {
    let (manager, object) = (manager(), segment(1.0));
    manager.display(&object, 0).unwrap();
    assert!(!object.to_be_updated(false));
    object.set_to_update_all();
    assert!(object.to_be_updated(false));
    assert_eq!(object.to_be_updated_modes(), vec![0]);
    assert!(object.update_presentations(false).unwrap());
    assert!(!object.to_be_updated(true));
}
