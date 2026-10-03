//! Native oracle cases at OCCT 3d097a0328e71b8.
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Plane3;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        FaceDivider, ShapeUpgradeFaceDivide, ShapeUpgradeFaceDivideArea, ShapeUpgradeSplitSurface,
    },
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

fn configure(divider: &ShapeUpgradeFaceDivideArea, mode: usize) {
    if mode == 7 {
        divider.set_max_area(1.);
    }
    if mode == 1 {
        divider.set_max_area(2.);
    }
    if mode == 2 || mode == 3 {
        divider.set_max_area(-1.);
        divider.set_nb_parts(4);
    }
    if mode == 3 {
        divider.set_splitting_by_number(true);
        divider.set_numbers_uv_splits(2, 3).unwrap();
    }
    if mode == 4 || mode == 5 {
        divider.set_max_area(2.);
        divider.set_split_surface_tool(&if mode == 4 {
            Handle::default()
        } else {
            Handle::new(ShapeUpgradeSplitSurface::new())
        });
    }
    if mode == 6 {
        divider.set_max_area(-1.);
        divider.set_nb_parts(0);
    }
}
fn face_count(shape: &Shape) -> f64 {
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut count = 0.;
    while faces.more() {
        count += 1.;
        faces.next().unwrap();
    }
    count
}
fn fixture(mode: usize) -> rustycad_tkbrep::topods::Face {
    if mode == 7 {
        use rustycad_tkg3d::geom::{GeomSphericalSurface, GeomSurface};
        let surface: Handle<dyn GeomSurface> = Handle::new(
            GeomSphericalSurface::new(rustycad_tkmath::gp::Axis3::default(), 1.).unwrap(),
        )
        .cast();
        return BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0., 3., -1.5, 1.5), 1e-7)
            .unwrap()
            .face()
            .unwrap();
    }
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 2., 0., 2.))
        .unwrap()
        .face()
        .unwrap()
}
fn observe(mode: usize) -> Vec<f64> {
    let face = fixture(mode);
    let divider = ShapeUpgradeFaceDivideArea::new(face.clone());
    configure(&divider, mode);
    let changed = (&divider as &dyn FaceDivider).perform(999.).unwrap();
    if mode == 7 {
        compare_sphere_patches(&divider.result());
    }
    let mut row = vec![
        mode as f64,
        f64::from(changed),
        face_count(&divider.result()),
        f64::from(divider.context().is_null()),
        f64::from(divider.result().is_same(&face.into_shape())),
        divider.max_area(),
        f64::from(divider.nb_parts()),
    ];
    for flag in [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ] {
        row.push(f64::from(divider.status_flag(flag)));
    }
    row
}
#[test]
fn area_subdivision_matches_native_rows() {
    let oracle = include_str!("../../../tools/oracle/tkshhealing_face_divide_area.expected");
    for (mode, line) in oracle
        .lines()
        .filter(|row| !row.starts_with("patch "))
        .enumerate()
    {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|word| word.parse().unwrap())
            .collect();
        let actual = observe(mode);
        assert_eq!(actual.len(), expected.len());
        for (column, (&found, &wanted)) in actual.iter().zip(&expected).enumerate() {
            assert!(
                found == wanted || (found - wanted).abs() < 1e-14,
                "mode {mode}, column {column}: {found}, expected {wanted}"
            );
        }
    }
}
#[test]
fn native_empty_constructors_and_area_defaults() {
    let base = ShapeUpgradeFaceDivide::empty();
    assert!(base.result().is_null());
    assert!(!base.perform(0.).unwrap());
    let divider = ShapeUpgradeFaceDivideArea::empty();
    assert!(divider.result().is_null());
    assert_eq!(divider.max_area(), precision::infinite());
    assert_eq!(divider.nb_parts(), 0);
    assert_eq!(divider.precision(), 1e-5);
    assert!(divider.context().is_null());
}
#[test]
fn grid_counts_preserve_native_nonpositive_behavior() {
    let divider = ShapeUpgradeFaceDivideArea::empty();
    divider.set_numbers_uv_splits(2, 3).unwrap();
    assert_eq!(divider.nb_parts(), 6);
    divider.set_numbers_uv_splits(-1, 3).unwrap();
    assert_eq!(divider.nb_parts(), 6);
    let error = divider.set_numbers_uv_splits(i32::MAX, 2).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

fn compare_sphere_patches(result: &Shape) {
    let oracle = include_str!("../../../tools/oracle/tkshhealing_face_divide_area.expected");
    let mut faces = Explorer::new(result, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    for line in oracle.lines().filter(|row| row.starts_with("patch ")) {
        assert!(faces.more());
        let actual = patch_observation(faces.current());
        let expected: Vec<f64> = line
            .split_whitespace()
            .skip(1)
            .map(|s| s.parse().unwrap())
            .collect();
        for (found, wanted) in actual.iter().zip(&expected) {
            assert!(
                (found - wanted).abs() < 1e-12,
                "patch {actual:?}, expected {expected:?}"
            );
        }
        faces.next().unwrap();
    }
    assert!(!faces.more());
}
fn patch_observation(shape: &Shape) -> [f64; 5] {
    let face = rustycad_tkbrep::topods::face(shape).unwrap();
    let (u1, u2, v1, v2) =
        rustycad_tkshhealing::shape_analysis::ShapeAnalysis::get_face_uv_bounds(&face).unwrap();
    let mut properties = rustycad_tkgeombase::gprop::GProps::default();
    rustycad_tktopalgo::brep_gprop::BRepGProp::surface_properties_eps(
        shape,
        &mut properties,
        1e-5,
        false,
    )
    .unwrap();
    [u1, u2, v1, v2, properties.mass()]
}
