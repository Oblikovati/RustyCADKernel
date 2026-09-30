// Rust translation of BinTools_ShapeSet::WritePolygon3D, WritePolygonOnTriangulation
// and WriteTriangulation (BinTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Writing the mesh sections of the grouped binary format.
use super::errors::{null_geometry, wrap_failure};
use super::format_version;
use super::scalars::{put_bool, put_integer, put_real, put_short_real};
use super::shape_set::BinShapeSet;
use super::shape_set_base::BinShapeSetBase;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};

/// The `<Tag> n\n` header of a mesh section.
fn write_header(stream: &mut IoStream, tag: &[u8], count: i32) {
    stream.insert_bytes(tag).insert_bytes(b" ");
    stream.insert_integer(i64::from(count)).insert_bytes(b"\n");
}

/// The object of a stored handle; Standard_NullObject for a null one, which the
/// native writer dereferences.
fn required<'h, T: RuntimeTyped>(handle: &'h Handle<T>, class: &str) -> Result<&'h T, Failure> {
    handle
        .get()
        .ok_or_else(|| null_geometry("BinTools_ShapeSet", class))
}

fn put_point3(stream: &mut IoStream, point: Point3) {
    for value in [point.x(), point.y(), point.z()] {
        put_real(stream, value);
    }
}

/// The records of one section: `count` records through `write_record`, stopping when
/// the progress indicator breaks; failures are wrapped in `context`.
fn write_records(
    progress: &ProgressRange<'_>,
    scope_name: &'static str,
    count: i32,
    context: &str,
    mut write_record: impl FnMut(i32) -> Result<(), Failure>,
) -> Result<(), Failure> {
    wrap_failure(context, || {
        let scope = ProgressScope::with_literal(progress, scope_name, f64::from(count), false);
        let mut index = 1;
        while index <= count && scope.more() {
            write_record(index)?;
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    })
}

impl BinShapeSet {
    /// `WritePolygon3D(OS, theRange)`: `Polygon3D n\n`, then for each polygon its node
    /// count, parameter flag, deflection, nodes and parameters;
    /// `set.write_polygon3d(&mut stream, &ProgressRange::default())?`.
    pub fn write_polygon3d(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let count = self.polygons3d.extent();
        write_header(stream, b"Polygon3D", count);
        let context = "EXCEPTION in BinTools_ShapeSet::WritePolygon3D(..)";
        write_records(progress, "Writing polygons 3D", count, context, |index| {
            write_polygon3d_record(
                stream,
                required(&self.polygons3d.get(index)?, "Poly_Polygon3D")?,
            )
        })
    }

    /// `WritePolygonOnTriangulation(OS, theRange)`: `PolygonOnTriangulations n\n`, then
    /// for each polygon its node count, node indices, deflection, and a flag followed by
    /// the parameters when it has them;
    /// `set.write_polygon_on_triangulation(&mut stream, &ProgressRange::default())?`.
    pub fn write_polygon_on_triangulation(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let count = self.nodes.extent();
        write_header(stream, b"PolygonOnTriangulations", count);
        let context = "EXCEPTION in BinTools_ShapeSet::WritePolygonOnTriangulation(..)";
        write_records(
            progress,
            "Writing polygons on triangulation",
            count,
            context,
            |index| {
                write_nodes_record(
                    stream,
                    required(&self.nodes.get(index)?, "Poly_PolygonOnTriangulation")?,
                )
            },
        )
    }

    /// `WriteTriangulation(OS, theRange)`: `Triangulations n\n`, then for each
    /// triangulation its sizes, UV flag, (version 4) normal flag, deflection, nodes, UV
    /// nodes, triangles and (version 4) normals as floats. Normals are written only when
    /// the triangulation has them and its table flag asks for them;
    /// `set.write_triangulation(&mut stream, &ProgressRange::default())?`.
    pub fn write_triangulation(
        &self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let count = self.triangulations.extent();
        write_header(stream, b"Triangulations", count);
        let with_normal_flag = self.format_nb() >= format_version::VERSION_4;
        let context = "EXCEPTION in BinTools_ShapeSet::WriteTriangulation(..)";
        write_records(progress, "Writing triangulation", count, context, |index| {
            let (triangulation, need_normals) = self.triangulations.get(index)?;
            let normals = triangulation.has_normals() && need_normals;
            write_triangulation_record(
                stream,
                required(&triangulation, "Poly_Triangulation")?,
                with_normal_flag.then_some(normals),
            )
        })
    }
}

fn write_polygon3d_record(stream: &mut IoStream, polygon: &Polygon3D) -> Result<(), Failure> {
    let count = polygon.nb_nodes();
    put_integer(stream, count);
    put_bool(stream, polygon.has_parameters());
    put_real(stream, polygon.deflection());
    let nodes = polygon.nodes();
    for index in 1..=count {
        put_point3(
            stream,
            *nodes
                .value(index)
                .map_err(|e| e.into_failure("NCollection_Array1::Value"))?,
        );
    }
    if polygon.has_parameters() {
        let parameters = polygon.parameters()?;
        for index in 1..=count {
            put_real(
                stream,
                *parameters
                    .value(index)
                    .map_err(|e| e.into_failure("NCollection_Array1::Value"))?,
            );
        }
    }
    Ok(())
}

fn write_nodes_record(
    stream: &mut IoStream,
    polygon: &PolygonOnTriangulation,
) -> Result<(), Failure> {
    let nodes = polygon.nodes();
    let length = nodes.len() as i32;
    put_integer(stream, length);
    for index in 1..=length {
        put_integer(
            stream,
            *nodes
                .value(index)
                .map_err(|e| e.into_failure("NCollection_Array1::Value"))?,
        );
    }
    put_real(stream, polygon.deflection());
    let parameters = polygon.parameters();
    let Some(parameters) = parameters.get() else {
        put_bool(stream, false);
        return Ok(());
    };
    put_bool(stream, true);
    for index in 1..=parameters.len() as i32 {
        put_real(stream, parameters.value(index)?);
    }
    Ok(())
}

/// One triangulation; `normals` is `None` before version 4 (no normal flag), else
/// whether the normals are written.
fn write_triangulation_record(
    stream: &mut IoStream,
    triangulation: &Triangulation,
    normals: Option<bool>,
) -> Result<(), Failure> {
    let (nb_nodes, nb_triangles) = (triangulation.nb_nodes(), triangulation.nb_triangles());
    put_integer(stream, nb_nodes);
    put_integer(stream, nb_triangles);
    put_bool(stream, triangulation.has_uv_nodes());
    if let Some(normals) = normals {
        put_bool(stream, normals);
    }
    put_real(stream, triangulation.deflection());
    for index in 1..=nb_nodes {
        put_point3(stream, triangulation.node(index)?);
    }
    if triangulation.has_uv_nodes() {
        for index in 1..=nb_nodes {
            let uv = triangulation.uv_node(index)?;
            put_real(stream, uv.x());
            put_real(stream, uv.y());
        }
    }
    for index in 1..=nb_triangles {
        let (a, b, c) = triangulation.triangle(index)?.get();
        for node in [a, b, c] {
            put_integer(stream, node);
        }
    }
    if normals == Some(true) {
        for index in 1..=nb_nodes {
            let normal = triangulation.normal_vec3(index)?;
            for component in [normal.x, normal.y, normal.z] {
                put_short_real(stream, component);
            }
        }
    }
    Ok(())
}
