//! Behavior tests of ShapeCustom_Surface (the native comparison is the sc_surface requests
//! of tools/tkshhealing_corpus_construct_package.py).
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomCylindricalSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkgeombase::geom_convert::surface_to_bspline_surface;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::shape_custom::ShapeCustomSurface;

fn bilinear(z: f64) -> Handle<dyn GeomSurface> {
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(0., 1., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., z),
    ];
    let poles = Array2::from_shared(&points, 1, 2, 1, 2).unwrap();
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}

#[test]
fn flat_bilinear_patch_becomes_a_plane_and_substitutes() {
    let mut tool = ShapeCustomSurface::with_surface(&bilinear(0.));
    let plane = tool.convert_to_analytical(1e-7, true).unwrap();
    assert!(!plane.cast::<GeomPlane>().is_null());
    assert!(tool.gap() <= 1e-7);
    assert!(tool.surface() == plane);
}

#[test]
fn twisted_patch_is_rejected_with_its_gap() {
    let mut tool = ShapeCustomSurface::with_surface(&bilinear(0.3));
    assert!(tool.convert_to_analytical(1e-3, false).unwrap().is_null());
    assert!(tool.gap() > 1e-3);
}

#[test]
fn exact_bspline_cylinder_is_recognized() {
    let cylinder: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.).unwrap()).cast();
    let two_pi = 2. * std::f64::consts::PI;
    let trimmed: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&cylinder, (0., two_pi), (0., 3.), (true, true))
            .unwrap(),
    )
    .cast();
    let spline: Handle<dyn GeomSurface> = surface_to_bspline_surface(&trimmed).unwrap().cast();
    let mut tool = ShapeCustomSurface::with_surface(&spline);
    let analytic = tool.convert_to_analytical(1e-6, false).unwrap();
    let found = analytic.cast::<GeomCylindricalSurface>();
    assert!((found.get().expect("a cylinder").radius() - 2.).abs() < 1e-6);
}

#[test]
fn clamped_ring_becomes_periodic_in_u() {
    let ring = [
        (2., 0.),
        (2., 2.),
        (-2., 2.),
        (-2., -2.),
        (2., -2.),
        (2., 0.),
    ];
    let points: Vec<Point3> = ring
        .iter()
        .flat_map(|&(x, y)| [0., 1., 2.].map(|z| Point3::new(x, y, z)))
        .collect();
    let poles = Array2::from_shared(&points, 1, 6, 1, 3).unwrap();
    let (uk, um) = ([0., 1., 2., 3., 4.], [3, 1, 1, 1, 3]);
    let (vk, vm) = ([0., 1.], [3, 3]);
    let spline = GeomBSplineSurface::new(
        &poles,
        (&Array1::from_shared(&uk, 1), &Array1::from_shared(&um, 1)),
        (&Array1::from_shared(&vk, 1), &Array1::from_shared(&vm, 1)),
        (2, 2),
        (false, false),
    )
    .unwrap();
    let mut tool = ShapeCustomSurface::with_surface(&Handle::new(spline).cast());
    let periodic = tool.convert_to_periodic(false, -1.).unwrap();
    let periodic = periodic.cast::<GeomBSplineSurface>();
    let periodic = periodic.get().expect("a B-spline surface");
    assert!(periodic.is_u_periodic() && !periodic.is_v_periodic());
}
