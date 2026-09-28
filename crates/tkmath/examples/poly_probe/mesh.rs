//! Triangulation queries, package geometry and Poly_Connect counterparts.
use super::{Fields, bounds, failure, flag, fvec, mesh, transform};
use crate::{xy, xyz};
use rustycad_tkernel::standard::{Failure, Handle, dump::DumpStream};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis1, Direction3, Point2, Point3, Transform3, Xy, Xyz},
    poly::{self, Connect, ListOfTriangulation, Triangle, Triangulation},
};
fn normals(mesh: &Triangulation) -> Result<(), Failure> {
    mesh.compute_normals()?;
    for index in 1..=mesh.nb_nodes() {
        fvec(mesh.normal_vec3(index)?);
    }
    xyz(Xyz::from(mesh.normal(1)?));
    Ok(())
}
fn ranges(mesh: &Triangulation, transform: Transform3, accurate: bool) {
    let mut first = BoundingBox3::default();
    print!("{} ", flag(mesh.min_max(&mut first, transform, accurate)));
    bounds(&first);
    mesh.update_cached_min_max();
    print!("{} ", flag(mesh.has_cached_min_max()));
    bounds(&mesh.cached_min_max());
    for accurate in [accurate, false] {
        let mut range = BoundingBox3::default();
        print!("{} ", flag(mesh.min_max(&mut range, transform, accurate)));
        bounds(&range);
    }
    mesh.set_cached_min_max(&BoundingBox3::default());
    print!("{} ", flag(mesh.has_cached_min_max()));
}
fn rays(mesh: &Triangulation, fields: &mut Fields<'_>) -> Result<(), Failure> {
    let origin = Point3::new(fields.real(), fields.real(), fields.real());
    let direction = Direction3::new(fields.real(), fields.real(), fields.real())?;
    let axis = Axis1::new(origin, direction);
    for closest in [true, false] {
        let (mut hit, mut distance) = (Triangle::new(-1, -1, -1), 91.0);
        let ok = poly::intersect(mesh, axis, closest, &mut hit, &mut distance)?;
        let (a, b, c) = hit.get();
        print!("{} {a} {b} {c} {} ", flag(ok), distance.to_bits());
    }
    Ok(())
}
fn catenated(mesh: &Handle<Triangulation>) -> Result<(), Failure> {
    let mut list = ListOfTriangulation::default();
    list.append(mesh.clone());
    list.append(Handle::default());
    list.append(mesh.copy());
    let result = poly::catenate(&list)?;
    let (nodes, triangles) = (result.nb_nodes(), result.nb_triangles());
    print!(
        "{nodes} {triangles} {} {} ",
        flag(result.has_uv_nodes()),
        flag(result.has_normals())
    );
    let (a, b, c) = result.triangle(triangles)?.get();
    print!("{a} {b} {c} ");
    xyz(result.node(nodes)?.xyz());
    Ok(())
}
pub(crate) fn mesh_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let mesh = mesh(&mut fields);
    let transform = transform(&mut fields);
    let accurate = fields.int() != 0;
    print!(
        "{} {} {} {} ",
        mesh.nb_nodes(),
        mesh.nb_triangles(),
        flag(mesh.has_geometry()),
        flag(mesh.is_double_precision())
    );
    for index in 1..=mesh.nb_nodes() {
        xyz(mesh.node(index).unwrap().xyz());
    }
    ranges(&mesh, transform, accurate);
    if let Err(error) = rays(&mesh, &mut fields) {
        failure(&error);
    }
    if let Err(error) = catenated(&mesh) {
        failure(&error);
    }
    if let Err(error) = normals(&mesh) {
        failure(&error);
    }
    let mut json = DumpStream::default();
    mesh.dump_json(&mut json).unwrap();
    let flat = mesh.map_normal_array().map_or(0, |a| a.len());
    println!("{} {flat}", json.text());
}
pub(crate) fn point_on_triangle_case(values: &[f64]) {
    let point = |i: usize| Xy::new(values[i], values[i + 1]);
    let (distance, uv) = poly::point_on_triangle(point(0), point(2), point(4), point(6));
    print!("{} ", distance.to_bits());
    xy(uv);
    println!();
}
pub(crate) fn tri_line_case(values: &[f64]) {
    let v = |i: usize| Xyz::new(values[i], values[i + 1], values[i + 2]);
    match Direction3::new(values[3], values[4], values[5]) {
        Ok(direction) => {
            let hit = poly::intersect_tri_line(v(0), direction, v(6), v(9), v(12));
            print!("{} {} ", flag(hit.is_some()), hit.unwrap_or(91.0).to_bits());
        }
        Err(error) => failure(&error),
    }
    println!();
}
pub(crate) fn properties_case(values: &[f64]) {
    let count = values[0] as usize;
    let points: Vec<Point2> = (0..count)
        .map(|i| Point2::new(values[1 + 2 * i], values[2 + 2 * i]))
        .collect();
    let (area, perimeter) = poly::polygon_properties(&points);
    println!("1 {} {}", area.to_bits(), perimeter.to_bits());
}
fn connect_report(mesh: &Handle<Triangulation>) -> Result<(), Failure> {
    let mut connect = Connect::new(mesh)?;
    for node in 1..=mesh.nb_nodes() {
        print!("{} ", connect.triangle(node)?);
    }
    for triangle in 1..=mesh.nb_triangles() {
        let (a, b, c) = connect.triangles(triangle)?;
        let (d, e, f) = connect.nodes(triangle)?;
        print!("{a} {b} {c} {d} {e} {f} ");
    }
    for node in 1..=mesh.nb_nodes() {
        print!("[");
        connect.initialize(node)?;
        while connect.more() {
            print!("{} ", connect.value());
            connect.next()?;
        }
        print!("] ");
    }
    Ok(())
}
pub(crate) fn connect_case(values: &[f64]) {
    let mesh = mesh(&mut Fields::new(values));
    if let Err(error) = connect_report(&mesh) {
        failure(&error);
    }
    println!();
}
