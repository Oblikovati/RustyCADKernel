fn cylinder(center: Point3, axis: Direction3, radius: f64, v: (f64, f64), u: (f64, f64)) -> Face {
    let frame = Axis2::new(center, axis).unwrap().axis3();
    let surface = Handle::new(GeomCylindricalSurface::new(frame, radius).unwrap()).into_cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (u.0, u.1, v.0, v.1), 1e-7)
        .unwrap()
        .face()
        .unwrap()
}
fn circular_plane(radius: f64) -> Face {
    let frame = Axis2::new(Point3::default(), Direction3::Y).unwrap();
    let plane = Handle::new(GeomPlane::new(frame.axis3())).into_cast();
    let circle = Circle3::new(frame, radius).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    BRepBuilderAPIMakeFace::from_surface_wire(&plane, &wire, true)
        .unwrap()
        .face()
        .unwrap()
}
fn intersect(first: &Face, second: &Face, approximation: [bool; 3]) -> FaceFace {
    assert!(!first.is_null());
    assert!(!second.is_null());
    let mut intersection = FaceFace::default();
    intersection.set_parameters(approximation[0], approximation[1], approximation[2], 1e-7);
    intersection.perform(first, second, false).unwrap();
    assert!(intersection.is_done());
    intersection
}
fn bare_face(surface: Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
fn bounded_cylinder(frame: Axis3, vmax: f64) -> Face {
    let surface =
        Handle::new(GeomCylindricalSurface::new(frame, 408.974517822893).unwrap()).into_cast();
    let maker = BRepBuilderAPIMakeFace::from_surface_bounds(
        &surface,
        (0., 0.0225015452057227, -146.010003766057, vmax),
        1e-7,
    )
    .unwrap();
    assert!(maker.is_done());
    maker.face().unwrap()
}
fn cone_first_face(radius: f64) -> Face {
    use rustycad_tkbrep::{topexp::Explorer, topods};
    use rustycad_tkg3d::top_abs::ShapeEnum;
    let mut maker =
        rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCone::new(radius, 0., 20.).unwrap();
    let shape = maker.shape().unwrap();
    assert!(!shape.is_null());
    let explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(explorer.more());
    topods::face(explorer.current()).unwrap()
}

fn occ32850_cylinders() -> (Face, Face) {
    let x1 = Direction3::new(-1.73024882663956e-6, 0., 0.999999999998503).unwrap();
    let x2 = Direction3::new(0., 1.73024882663956e-6, 0.999999999998503).unwrap();
    let frame1 = Axis3::new(
        Point3::new(25.8071575178163, 0., -373.974517822281),
        Direction3::Y,
        x1,
    )
    .unwrap();
    let frame2 = Axis3::new(
        Point3::new(0., 1974.19284248218, -373.974517822281),
        Direction3::X,
        x2,
    )
    .unwrap();
    let first = bounded_cylinder(frame1, 2146.01000376606);
    let second = bounded_cylinder(frame2, 946.010003766057);
    (first, second)
}
