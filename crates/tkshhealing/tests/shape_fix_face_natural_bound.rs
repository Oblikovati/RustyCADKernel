//! Native-backed natural-bound construction and periodic wire handling.
#[path = "support/shape_fix_face.rs"]
mod support;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Builder, Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomCylindricalSurface, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysis;
use rustycad_tkshhealing::shape_fix::ShapeFixFace;
use support::*;

#[test]
fn add_natural_bound_builds_a_finite_empty_sphere_face() {
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap())
        .cast::<dyn GeomSurface>();
    let fixer = ShapeFixFace::default();
    fixer.init_surface(&sphere, 1e-7, true).unwrap();
    fixer.set_fix_add_natural_bound_mode(0);
    let parent: &rustycad_tkshhealing::shape_fix::ShapeFixRoot = &fixer;
    let root_shape = parent.fix_root_data().with(|fields| fields.shape.clone());

    assert!(fixer.fix_add_natural_bound().unwrap());
    let face = fixer.face().unwrap();
    let retained_root = parent.fix_root_data().with(|fields| fields.shape.clone());
    assert!(retained_root.is_same(&root_shape));
    assert!(!face.clone().into_shape().is_same(&root_shape));
    assert_eq!(count_wires(&face), 1);
    assert_eq!(wire_orientations(&face), [Orientation::Forward]);
    assert!(fixer.result().is_same(&face.clone().into_shape()));
}

#[test]
fn empty_cylinder_skips_natural_bound_because_one_uv_range_is_infinite() {
    let cylinder = Handle::new(
        GeomCylindricalSurface::new(
            Axis3::new(
                Point3::default(),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
            1.0,
        )
        .unwrap(),
    )
    .cast::<dyn GeomSurface>();
    let fixer = ShapeFixFace::default();
    fixer.init_surface(&cylinder, 1e-7, true).unwrap();

    assert!(!fixer.fix_add_natural_bound().unwrap());
    assert_eq!(count_wires(&fixer.face().unwrap()), 0);
}

#[test]
fn uninitialized_face_has_no_natural_boundary_to_add() {
    assert!(!ShapeFixFace::default().fix_add_natural_bound().unwrap());
}

#[test]
fn native_sphere_wire_natural_bound_stays_unchanged_for_an_outer_wire() {
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap())
        .cast::<dyn GeomSurface>();
    let wire = patch_wire(&sphere, (0.2, 1.0, 0.1, 0.5));
    let fixer = ShapeFixFace::default();
    fixer.init_surface(&sphere, 1e-7, true).unwrap();
    fixer.add_wire(&wire).unwrap();

    assert!(!fixer.fix_add_natural_bound().unwrap());
    assert_eq!(count_wires(&fixer.face().unwrap()), 1);
}

#[test]
fn reversed_torus_wire_gets_a_natural_boundary_and_periodic_shift() {
    let torus = Handle::new(GeomToroidalSurface::new(Axis3::default(), 5.0, 1.0).unwrap())
        .cast::<dyn GeomSurface>();
    let mut reversed = patch_wire(&torus, (0.2, 1.0, 0.2, 1.0)).into_shape();
    reversed.reverse();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &torus, 1e-7)
        .unwrap();
    Builder.add(&mut face, &reversed).unwrap();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.set_fix_add_natural_bound_mode(1);
    assert!(!ShapeAnalysis::is_outer_bound(&fixer.face().unwrap()).unwrap());

    assert!(fixer.fix_add_natural_bound().unwrap());
    let face = fixer.face().unwrap();
    assert!(fixer.result().is_null());
    let wires = wire_shapes(&face);
    assert_eq!(wires.len(), 2);
    assert_pcurves_rebuild_the_same_3d_edges(&face, &wires);
    assert_uv_close(
        first_pcurve_midpoint(&face, &wires[0]),
        (6.483_185_307_179_586, 6.883_185_307_179_586),
    );
    assert_uv_close(
        first_pcurve_midpoint(&face, &wires[1]),
        (3.641_592_653_589_793, 6.783_185_307_179_586),
    );
}
