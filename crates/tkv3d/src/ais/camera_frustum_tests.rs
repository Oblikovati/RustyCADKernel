//! Rust-only replay of the committed local OCCT camera-frustum oracle.
use super::*;
use crate::{prs_mgr::*, select_mgr::*, select3d::*};
use rustycad_tkernel::{
    quantity::{Color, ColorSpace, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{
    aspect::AspectTypeOfFacingModel, graphic3d::*, null_driver::NullGraphicDriver,
};
const FIXTURE: &str = include_str!("../../tests/fixtures/camera-frustum-native.txt");
fn rows(kind: &str) -> impl Iterator<Item = Vec<f64>> + '_ {
    FIXTURE
        .lines()
        .filter(move |line| line.split_whitespace().next() == Some(kind))
        .map(|line| {
            line.split_whitespace()
                .skip(1)
                .map(|field| field.parse().unwrap())
                .collect()
        })
}
fn assert_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (&a, &b) in actual.iter().zip(expected) {
        assert!(
            (a - b).abs() <= 2.0e-12 * b.abs().max(1.0),
            "actual {a:?}, expected {b:?}"
        );
    }
}
fn camera(id: usize) -> Handle<Graphic3dCamera> {
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(frustum_projection(id));
    camera.set_aspect(if id % 2 == 1 { 1.7 } else { 1.0 });
    camera.set_scale(if id % 2 == 1 { 24.0 } else { 8.0 });
    camera
        .set_z_range(
            if !id.is_multiple_of(3) { 1.0 } else { 0.125 },
            if !id.is_multiple_of(3) { 100.0 } else { 250.0 },
        )
        .unwrap();
    camera.set_fov_y(if id % 2 == 1 { 55.0 } else { 30.0 });
    orient_camera(&camera, id);
    camera
}
#[test]
fn native_frustum_topology_and_corner_narrowing() {
    let object = AisCameraFrustum::new().unwrap();
    for row in rows("topology") {
        let id = row[0] as usize;
        object.set_camera_frustum(&camera(id)).unwrap();
        let fields = object.fields.read();
        let mut actual = vec![
            id as f64,
            fields.triangles.vertex_number() as f64,
            fields.triangles.edge_number() as f64,
            fields.borders.edge_number() as f64,
        ];
        actual.extend((1..=36).map(|i| fields.triangles.edge(i).unwrap() as f64));
        actual.extend((1..=48).map(|i| fields.borders.edge(i).unwrap() as f64));
        assert_eq!(actual, row);
        assert_frustum_corners(&object, id);
    }
}
fn assert_frustum_corners(object: &AisCameraFrustum, id: usize) {
    for row in rows("vertex").filter(|row| row[0] == id as f64) {
        let index = row[1] as usize;
        let fields = object.fields.read();
        let mut actual = fields.points[index].to_vec();
        let triangle = fields.triangles.vertice(index as i32 + 1).unwrap();
        let border = fields.borders.vertice(index as i32 + 1).unwrap();
        actual.extend([
            triangle.x(),
            triangle.y(),
            triangle.z(),
            border.x(),
            border.y(),
            border.z(),
        ]);
        assert_close(&actual, &row[2..]);
    }
}
fn appearance(object: &AisCameraFrustum, stage: f64) -> Vec<f64> {
    let drawer = object.attributes();
    let shading = drawer.shading_aspect();
    let fill = shading.aspect();
    let mut values = vec![
        stage,
        f64::from(object.has_color()),
        drawer.display_mode() as f64,
        f64::from(drawer.transparency()),
        shading.transparency(AspectTypeOfFacingModel::BothSide),
        fill.alpha_mode() as i32 as f64,
        shading.material(AspectTypeOfFacingModel::BothSide).name() as i32 as f64,
    ];
    values.extend(appearance_colors(object));
    values
}
#[test]
fn native_frustum_appearance_and_color_ownership() {
    let object = AisCameraFrustum::new().unwrap();
    for row in rows("appearance") {
        match row[0] as i32 {
            1 | 3 => object.unset_color().unwrap(),
            2 => object
                .set_color(Color::new(0.2, 0.4, 0.6, ColorSpace::Rgb).unwrap())
                .unwrap(),
            4 => object.unset_transparency().unwrap(),
            _ => {}
        }
        assert_close(&appearance(&object, row[0]), &row);
    }
}
fn selection_values(object: &Handle<AisCameraFrustum>, id: f64, mode: i32) -> Vec<f64> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode).unwrap();
    let entities = selection.entities();
    let mut values = vec![id, mode as f64, entities.len() as f64];
    let Some(entity) = entities.first() else {
        return values;
    };
    let sensitive = entity.base_sensitive();
    values.extend(sensitive_values(&sensitive, object));
    values
}
#[test]
fn native_frustum_edge_group_and_volume_selection() {
    let object = AisCameraFrustum::new().unwrap();
    for row in rows("selection") {
        object.set_camera_frustum(&camera(row[0] as usize)).unwrap();
        assert_close(&selection_values(&object, row[0], row[1] as i32), &row);
    }
}
fn presentation() -> Handle<dyn Graphic3dStructureSource> {
    let manager =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    Graphic3dStructure::new(&manager, &Handle::default())
        .unwrap()
        .cast()
}
#[test]
fn native_frustum_empty_and_populated_presentation_routing() {
    for row in rows("presentation") {
        let object = AisCameraFrustum::new().unwrap();
        if row[0] != 0.0 {
            object.set_camera_frustum(&camera(0)).unwrap();
        }
        assert_presentation_row(&object, &row);
    }
}
fn assert_presentation_aspects(
    object: &AisCameraFrustum,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) {
    let groups = prs.groups();
    if groups.is_empty() {
        return;
    }
    let line: Handle<dyn Graphic3dAspectsSource> =
        object.attributes().line_aspect().aspect().cast();
    assert_eq!(groups.last().unwrap().aspects(), line);
    if mode != 1 {
        return;
    }
    let shading: Handle<dyn Graphic3dAspectsSource> =
        object.attributes().shading_aspect().aspect().cast();
    assert_eq!(groups[0].aspects(), shading);
}

#[test]
fn frustum_retains_arrays_but_snapshots_camera_without_ownership() {
    let object = AisCameraFrustum::new().unwrap();
    let camera = camera(1);
    object.set_camera_frustum(&camera).unwrap();
    let (triangles, borders, points) = {
        let f = object.fields.read();
        (f.triangles.clone(), f.borders.clone(), f.points)
    };
    camera.set_scale(99.0);
    assert_eq!(object.fields.read().points, points);
    object.set_camera_frustum(&Handle::default()).unwrap();
    assert_eq!(object.fields.read().points, points);
    object.set_camera_frustum(&camera).unwrap();
    assert_eq!(object.fields.read().triangles, triangles);
    assert_eq!(object.fields.read().borders, borders);
    assert_ne!(object.fields.read().points, points);
    assert_eq!(object.dynamic_type().unwrap().name(), "AIS_CameraFrustum");
    assert_eq!(object.kind_of_interactive(), AisKindOfInteractive::None);
}

fn orient_camera(camera: &Graphic3dCamera, id: usize) {
    if id < 4 {
        return;
    }
    camera.set_eye(&Point3::new(10.0, -20.0, 30.0)).unwrap();
    camera.set_center(&Point3::new(-2.0, 3.0, 1.0)).unwrap();
    camera.set_up(&Direction3::new(0.0, 1.0, 0.0).unwrap());
}

fn appearance_colors(object: &AisCameraFrustum) -> Vec<f64> {
    let drawer = object.attributes();
    let shading = drawer.shading_aspect();
    let mut values = Vec::new();
    for color in [
        drawer.line_aspect().aspect().interior_color(),
        shading.color(AspectTypeOfFacingModel::FrontSide),
        shading.color(AspectTypeOfFacingModel::BackSide),
        drawer.color(),
    ] {
        values.extend([color.red(), color.green(), color.blue()]);
    }
    values
}

fn sensitive_values(
    sensitive: &Handle<dyn Select3DSensitiveEntity>,
    object: &Handle<AisCameraFrustum>,
) -> Vec<f64> {
    let owner = sensitive.owner_id();
    let mut values = vec![
        sensitive.sensitivity_factor() as f64,
        owner.priority() as f64,
        f64::from(owner.selectable() == object.cast::<dyn SelectMgrSelectableObject>()),
        sensitive.nb_sub_elements() as f64,
    ];
    values.extend(sensitive_geometry(sensitive));
    values
}
fn sensitive_geometry(sensitive: &Handle<dyn Select3DSensitiveEntity>) -> [f64; 9] {
    let center = sensitive.center_of_geometry();
    let bounds = sensitive.bounding_box();
    let min = bounds.corner_min();
    let max = bounds.corner_max();
    [
        center.x(),
        center.y(),
        center.z(),
        min.x,
        min.y,
        min.z,
        max.x,
        max.y,
        max.z,
    ]
}

fn assert_presentation_row(object: &AisCameraFrustum, row: &[f64]) {
    let prs = presentation();
    let mode = row[1] as i32;
    object.compute(&Handle::default(), &prs, mode).unwrap();
    assert_eq!(
        [
            row[0],
            row[1],
            f64::from(object.accept_display_mode(mode)),
            f64::from(prs.is_infinite()),
            prs.groups().len() as f64
        ],
        row
    );
    assert_presentation_aspects(object, &prs, mode);
}

fn selection_entity(object: &AisCameraFrustum, mode: i32) -> Handle<dyn Select3DSensitiveEntity> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode).unwrap();
    selection.entities()[0].base_sensitive()
}
#[test]
fn native_frustum_selection_snapshot_and_triangle_alias_lifecycle() {
    let object = AisCameraFrustum::new().unwrap();
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_z_range(1.0, 20.0).unwrap();
    object.set_camera_frustum(&camera).unwrap();
    let (segment, volume, old_center, old_vertex) = capture_frustum_sensitivity(&object);
    camera.set_z_range(3.0, 90.0).unwrap();
    camera.set_scale(15.0);
    object.set_camera_frustum(&camera).unwrap();
    assert_selection_lifecycle(&object, &segment, &volume, old_center, old_vertex);
    assert_eq!(
        volume.state.read().verts,
        object.fields.read().triangles.attributes()
    );
    assert_eq!(AisCameraFrustumSelectionMode::Edges as i32, 0);
    assert_eq!(AisCameraFrustumSelectionMode::Volume as i32, 1);
}
#[test]
fn frustum_selection_before_camera_is_a_typed_error_and_unknown_modes_are_empty() {
    let object = AisCameraFrustum::new().unwrap();
    let selection = Handle::new(SelectMgrSelection::new(0));
    assert!(
        object
            .compute_selection(&selection, 0)
            .unwrap_err()
            .message()
            .contains("borders=null")
    );
    assert!(
        object
            .compute_selection(&selection, 1)
            .unwrap_err()
            .message()
            .contains("triangles=null")
    );
    object.compute_selection(&selection, 9).unwrap();
    assert!(selection.entities().is_empty());
}
#[test]
fn frustum_groups_keep_live_aspects_through_color_and_transparency_changes() {
    let object = AisCameraFrustum::new().unwrap();
    object.set_camera_frustum(&camera(1)).unwrap();
    let prs = presentation();
    object.compute(&Handle::default(), &prs, 1).unwrap();
    let aspects = frustum_group_aspects(&prs);
    let red = Color::new(1.0, 0.0, 0.0, ColorSpace::Rgb).unwrap();
    object.set_color(red).unwrap();
    assert!(aspects.iter().all(|aspect| aspect.interior_color() == red));
    object.unset_color().unwrap();
    object.unset_transparency().unwrap();
    let white = Color::named(NamedColor::WHITE);
    assert!(
        aspects
            .iter()
            .all(|aspect| aspect.interior_color() == white)
    );
    assert_eq!(aspects[0].front_material().transparency(), 0.0);
    assert_eq!(aspects[0].alpha_mode(), Graphic3dAlphaMode::Blend);
}

fn frustum_projection(id: usize) -> Graphic3dCameraProjection {
    [
        Graphic3dCameraProjection::Orthographic,
        Graphic3dCameraProjection::Perspective,
        Graphic3dCameraProjection::Stereo,
        Graphic3dCameraProjection::MonoLeftEye,
    ][id % 4]
}

fn capture_frustum_sensitivity(
    object: &AisCameraFrustum,
) -> (
    Handle<Select3DSensitiveSegment>,
    Handle<Select3DSensitivePrimitiveArray>,
    Point3,
    Point3,
) {
    let edges = selection_entity(object, 0).cast::<Select3DSensitiveGroup>();
    let segment = edges.sub_entity(1).cast::<Select3DSensitiveSegment>();
    let volume = selection_entity(object, 1).cast::<Select3DSensitivePrimitiveArray>();
    let old_center = volume.center_of_geometry();
    let old_vertex = object.fields.read().triangles.vertice(1).unwrap();
    (segment, volume, old_center, old_vertex)
}

fn assert_selection_lifecycle(
    object: &AisCameraFrustum,
    segment: &Select3DSensitiveSegment,
    volume: &Select3DSensitivePrimitiveArray,
    old_center: Point3,
    old_vertex: Point3,
) {
    let actual = [
        f64::from(segment.start_point() == old_vertex),
        f64::from(volume.center_of_geometry() == old_center),
        f64::from(object.fields.read().triangles.vertice(1).unwrap() == old_vertex),
        f64::from(
            object.attributes().shading_aspect().aspect().alpha_mode() == Graphic3dAlphaMode::Blend,
        ),
    ];
    assert_eq!(actual.as_slice(), rows("lifecycle").next().unwrap());
}

fn frustum_group_aspects(
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Vec<Handle<dyn Graphic3dAspectsSource>> {
    prs.groups()
        .into_iter()
        .map(|group| group.aspects())
        .collect()
}
#[test]
fn camera_update_invalidates_every_presentation_and_null_preserves_update_flags() {
    let object = AisCameraFrustum::new().unwrap();
    let manager = frustum_presentation_manager();
    let wire = manager
        .presentation(&object.cast(), 0, true, &Handle::default())
        .unwrap();
    let shaded = manager
        .presentation(&object.cast(), 1, true, &Handle::default())
        .unwrap();
    wire.set_update_status(false);
    shaded.set_update_status(false);
    object.set_camera_frustum(&Handle::default()).unwrap();
    assert!(!wire.must_be_updated() && !shaded.must_be_updated());
    object.set_camera_frustum(&camera(2)).unwrap();
    assert!(wire.must_be_updated() && shaded.must_be_updated());
    manager.update(&object.cast(), 1).unwrap();
    assert!(wire.must_be_updated() && !shaded.must_be_updated());
}
fn frustum_presentation_manager() -> Handle<PrsMgrPresentationManager> {
    let manager =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    PrsMgrPresentationManager::new(&manager)
}
#[test]
fn frustum_runtime_descriptor_retains_native_name_parent_and_interfaces() {
    use rustycad_tkernel::standard::RuntimeTyped;
    let object = AisCameraFrustum::new().unwrap();
    let descriptor = object.dynamic_type().unwrap();
    assert_eq!(AisCameraFrustum::TYPE_NAME, "AIS_CameraFrustum");
    assert!(std::ptr::eq(descriptor, AisCameraFrustum::static_type()));
    assert_eq!(descriptor.parent().unwrap().name(), "AIS_InteractiveObject");
    assert!(!object.cast::<dyn AisInteractiveObject>().is_null());
    assert!(!object.cast::<dyn PrsMgrPresentableObject>().is_null());
    assert!(!object.cast::<dyn SelectMgrSelectableObject>().is_null());
    assert_eq!(object.signature(), -1);
}
