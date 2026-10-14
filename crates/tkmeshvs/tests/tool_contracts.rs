//! Independent invariants supplement the exact native transcript, using actual public methods.
use martensite_tkmeshvs::{MeshVSDrawer, MeshVSDrawerAttribute as K, MeshVSTool as T};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkmath::gp::Vector3;
use rustycad_tkservice::{
    aspect::AspectTypeOfLine,
    graphic3d::{
        Graphic3dAspectsLineMembers, Graphic3dAspectsMembers, Graphic3dMaterialAspect,
        Graphic3dNameOfMaterial as M,
    },
};
#[test]
fn nonplanarity_keeps_single_normal_but_writes_actual_unnormalized_average() {
    let nodes = nodes(&[0., 0., 0., 1., 0., 0., 1., 1., 1., 0., 1., 0.]);
    let mut normal = Vector3::new(7., 8., 9.);
    assert!(!T::get_normal(&nodes, &mut normal).unwrap());
    assert_eq!(normal, Vector3::new(7., 8., 9.));
    assert!(!T::get_average_normal(&nodes, &mut normal).unwrap());
    assert_eq!(normal.x(), 0.);
    assert!((normal.y() + std::f64::consts::FRAC_1_SQRT_2 / 2.).abs() < 1e-15);
    assert!((normal.z() - (1. + std::f64::consts::FRAC_1_SQRT_2) / 2.).abs() < 1e-15);
    assert!(normal.y() * normal.y() + normal.z() * normal.z() < 1.);
}
#[test]
fn degenerate_and_short_polygons_preserve_native_distinct_output_contracts() {
    let mut normal = Vector3::new(7., 8., 9.);
    assert!(!T::get_normal(&nodes(&[0., 0., 0., 1., 0., 0.]), &mut normal).unwrap());
    assert_eq!(normal, Vector3::new(7., 8., 9.));
    assert!(T::get_normal(&nodes(&[0., 0., 0., 1., 0., 0., 2., 0., 0.]), &mut normal).unwrap());
    assert_eq!(normal, Vector3::new(0., 0., 0.));
}
#[test]
fn too_long_header_failure_preserves_output_and_all_original_array_values() {
    let values = [4., 0., 0., 0., 1., 0., 0., 0., 1., 0.];
    let mut input = Array1::with_bounds(0, 9).unwrap();
    input.as_mut_slice().copy_from_slice(&values);
    let mut normal = Vector3::new(7., 8., 9.);
    let cause = T::get_average_normal(&input, &mut normal).unwrap_err();
    assert!(cause.is_kind("Standard_OutOfRange"));
    assert!(cause.message().starts_with("NCollection_Array1::Value"));
    assert_eq!(normal, Vector3::new(7., 8., 9.));
    assert_eq!(input.as_slice(), &values);
}
fn nodes(values: &[f64]) -> Array1<'static, f64> {
    let mut nodes = Array1::with_bounds(1, values.len() as i32).unwrap();
    nodes.as_mut_slice().copy_from_slice(values);
    nodes
}
#[test]
fn material_input_and_both_result_material_values_are_independent() {
    let drawer = Handle::new(MeshVSDrawer::default());
    let mut material = Graphic3dMaterialAspect::new(M::Gold);
    material.set_transparency(0.25).unwrap();
    let aspect = T::create_aspect_fill_area3d_with_material(&drawer, &material, true).unwrap();
    material.set_transparency(0.75).unwrap();
    assert_eq!(aspect.front_material().transparency(), 0.25);
    assert_eq!(aspect.back_material().transparency(), 0.25);
    aspect.change_front_material(|front| front.set_transparency(0.5).unwrap());
    assert_eq!(aspect.back_material().transparency(), 0.25);
    assert_eq!(material.transparency(), 0.75);
    assert_eq!(drawer.ref_count(), 1);
    assert_eq!(aspect.ref_count(), 1);
}
#[test]
fn drawer_material_factory_uses_integer_ids_and_never_the_material_value_channel() {
    let drawer = Handle::new(MeshVSDrawer::default());
    drawer.set_material(
        K::FrontMaterial as i32,
        &Graphic3dMaterialAspect::new(M::Gold),
    );
    let aspect = T::create_aspect_fill_area3d(&drawer, true).unwrap();
    assert_eq!(aspect.front_material().name(), M::Brass);
    assert!(
        T::create_aspect_fill_area3d(&drawer, false)
            .unwrap()
            .is_null()
    );
    drawer.set_integer(K::FrontMaterial as i32, M::Gold as i32);
    drawer.set_integer(K::BackMaterial as i32, M::Steel as i32);
    let aspect = T::create_aspect_fill_area3d(&drawer, true).unwrap();
    assert_eq!(aspect.front_material().name(), M::Gold);
    assert_eq!(aspect.back_material().name(), M::Steel);
}
#[test]
fn original_drawer_reuse_and_prior_aspects_survive_factory_failure_and_repair() {
    let drawer = Handle::new(MeshVSDrawer::default());
    drawer.set_double(K::BeamWidth as i32, 2.);
    let first = T::create_aspect_line3d(&drawer, true).unwrap();
    drawer.set_double(K::BeamWidth as i32, 0.);
    assert!(
        T::create_aspect_line3d(&drawer, true)
            .unwrap_err()
            .is_kind("Standard_OutOfRange")
    );
    assert_eq!(first.line_width(), 2.);
    drawer.set_double(K::BeamWidth as i32, 3.);
    let last = T::create_aspect_line3d(&drawer, true).unwrap();
    assert_eq!(last.line_width(), 3.);
    last.set_line_type(AspectTypeOfLine::Dash);
    assert_eq!(first.line_type(), AspectTypeOfLine::Solid);
    assert_eq!(drawer.ref_count(), 1);
}
#[test]
fn generated_orientation_and_degeneracy_cases_preserve_exact_axis_normals() {
    for sign in [-1., 1.] {
        for scale in [1., 2., 8.] {
            let input = nodes(&[0., 0., 0., scale, 0., 0., 0., sign * scale, 0.]);
            let mut output = Vector3::default();
            assert!(T::get_normal(&input, &mut output).unwrap());
            assert_eq!(output, Vector3::new(0., 0., sign));
        }
    }
}
