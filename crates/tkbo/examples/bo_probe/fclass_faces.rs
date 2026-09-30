use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Wire},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Cone3, Cylinder3, Ellipse3, Plane3, Point3, Sphere3, Torus3,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge as MakeEdge, BRepBuilderAPIMakeFace as MakeFace,
    BRepBuilderAPIMakeWire as MakeWire,
};
fn circle(radius: f64) -> Result<Wire, Failure> {
    let edge = MakeEdge::from_circ(Circle3::new(Axis2::default(), radius)?)?.edge()?;
    MakeWire::from_edge(&edge)?.wire()
}
fn circular_face(radius: f64, hole: bool) -> Result<Face, Failure> {
    let mut maker = MakeFace::from_plane_wire(Plane3::default(), &circle(radius)?, true)?;
    if hole {
        let inner = Wire::unchecked(circle(radius * 0.5)?.reversed());
        maker.add(&inner)?;
    }
    maker.face()
}
pub fn face(kind: i32, r: f64) -> Result<Face, Failure> {
    let bounds = (0.25, 5.5, -1.0, 1.0);
    let maker = match kind {
        1 => MakeFace::from_cylinder_bounds(Cylinder3::new(Axis3::default(), r)?, bounds)?,
        2 => MakeFace::from_cone_bounds(Cone3::new(Axis3::default(), 0.3, r)?, bounds)?,
        3 => MakeFace::from_sphere_bounds(Sphere3::new(Axis3::default(), r)?, bounds)?,
        4 => MakeFace::from_torus_bounds(Torus3::new(Axis3::default(), r * 2., r)?, bounds)?,
        5 | 6 => return circular_face(r, kind == 6),
        7 => MakeFace::from_plane(Plane3::default())?,
        9 => MakeFace::from_sphere(Sphere3::new(Axis3::default(), r)?)?,
        10 => return thin_ellipse(r),
        11..=13 => return unusual_face(kind, r),
        _ => MakeFace::from_plane_bounds(Plane3::default(), (0., r, 0., r))?,
    };
    let face = maker.face()?;
    Ok(if kind == 8 {
        Face::unchecked(face.oriented(Orientation::Reversed))
    } else {
        face
    })
}

fn unusual_face(kind: i32, radius: f64) -> Result<Face, Failure> {
    let mut face = MakeFace::from_plane(Plane3::default())?.face()?;
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    if kind == 11 {
        wire = MakeWire::from_edge(
            &MakeEdge::from_points(Point3::default(), Point3::new(radius, 0., 0.))?.edge()?,
        )?
        .wire()?;
    }
    if kind == 13 {
        let mut edge = Edge::default();
        builder.make_edge(&mut edge)?;
        builder.add(&mut wire, &edge)?;
    }
    builder.add(&mut face, &wire)?;
    Ok(face)
}

fn thin_ellipse(radius: f64) -> Result<Face, Failure> {
    let ellipse = Ellipse3::new(Axis2::default(), radius, radius * 1e-5)?;
    let edge = MakeEdge::from_elips(ellipse)?.edge()?;
    MakeFace::from_plane_wire(
        Plane3::default(),
        &MakeWire::from_edge(&edge)?.wire()?,
        true,
    )?
    .face()
}
