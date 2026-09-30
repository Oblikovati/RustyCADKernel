// Rust translation of BRepTools_ShapeSet::WriteTriangulation, DumpTriangulation and
// ReadTriangulation (ModelingData/TKBRep/BRepTools/BRepTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Triangulations` section of the ASCII format.
use super::BRepToolsShapeSet;
use super::polygon_io::{
    compact_header, dump_header, dump_rank, read_section_count, write_coordinates,
};
use crate::toptools::{ShapeSet, format_version};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools::get_real;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::poly::{Triangle, Triangulation};

/// `"with"` or `"without"`.
fn with_word(flag: bool) -> &'static [u8] {
    if flag { b"with" } else { b"without" }
}

impl BRepToolsShapeSet {
    /// `WriteTriangulation(OS, Compact, theProgress)`: the triangulations, compact (the
    /// layout `ReadTriangulation` accepts) or readable; the normals are written from
    /// version 3 on when the triangulation has them and its flag asks for them;
    /// `set.write_triangulation(&mut s, true, &range)?`.
    pub fn write_triangulation(
        &self,
        stream: &mut IoStream,
        compact: bool,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let count = self.triangulations.extent();
        let scope =
            ProgressScope::with_literal(progress, "Triangulations", f64::from(count), false);
        if compact {
            compact_header(stream, count, b"Triangulations");
        } else {
            dump_header(stream, count, b"Triangulations");
        }
        let mut index = 1;
        while index <= count && scope.more() {
            let (triangulation, with_normals) = self.triangulations.get(index)?;
            let triangulation = triangulation.get().ok_or_else(|| {
                crate::brep::errors::null_object(
                    "BRepTools_ShapeSet::WriteTriangulation: null triangulation",
                )
            })?;
            self.write_triangulation_record(stream, index, triangulation, with_normals, compact)?;
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }

    /// `DumpTriangulation(OS)`: `WriteTriangulation(OS, false)`; `set.dump_triangulation(&mut s)?`.
    pub fn dump_triangulation(&self, stream: &mut IoStream) -> Result<(), Failure> {
        self.write_triangulation(stream, false, &ProgressRange::default())
    }

    fn write_triangulation_record(
        &self,
        stream: &mut IoStream,
        index: i32,
        triangulation: &Triangulation,
        with_normals: bool,
        compact: bool,
    ) -> Result<(), Failure> {
        let normals = triangulation.has_normals() && with_normals;
        let version3 = self.format_nb() >= format_version::VERSION_3;
        write_triangulation_header(stream, (index, triangulation), normals, version3, compact);
        if !compact {
            stream.insert_bytes(b"  Deflection : ");
        }
        stream.insert_real(triangulation.deflection());
        stream.insert_bytes(b"\n");
        write_nodes(stream, triangulation, compact)?;
        write_triangles(stream, triangulation, compact)?;
        if version3 && normals {
            write_normals(stream, triangulation, compact)?;
        }
        stream.insert_bytes(b"\n");
        Ok(())
    }

    /// `ReadTriangulation(IS, theProgress)`: appends the triangulations of a
    /// `Triangulations n` section (normals from version 3 on), each with its normal
    /// flag; another word reads nothing; `set.read_triangulation(&mut s, &range)?`.
    pub fn read_triangulation(
        &mut self,
        stream: &mut IoStream,
        progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let Some(count) = read_section_count(stream, b"Triangulations") else {
            return Ok(());
        };
        let version3 = self.format_nb() >= format_version::VERSION_3;
        let scope =
            ProgressScope::with_literal(progress, "Triangulations", f64::from(count), false);
        let mut has_normals = false;
        let mut index = 1;
        while index <= count && scope.more() {
            let triangulation = read_triangulation_record(stream, version3, &mut has_normals)?;
            self.triangulations
                .add(&Handle::new(triangulation), has_normals);
            index += 1;
            drop(scope.next(1.0));
        }
        Ok(())
    }
}

/// The sizes and flags line (compact) or the readable header of one triangulation.
fn write_triangulation_header(
    stream: &mut IoStream,
    (index, triangulation): (i32, &Triangulation),
    normals: bool,
    version3: bool,
    compact: bool,
) {
    let (nb_nodes, nb_triangles) = (triangulation.nb_nodes(), triangulation.nb_triangles());
    if compact {
        stream
            .insert_integer(i64::from(nb_nodes))
            .insert_bytes(b" ");
        stream
            .insert_integer(i64::from(nb_triangles))
            .insert_bytes(b" ");
        stream.insert_bytes(if triangulation.has_uv_nodes() {
            b"1 "
        } else {
            b"0 "
        });
        if version3 {
            stream.insert_bytes(if normals { b"1 " } else { b"0 " });
        }
        return;
    }
    stream.insert_bytes(b"  ");
    stream.insert_integer(i64::from(index));
    stream.insert_bytes(b" : Triangulation with ");
    stream.insert_integer(i64::from(nb_nodes));
    stream.insert_bytes(b" Nodes and ");
    stream.insert_integer(i64::from(nb_triangles));
    stream.insert_bytes(b" Triangles\n      ");
    stream.insert_bytes(with_word(triangulation.has_uv_nodes()));
    stream.insert_bytes(b" UV nodes\n");
    if version3 {
        stream.insert_bytes(b"      ");
        stream.insert_bytes(with_word(normals));
        stream.insert_bytes(b" normals\n");
    }
}

/// The 3D nodes, then the UV nodes when there are some.
fn write_nodes(
    stream: &mut IoStream,
    triangulation: &Triangulation,
    compact: bool,
) -> Result<(), Failure> {
    if !compact {
        stream.insert_bytes(b"\n3D Nodes :\n");
    }
    for rank in 1..=triangulation.nb_nodes() {
        let node = triangulation.node(rank)?;
        dump_rank(stream, rank, compact);
        write_coordinates(stream, &[node.x(), node.y(), node.z()], compact);
    }
    if !triangulation.has_uv_nodes() {
        return Ok(());
    }
    if !compact {
        stream.insert_bytes(b"\nUV Nodes :\n");
    }
    for rank in 1..=triangulation.nb_nodes() {
        let node = triangulation.uv_node(rank)?;
        dump_rank(stream, rank, compact);
        write_coordinates(stream, &[node.x(), node.y()], compact);
    }
    Ok(())
}

/// The node triples of the triangles (`std::setw(10)` each in the dump layout).
fn write_triangles(
    stream: &mut IoStream,
    triangulation: &Triangulation,
    compact: bool,
) -> Result<(), Failure> {
    if !compact {
        stream.insert_bytes(b"\nTriangles :\n");
    }
    for rank in 1..=triangulation.nb_triangles() {
        dump_rank(stream, rank, compact);
        let (n1, n2, n3) = triangulation.triangle(rank)?.get();
        for (position, node) in [n1, n2, n3].into_iter().enumerate() {
            if !compact {
                stream.set_width(10);
            }
            stream.insert_integer(i64::from(node));
            if position < 2 {
                stream.insert_bytes(b" ");
            }
        }
        stream.insert_bytes(if compact { b" " } else { b"\n" });
    }
    Ok(())
}

/// The single-precision normals, each component followed by a space (compact) or a
/// line end; in the dump layout only the first component of a node is padded.
fn write_normals(
    stream: &mut IoStream,
    triangulation: &Triangulation,
    compact: bool,
) -> Result<(), Failure> {
    if !compact {
        stream.insert_bytes(b"\nNormals :\n");
    }
    for rank in 1..=triangulation.nb_nodes() {
        if !compact {
            stream.set_width(10);
            stream.insert_integer(i64::from(rank));
            stream.insert_bytes(b" : ");
            stream.set_width(17);
        }
        let normal: Vec3<f32> = triangulation.normal_vec3(rank)?;
        for component in [normal.x, normal.y, normal.z] {
            stream.insert_real(f64::from(component));
            stream.insert_bytes(if compact { b" " } else { b"\n" });
        }
    }
    Ok(())
}

/// One compact triangulation record; `has_normals` keeps its value across records
/// before version 3, as the native local does.
fn read_triangulation_record(
    stream: &mut IoStream,
    version3: bool,
    has_normals: &mut bool,
) -> Result<Triangulation, Failure> {
    let (mut nb_nodes, mut nb_triangles, mut has_uv) = (0, 0, false);
    stream.extract_int(&mut nb_nodes);
    stream.extract_int(&mut nb_triangles);
    stream.extract_bool(&mut has_uv);
    if version3 {
        stream.extract_bool(has_normals);
    }
    let deflection = get_real(stream);
    let triangulation = Triangulation::with_sizes(nb_nodes, nb_triangles, has_uv, *has_normals)?;
    for rank in 1..=nb_nodes {
        let (x, y, z) = (get_real(stream), get_real(stream), get_real(stream));
        triangulation.set_node(rank, Point3::new(x, y, z))?;
    }
    if has_uv {
        for rank in 1..=nb_nodes {
            let (u, v) = (get_real(stream), get_real(stream));
            triangulation.set_uv_node(rank, Point2::new(u, v))?;
        }
    }
    for rank in 1..=nb_triangles {
        let (mut n1, mut n2, mut n3) = (0, 0, 0);
        stream.extract_int(&mut n1);
        stream.extract_int(&mut n2);
        stream.extract_int(&mut n3);
        triangulation.set_triangle(rank, Triangle::new(n1, n2, n3))?;
    }
    if *has_normals {
        for rank in 1..=nb_nodes {
            let (x, y, z) = (get_real(stream), get_real(stream), get_real(stream));
            // `NCollection_Vec3<float>(aNorm)`: each double component narrowed to float.
            triangulation.set_normal_vec3(rank, Vec3::new(x as f32, y as f32, z as f32))?;
        }
    }
    triangulation.set_deflection(deflection);
    Ok(triangulation)
}
