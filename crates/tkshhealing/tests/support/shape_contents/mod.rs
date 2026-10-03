use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Wire},
};
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    standard::Handle,
};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dOffsetCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCurve, GeomLine, GeomOffsetCurve, GeomPlane, GeomSurface,
    GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisShapeContents;
pub fn modes(c: &mut ShapeAnalysisShapeContents) {
    *c.modify_big_spline_mode() = true;
    *c.modify_indirect_mode() = true;
    *c.modify_offset_surface_mode() = true;
    *c.modify_trimmed_3d_mode() = true;
    *c.modify_offset_curve_mode() = true;
    *c.modify_trimmed_2d_mode() = true;
}
pub fn face(surface: &Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, surface, 1e-7)
        .unwrap();
    face
}
pub fn spline(count: i32) -> Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 2, 1, count).unwrap();
    for j in 1..=count {
        poles[(1, j)] = Point3::new(0., f64::from(j - 1), 0.);
        poles[(2, j)] = Point3::new(1., f64::from(j - 1), 0.);
    }
    let v: Vec<f64> = (0..count).map(f64::from).collect();
    let mults: Vec<i32> = (0..count)
        .map(|i| if i == 0 || i == count - 1 { 2 } else { 1 })
        .collect();
    let u = Array1::from_shared(&[0., 1.], 1);
    let um = Array1::from_shared(&[2, 2], 1);
    let v = Array1::from_shared(&v, 1);
    let vm = Array1::from_shared(&mults, 1);
    Handle::new(
        GeomBSplineSurface::new(&poles, (&u, &um), (&v, &vm), (1, 1), (false, false)).unwrap(),
    )
    .cast()
}
pub fn curve_face() -> Face {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = face(&plane);
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::default(),
        Direction3::new(1., 0., 0.).unwrap(),
    ))
    .cast();
    let line2: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&line, 0., 1., true, true).unwrap()).cast();
    let trimmed2: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&line2, 0., 1., true, true).unwrap()).cast();
    append_edge(&builder, &mut wire, &face, &trimmed, &trimmed2);
    let offset: Handle<dyn GeomCurve> = Handle::new(
        GeomOffsetCurve::new(&line, 1., Direction3::new(0., 0., 1.).unwrap(), false).unwrap(),
    )
    .cast();
    let offset2: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&line2, 1., false).unwrap()).cast();
    append_edge(&builder, &mut wire, &face, &offset, &offset2);
    builder.add(&mut face, &wire).unwrap();
    face
}
fn append_edge(
    builder: &BRepBuilder,
    wire: &mut Wire,
    face: &Face,
    curve: &Handle<dyn GeomCurve>,
    pcurve: &Handle<dyn Geom2dCurve>,
) {
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, curve, 1e-7)
        .unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, pcurve, face, 1e-7)
        .unwrap();
    builder.range(&edge, 0., 1., false).unwrap();
    builder.add(wire, &edge).unwrap();
}
