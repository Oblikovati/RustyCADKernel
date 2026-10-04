//! Public `Perform` coverage for offset and swept adaptor surface families.
use crate::int_tools::BeanFaceIntersector;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomOffsetSurface, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

type UvBounds = (f64, f64, f64, f64);
type SurfaceCase = (Handle<dyn GeomSurface>, UvBounds, Edge);

#[test]
fn public_perform_matches_native_offset_and_swept_surface_families() {
    for (name, first, last) in [
        ("offset_plane", 0.0, 4.0),
        ("offset_sphere", 0.0, 6.0),
        ("revolution_torus", 0.0, 4.0),
        ("extruded_cylinder", 0.0, 4.0),
    ] {
        assert_public_surface_case(name, first, last).unwrap();
    }
}

fn assert_public_surface_case(name: &str, first: f64, last: f64) -> Result<(), Failure> {
    let (surface, bounds, edge) = surface_case(name)?;
    let face = make_face(&surface, bounds)?;
    let mut solver = BeanFaceIntersector::from_shapes(&edge, &face)?;
    solver.set_bean_parameters(first, last);
    solver.set_surface_parameters(bounds.0, bounds.1, bounds.2, bounds.3);
    for pass in 0..2 {
        solver.perform()?;
        assert_native_ranges(&solver, name, pass);
    }
    Ok(())
}

fn surface_case(name: &str) -> Result<SurfaceCase, Failure> {
    match name {
        "offset_plane" => Ok((offset_plane()?, (-1., 1., -1., 1.), vertical_edge()?)),
        "offset_sphere" => Ok((
            offset_sphere()?,
            (0., 2. * PI, -PI / 2., PI / 2.),
            sphere_edge()?,
        )),
        "revolution_torus" => Ok((
            revolution_surface()?,
            (0., 2. * PI, -PI, PI),
            vertical_torus_edge()?,
        )),
        "extruded_cylinder" => Ok((
            extruded_surface()?,
            (0., 2. * PI, -1., 1.),
            horizontal_edge()?,
        )),
        _ => Err(Failure::with_message(
            "Standard_ProgramError",
            &format!("unknown BeanFace family case {name}; expected a named supported case"),
        )),
    }
}

fn offset_plane() -> Result<Handle<dyn GeomSurface>, Failure> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    Ok(Handle::new(GeomOffsetSurface::new(&plane, 0.5, false)?).cast())
}

fn offset_sphere() -> Result<Handle<dyn GeomSurface>, Failure> {
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0)?).cast();
    Ok(Handle::new(GeomOffsetSurface::new(&sphere, 0.5, false)?).cast())
}

fn revolution_surface() -> Result<Handle<dyn GeomSurface>, Failure> {
    let center = Point3::new(2.0, 0.0, 0.0);
    let frame = Axis2::with_x_direction(center, Direction3::Y, Direction3::X)?;
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, 0.5)?).cast();
    Ok(Handle::new(GeomSurfaceOfRevolution::new(
        &circle,
        Axis1::new(Point3::default(), Direction3::Z),
    )?)
    .cast())
}

fn extruded_surface() -> Result<Handle<dyn GeomSurface>, Failure> {
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(Axis2::default(), 1.0)?).cast();
    Ok(Handle::new(GeomSurfaceOfLinearExtrusion::new(&circle, Direction3::Z)?).cast())
}

fn make_face(surface: &Handle<dyn GeomSurface>, bounds: UvBounds) -> Result<Face, Failure> {
    BRepBuilderAPIMakeFace::from_surface_bounds(surface, bounds, 1e-7)?.face()
}

fn vertical_edge() -> Result<Edge, Failure> {
    make_edge(Point3::new(0.0, 0.0, -1.0), Point3::new(0.0, 0.0, 1.0))
}

fn vertical_torus_edge() -> Result<Edge, Failure> {
    make_edge(Point3::new(2.0, 0.0, -1.0), Point3::new(2.0, 0.0, 1.0))
}

fn sphere_edge() -> Result<Edge, Failure> {
    make_edge(Point3::new(0.5, 0.0, -3.0), Point3::new(0.5, 0.0, 3.0))
}

fn horizontal_edge() -> Result<Edge, Failure> {
    make_edge(Point3::new(-2.0, 0.0, 0.5), Point3::new(2.0, 0.0, 0.5))
}

fn make_edge(first: Point3, last: Point3) -> Result<Edge, Failure> {
    BRepBuilderAPIMakeEdge::from_points(first, last)?.edge()
}

fn assert_native_ranges(solver: &BeanFaceIntersector, name: &str, pass: usize) {
    let expected = native_ranges(name, pass);
    assert!(solver.is_done(), "{name} pass {pass}");
    assert_eq!(
        solver.result().length() as usize,
        expected.len(),
        "{name} pass {pass}"
    );
    for (index, (first, last)) in expected.into_iter().enumerate() {
        let actual = solver.result().iter().nth(index).unwrap();
        assert!((actual.first() - first).abs() <= 1e-12, "{name} first");
        assert!((actual.last() - last).abs() <= 1e-12, "{name} last");
    }
}

fn native_ranges(name: &str, pass: usize) -> Vec<(f64, f64)> {
    include_str!("../../../../../../tools/oracle/tkbo_bean_swept_surfaces.expected")
        .lines()
        .filter_map(|line| parse_native_range(line, name, pass))
        .flatten()
        .collect()
}

fn parse_native_range(line: &str, name: &str, pass: usize) -> Option<Vec<(f64, f64)>> {
    let mut fields = line.split_whitespace();
    if fields.next()? != name || fields.next()?.parse::<usize>().ok()? != pass {
        return None;
    }
    if fields.next()?.parse::<i32>().ok()? != 1 {
        return None;
    }
    let count = fields.next()?.parse::<usize>().ok()?;
    let ranges = (0..count)
        .map(|_| Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?)))
        .collect::<Option<Vec<_>>>()?;
    Some(ranges)
}

const PI: f64 = std::f64::consts::PI;
