//! Behavior tests of ShapeCustom_ConvertToBSpline (the native comparison is the sc_modif
//! kind 3 requests of tools/tkshhealing_corpus_shapecustom.py).
mod healing_support;

use healing_support::{edges_of, square_face};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, NewCurve, NewSurface};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkshhealing::shape_custom::ShapeCustomConvertToBSpline;

#[test]
fn plane_is_kept_by_default() {
    let face = square_face(0.);
    let modification = ShapeCustomConvertToBSpline::new();
    let mut surface = NewSurface::default();
    assert!(!modification.new_surface(&face, &mut surface).unwrap());
    let edge = &edges_of(&face.clone().into_shape())[0];
    assert!(
        !modification
            .new_curve(edge, &mut NewCurve::default())
            .unwrap()
    );
}

#[test]
fn plane_mode_converts_the_plane_on_the_face_bounds() {
    let face = square_face(0.);
    let modification = ShapeCustomConvertToBSpline::new();
    modification.set_plane_mode(true);
    let mut surface = NewSurface::default();
    assert!(modification.new_surface(&face, &mut surface).unwrap());
    let spline = surface.surface.cast::<GeomBSplineSurface>();
    let spline = spline.get().expect("a B-spline surface");
    assert_eq!((spline.u_degree(), spline.v_degree()), (1, 1));
    // The plane of a polygon face is framed at its centre: the UV box is [-0.5, 0.5]^2.
    assert_eq!(spline.bounds(), (-0.5, 0.5, -0.5, 0.5));
    assert!(!surface.rev_wires && !surface.rev_face);
    let edge = &edges_of(&face.clone().into_shape())[0];
    // BRepBuilderAPI stores no p-curve on a plane: no curve on surface asks for a copy.
    assert!(
        !modification
            .new_curve(edge, &mut NewCurve::default())
            .unwrap()
    );
}

#[test]
fn modifier_rebuilds_a_plane_face_on_a_bspline() {
    let face = square_face(2.).into_shape();
    let modification = ShapeCustomConvertToBSpline::new();
    modification.set_plane_mode(true);
    let modification: Handle<dyn BRepToolsModification> = Handle::new(modification).cast();
    let modifier = Modifier::with_modification(&face, &modification).unwrap();
    assert!(modifier.is_done());
    let result = modifier.modified_shape(&face).unwrap();
    let rebuilt = rustycad_tkbrep::topods::face(&result).unwrap();
    let surface = rustycad_tkbrep::brep::BRepTool::surface(&rebuilt).unwrap();
    assert!(!surface.cast::<GeomBSplineSurface>().is_null());
}
