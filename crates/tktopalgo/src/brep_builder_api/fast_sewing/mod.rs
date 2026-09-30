// Rust translation of BRepBuilderAPI_FastSewing.hxx and of the constructor, both Add,
// Perform, GetStatuses and Compute3DRange of BRepBuilderAPI_FastSewing.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_FastSewing`: sews naturally bounded, finite surfaces whose common
//! boundaries have the same parameterization, by merging their corner points.
mod topology;

use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::Quilt;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::CellFilter;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::integer::integer_last;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use std::io::Write;
use topology::{FsEdge, FsFace, FsVertex, NodeInspector};

/// `BRepBuilderAPI_FastSewing::FS_VARStatuses`: the set bits of `FS_Statuses`.
pub type FastSewingStatuses = u32;

/// `BRepBuilderAPI_FastSewing::FS_Statuses`, with the native bit values;
/// `assert_eq!(FastSewingStatus::EmptyInput as u32, 0x40)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FastSewingStatus {
    /// `FS_OK`.
    Ok = 0x00,
    /// `FS_Degenerated`: an edge of a face is degenerated.
    Degenerated = 0x01,
    /// `FS_FindVertexError` (never set natively).
    FindVertexError = 0x02,
    /// `FS_FindEdgeError` (never set natively).
    FindEdgeError = 0x04,
    /// `FS_FaceWithNullSurface`.
    FaceWithNullSurface = 0x08,
    /// `FS_NotNaturalBoundsFace`: a rectangular trimmed surface.
    NotNaturalBoundsFace = 0x10,
    /// `FS_InfiniteSurface`.
    InfiniteSurface = 0x20,
    /// `FS_EmptyInput`.
    EmptyInput = 0x40,
    /// `FS_Exception`: an exception during Perform.
    Exception = 0x80,
}

/// The message `GetStatuses` prints for a status bit, None for a bit it stops at.
fn status_message(bit: u32) -> Option<&'static str> {
    Some(match bit {
        0x01 => "Degenerated case. Try to reduce tolerance.\n",
        0x02 => "Error while creating list of vertices.\n",
        0x04 => "Error while creating list of edges.\n",
        0x80 => "Exception during the operation.\n",
        0x08 => "Source face has null surface.\n",
        0x10 => "Source face has trimmed surface.\n",
        0x20 => "Source face has the surface with infinite boundaries.\n",
        0x40 => "Empty source data.\n",
        _ => return None,
    })
}

/// `BRepBuilderAPI_FastSewing` (docs/topalgo-design.md section 11): add faces or
/// surfaces, `perform`, then read the result and the statuses.
///
/// ```
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIFastSewing, FastSewingStatus};
/// let mut sewing = BRepBuilderAPIFastSewing::new(1e-6);
/// sewing.perform();
/// assert_eq!(sewing.get_statuses(None).unwrap(), FastSewingStatus::EmptyInput as u32);
/// ```
pub struct BRepBuilderAPIFastSewing {
    parent: StandardTransient,
    my_res_shape: Shape,
    my_face_vec: Vec<FsFace>,
    my_vertex_vec: Vec<FsVertex>,
    my_edge_vec: Vec<FsEdge>,
    my_tolerance: f64,
    my_status_list: FastSewingStatuses,
}
impl_transient!(
    BRepBuilderAPIFastSewing,
    "BRepBuilderAPI_FastSewing",
    StandardTransient,
    parent
);

impl Default for BRepBuilderAPIFastSewing {
    /// `BRepBuilderAPI_FastSewing()`: tolerance 1e-6.
    fn default() -> Self {
        Self::new(1.0e-06)
    }
}

/// The bounds of a surface to sew, or the status that rejects it.
fn sewable_bounds(
    surface: &Handle<dyn GeomSurface>,
) -> Result<(f64, f64, f64, f64), FastSewingStatus> {
    let Some(s) = surface.get() else {
        return Err(FastSewingStatus::FaceWithNullSurface);
    };
    if surface
        .cast::<GeomRectangularTrimmedSurface>()
        .get()
        .is_some()
    {
        return Err(FastSewingStatus::NotNaturalBoundsFace);
    }
    let (uf, ul, vf, vl) = s.bounds();
    if [uf, ul, vf, vl].into_iter().any(precision::is_infinite) {
        return Err(FastSewingStatus::InfiniteSurface);
    }
    Ok((uf, ul, vf, vl))
}

impl BRepBuilderAPIFastSewing {
    /// `BRepBuilderAPI_FastSewing(theTolerance)`: the tolerance of connexity and no
    /// status; `BRepBuilderAPIFastSewing::new(1e-6)`.
    pub fn new(tolerance: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_res_shape: Shape::default(),
            my_face_vec: Vec::new(),
            my_vertex_vec: Vec::new(),
            my_edge_vec: Vec::new(),
            my_tolerance: tolerance,
            my_status_list: 0,
        }
    }

    /// `SetStatus(theStatus)`: sets a status bit and returns its value.
    fn set_status(&mut self, status: FastSewingStatus) -> FastSewingStatuses {
        let id = status as u32;
        self.my_status_list |= id;
        id
    }

    /// `Add(theShape)`: each face (once) whose surface is not null, not rectangular
    /// trimmed and finite; the others set their status. False when no face was added
    /// (FS_EmptyInput for a null shape); `sewing.add_shape(&shape)?`.
    pub fn add_shape(&mut self, shape: &Shape) -> Result<bool, Failure> {
        if shape.is_null() {
            self.set_status(FastSewingStatus::EmptyInput);
            return Ok(false);
        }
        let mut result = false;
        let mut map = MapOfShape::default();
        for f in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if !map.add(f.clone()) {
                continue;
            }
            let face: Face = topods::face(&f)?;
            if let Err(status) = sewable_bounds(&BRepTool::surface(&face)?) {
                self.set_status(status);
                continue;
            }
            self.my_face_vec.push(FsFace::new(face));
            result = true;
        }
        Ok(result)
    }

    /// `Add(theSurface)`: a face naturally bounded by the surface (checked as by
    /// `add_shape`); `sewing.add_surface(&surface)?`.
    pub fn add_surface(&mut self, surface: &Handle<dyn GeomSurface>) -> Result<bool, Failure> {
        if let Err(status) = sewable_bounds(surface) {
            self.set_status(status);
            return Ok(false);
        }
        let b = BRepBuilder::default();
        let mut face = Face::default();
        b.make_face(&mut face);
        b.make_face_on_surface(&mut face, surface, self.my_tolerance)?;
        b.natural_restriction(&face, true)?;
        self.my_face_vec.push(FsFace::new(face));
        Ok(true)
    }

    /// `Perform()`: FS_EmptyInput without faces; otherwise the corner points become
    /// shared vertices, the sides shared edges, and the faces are quilted into shells.
    /// An exception sets FS_Exception, as the native catch; `sewing.perform()`.
    pub fn perform(&mut self) {
        if self.my_face_vec.is_empty() {
            self.set_status(FastSewingStatus::EmptyInput);
            return;
        }
        if self.perform_steps().is_err() {
            self.set_status(FastSewingStatus::Exception);
        }
    }

    /// The `try` block of `Perform`.
    fn perform_steps(&mut self) -> Result<(), Failure> {
        let range = self.compute_3d_range()?;
        let mut cells: CellFilter<NodeInspector> = CellFilter::new(std_max(
            self.my_tolerance,
            range / f64::from(integer_last()),
        ));
        let mut points: Vec<Point3> = Vec::new();
        for i in 0..self.my_face_vec.len() {
            self.find_vertexes(i, &mut cells, &mut points)?;
        }
        for i in 0..self.my_face_vec.len() {
            self.find_edges(i)?;
        }
        for vertex in &mut self.my_vertex_vec {
            vertex.create_topological_vertex(self.my_tolerance)?;
        }
        for i in 0..self.my_edge_vec.len() {
            let edge = self.my_edge_vec[i].create_topological_edge(
                &self.my_vertex_vec,
                &self.my_face_vec,
                self.my_tolerance,
            )?;
            self.my_edge_vec[i].my_topo_edge = edge;
        }
        let mut quilt = Quilt::new();
        for face in &mut self.my_face_vec {
            face.create_topological_wire(&self.my_edge_vec, self.my_tolerance)?;
            face.create_topological_face()?;
            quilt.add(&face.my_ret_face)?;
        }
        self.my_res_shape = quilt.shells()?;
        Ok(())
    }

    /// `SetTolerance(theToler)`; `sewing.set_tolerance(1e-3)`.
    pub fn set_tolerance(&mut self, toler: f64) {
        self.my_tolerance = toler;
    }

    /// `GetTolerance()`; `sewing.get_tolerance()`.
    pub fn get_tolerance(&self) -> f64 {
        self.my_tolerance
    }

    /// `GetResult()`: the sewn shells (a null shape before Perform);
    /// `sewing.get_result().is_null()`.
    pub fn get_result(&self) -> &Shape {
        &self.my_res_shape
    }

    /// `GetStatuses(theOS)`: the status bits; with `out`, one message per set bit in
    /// bit order ("Fast Sewing OK!" when none); `sewing.get_statuses(None)?`.
    pub fn get_statuses(&self, out: Option<&mut dyn Write>) -> Result<FastSewingStatuses, Failure> {
        let Some(out) = out else {
            return Ok(self.my_status_list);
        };
        let io = |e: std::io::Error| Failure::new(&e.to_string());
        if self.my_status_list == 0 {
            out.write_all(b"Fast Sewing OK!\n").map_err(io)?;
            return Ok(self.my_status_list);
        }
        for i in 0..u32::BITS {
            let bit: u32 = 1 << i;
            if bit & self.my_status_list == 0 {
                continue;
            }
            let Some(message) = status_message(bit) else {
                return Ok(self.my_status_list);
            };
            out.write_all(message.as_bytes()).map_err(io)?;
        }
        Ok(self.my_status_list)
    }

    /// `Compute3DRange()`: the largest side of the box of the corner points of the
    /// surfaces.
    fn compute_3d_range(&self) -> Result<f64, Failure> {
        let mut bounds = BoundingBox3::default();
        for face in &self.my_face_vec {
            let surface = BRepTool::surface(&face.my_src_face)?;
            let Some(s) = surface.get() else { continue };
            let (uf, ul, vf, vl) = s.bounds();
            for (u, v) in [(uf, vf), (ul, vf), (ul, vl), (uf, vl)] {
                bounds.add_point(s.value(u, v)?);
            }
        }
        let [xm, ym, zm, x_max, y_max, z_max] = bounds.get()?;
        let mut delta = x_max - xm;
        delta = std_max(delta, y_max - ym);
        delta = std_max(delta, z_max - zm);
        Ok(delta)
    }
}
