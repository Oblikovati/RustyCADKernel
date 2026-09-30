// Rust translation of Poly::ReadTriangulation, ReadPolygon3D and ReadPolygon2D in
// FoundationClasses/TKMath/Poly/Poly.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Readers of the compact text. A wrong header gives a null handle, as natively; the
//! counts then size one-based tables, so a negative count raises `Standard_RangeError`.
use super::tokens::TokenReader;
use crate::{
    gp::{Point2, Point3},
    poly::{Polygon2D, Polygon3D, Triangle, Triangulation},
};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use std::io::BufRead;
fn header_matches<R: BufRead>(
    reader: &mut TokenReader<'_, R>,
    expected: &str,
) -> Result<bool, Failure> {
    Ok(reader.token()?.as_deref() == Some(expected))
}
fn table<T: Default>(member: &str, count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(|e| e.into_failure(member))
}
fn read_points3<R: BufRead>(
    reader: &mut TokenReader<'_, R>,
    count: i32,
) -> Result<Array1<'static, Point3>, Failure> {
    let mut nodes: Array1<'static, Point3> = table("Poly::ReadTriangulation", count)?;
    for node in nodes.iter_mut() {
        let (x, y, z) = (reader.real("x")?, reader.real("y")?, reader.real("z")?);
        node.set_coordinates(x, y, z);
    }
    Ok(nodes)
}
fn read_points2<R: BufRead>(
    reader: &mut TokenReader<'_, R>,
    nodes: &mut Array1<'static, Point2>,
) -> Result<(), Failure> {
    for node in nodes.iter_mut() {
        let (x, y) = (reader.real("x")?, reader.real("y")?);
        *node = Point2::new(x, y);
    }
    Ok(())
}
fn read_triangles<R: BufRead>(
    reader: &mut TokenReader<'_, R>,
    count: i32,
) -> Result<Array1<'static, Triangle>, Failure> {
    let mut triangles = table("Poly::ReadTriangulation", count)?;
    for triangle in triangles.iter_mut() {
        let (a, b, c) = (
            reader.integer("n1")?,
            reader.integer("n2")?,
            reader.integer("n3")?,
        );
        *triangle = Triangle::new(a, b, c);
    }
    Ok(triangles)
}
/// `Poly::ReadTriangulation`: reads the compact text of [`super::write_triangulation`];
/// `read_triangulation(&mut input)?`.
pub fn read_triangulation(input: &mut impl BufRead) -> Result<Handle<Triangulation>, Failure> {
    let mut reader = TokenReader::new(input);
    if !header_matches(&mut reader, "Poly_Triangulation")? {
        return Ok(Handle::default());
    }
    let nb_nodes = reader.integer("nbNodes")?;
    let nb_triangles = reader.integer("nbTriangles")?;
    let has_uv = reader.flag("hasUV")?;
    let deflection = reader.real("deflection")?;
    let nodes = read_points3(&mut reader, nb_nodes)?;
    let mut uv_nodes = table("Poly::ReadTriangulation", nb_nodes)?;
    if has_uv {
        read_points2(&mut reader, &mut uv_nodes)?;
    }
    let triangles = read_triangles(&mut reader, nb_triangles)?;
    let triangulation = if has_uv {
        Triangulation::from_nodes_and_uv(&nodes, &uv_nodes, &triangles)?
    } else {
        Triangulation::from_nodes(&nodes, &triangles)?
    };
    triangulation.set_deflection(deflection);
    Ok(Handle::new(triangulation))
}
/// `Poly::ReadPolygon3D`; `read_polygon3d(&mut input)?`.
pub fn read_polygon3d(input: &mut impl BufRead) -> Result<Handle<Polygon3D>, Failure> {
    let mut reader = TokenReader::new(input);
    if !header_matches(&mut reader, "Poly_Polygon3D")? {
        return Ok(Handle::default());
    }
    let nb_nodes = reader.integer("nbNodes")?;
    let has_parameters = reader.flag("hasparameters")?;
    let deflection = reader.real("deflection")?;
    let nodes = read_points3(&mut reader, nb_nodes)?;
    let mut parameters: Array1<'static, f64> = table("Poly::ReadPolygon3D", nb_nodes)?;
    if has_parameters {
        for parameter in parameters.iter_mut() {
            *parameter = reader.real("parameter")?;
        }
    }
    let polygon = if has_parameters {
        Polygon3D::from_nodes_and_parameters(&nodes, &parameters)?
    } else {
        Polygon3D::from_nodes(&nodes)
    };
    polygon.set_deflection(deflection);
    Ok(Handle::new(polygon))
}
/// `Poly::ReadPolygon2D`; `read_polygon2d(&mut input)?`.
pub fn read_polygon2d(input: &mut impl BufRead) -> Result<Handle<Polygon2D>, Failure> {
    let mut reader = TokenReader::new(input);
    if !header_matches(&mut reader, "Poly_Polygon2D")? {
        return Ok(Handle::default());
    }
    let nb_nodes = reader.integer("nbNodes")?;
    let deflection = reader.real("deflection")?;
    let mut nodes = table("Poly::ReadPolygon2D", nb_nodes)?;
    read_points2(&mut reader, &mut nodes)?;
    let polygon = Polygon2D::from_nodes(&nodes);
    polygon.set_deflection(deflection);
    Ok(Handle::new(polygon))
}
