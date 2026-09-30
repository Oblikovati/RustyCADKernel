// Rust translation of ModelingAlgorithms/TKHelix/HelixBRep/HelixBRep_BuilderHelix.cxx
// (Perform, BuildPart, Smoothing and SmoothingEdges) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The build of `HelixBRep_BuilderHelix`: one wire per part from the coils of
//! `HelixGeom_BuilderHelix`, the tangents smoothed across the part joints, and one wire
//! of all the parts.
use super::builder_helix::HelixBRepBuilderHelix;
use crate::helix_geom::errors::{array_failure, null_object};
use crate::helix_geom::{
    HelixGeomBuilderApproxCurve, HelixGeomBuilderHelix, HelixGeomBuilderHelixGen,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Line3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use std::f64::consts::PI;

/// The geometry of one part: its axis, start point, height, pitch, taper and sense.
pub(super) struct HelixPart {
    pub(super) axis: Axis1,
    pub(super) p_start: Point3,
    pub(super) height: f64,
    pub(super) pitch: f64,
    pub(super) taper_angle: f64,
    pub(super) is_clockwise: bool,
}

/// `std::max(a, b)`: `(a < b) ? b : a`.
fn native_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

// `theArray->Value(i)` of a parameter array (a set array; Standard_OutOfRange outside
// its bounds).
fn value_at<T: Copy>(array: &Option<Array1<'static, T>>, index: i32) -> Result<T, Failure> {
    let array = array
        .as_ref()
        .ok_or_else(|| null_object("NCollection_HArray1"))?;
    array
        .value(index)
        .copied()
        .map_err(array_failure("NCollection_Array1::Value"))
}

impl HelixBRepBuilderHelix {
    /// `Perform()`: nothing without valid parameters (error status not 0); otherwise
    /// builds each part from the end of the previous one, smooths the joints and makes
    /// the wire. A part error stops the build with its status; an exception (an array
    /// index out of its bounds, no part, a failing wire) propagates with the reached
    /// tolerance reset to 0 and the shape null; `builder.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if self.error_status != 0 {
            return Ok(());
        }
        self.tol_reached = 0.;
        self.shape.nullify();
        let mut parts: Vec<Wire> = Vec::new();
        let mut axis: Axis1 = self.axis3.axis();
        let mut p_start: Point3 = self.axis3.location();
        let half_diam: f64 = 0.5 * value_at(&self.diams, 1)?;
        p_start.translate(Vector3::from(self.axis3.x_direction()) * half_diam);
        let is_clockwise: bool = self.axis3.direct()?;
        for i in 1..=self.n_parts {
            let part = self.part_geometry(i, axis, p_start, is_clockwise)?;
            let wire: Wire = self.build_part(&part)?;
            if self.error_status != 0 {
                return Ok(());
            }
            let (_, v2): (Vertex, Vertex) = topexp::wire_vertices(&wire);
            p_start = BRepTool::pnt(&v2)?;
            axis.set_location(
                axis.location()
                    .translated(Vector3::from(axis.direction()) * part.height),
            );
            parts.push(wire);
        }
        self.smoothing(&parts)?;
        self.shape = Self::joined_wire(&parts)?;
        Ok(())
    }

    // The height, pitch and taper of part `i` (a pitch flag false makes the pitch value
    // a number of turns).
    fn part_geometry(
        &self,
        i: i32,
        axis: Axis1,
        p_start: Point3,
        is_clockwise: bool,
    ) -> Result<HelixPart, Failure> {
        let height: f64 = value_at(&self.heights, i)?;
        let pitch: f64 = if value_at(&self.is_pitches, i)? {
            value_at(&self.pitches, i)?
        } else {
            height / value_at(&self.pitches, i)?
        };
        let (d_next, d_this) = (value_at(&self.diams, i + 1)?, value_at(&self.diams, i)?);
        let taper_angle: f64 = (0.5 * (d_next - d_this) / height).atan();
        Ok(HelixPart {
            axis,
            p_start,
            height,
            pitch,
            taper_angle,
            is_clockwise,
        })
    }

    // BRepBuilderAPI_MakeWire of the first part, then Add of each other one; its Shape().
    fn joined_wire(parts: &[Wire]) -> Result<rustycad_tkbrep::topods::Shape, Failure> {
        let first: &Wire = parts.first().ok_or_else(no_such_object)?;
        let mut mk_wire = BRepBuilderAPIMakeWire::from_wire(first)?;
        for part in &parts[1..] {
            mk_wire.add_wire(part)?;
        }
        Ok(mk_wire.shape()?.clone())
    }

    // The validity checks of BuildPart, in the native order: the error status of a
    // refused part, if any; `dist` is the distance from the start point to the axis.
    fn part_error(&self, part: &HelixPart, dist: f64) -> i32 {
        let tol_prec: f64 = self.tolerance;
        let tol_ang: f64 = 1.0e-7;
        if part.taper_angle > PI / 2. - tol_ang {
            return 13; // invalid TaperAngle value
        }
        if part.height < tol_prec {
            return 12; // invalid Height value
        }
        if part.pitch < tol_prec {
            return 11; // invalid Pitch value
        }
        if dist < tol_prec {
            return 10; // myPStart belongs to the myAxis
        }
        0
    }

    // BuildPart: the checks, the coils of HelixGeom_BuilderHelix in the frame at the
    // foot of the start point, and one edge per coil. Deviation: the native branches
    // for a start angle other than 0 (a trimmed first coil, coils before the start) are
    // dead, since the angle is the local constant 0, and are not ported.
    fn build_part(&mut self, part: &HelixPart) -> Result<Wire, Failure> {
        let mut the_part = Wire::default();
        (self.error_status, self.warning_status) = (0, 0);
        // gp_Lin(theAxis).Distance(thePStart); computing it before the checks that do
        // not use it changes nothing.
        let dist: f64 = Line3::from_axis(part.axis).distance(part.p_start);
        self.error_status = self.part_error(part, dist);
        if self.error_status != 0 {
            return Ok(the_part);
        }
        let mut bh = self.coil_builder(part, dist)?;
        bh.perform()?;
        if bh.error_status() != 0 {
            self.error_status = 2;
            return Ok(the_part);
        }
        let builder = BRepBuilder::default();
        builder.make_wire(&mut the_part);
        self.tol_reached = native_max(self.tol_reached, bh.tolerance_reached());
        if !self.add_coil_edges(&mut the_part, bh.curves().iter())? {
            return Ok(the_part);
        }
        if self.tol_reached > self.tolerance {
            self.warning_status = 1;
        }
        Ok(the_part)
    }

    // The HelixGeom_BuilderHelix of a part: position at the foot M1 of the start point
    // on the axis, X towards the start point, and [0, height / (pitch / 2*PI)].
    fn coil_builder(&self, part: &HelixPart, dist: f64) -> Result<HelixGeomBuilderHelix, Failure> {
        let dir: Direction3 = part.axis.direction();
        let vec1 = Vector3::from(dir);
        let m0: Point3 = part.axis.location();
        let dm: f64 = vec1.dot(Vector3::between(m0, part.p_start));
        let m1: Point3 = m0.translated(vec1 * dm);
        let dir_x: Direction3 = Direction3::from_vector(Vector3::between(m1, part.p_start))?;
        let ax2 = Axis2::with_x_direction(m1, dir, dir_x)?;
        let c1: f64 = part.pitch / (2. * PI);
        let (t0, t2): (f64, f64) = (0., part.height / c1);
        let mut bh = HelixGeomBuilderHelix::new();
        bh.set_position(ax2);
        bh.set_curve_parameters(
            t0,
            t2,
            part.pitch,
            dist,
            part.taper_angle,
            part.is_clockwise,
        );
        bh.set_tolerance(self.tolerance);
        bh.set_approx_parameters(self.continuity, self.max_degree, self.max_segments);
        Ok(bh)
    }

    // One edge per coil, between vertices of the reached tolerance, added to the part;
    // false (error status 3) when an edge cannot be made.
    fn add_coil_edges<'c>(
        &mut self,
        the_part: &mut Wire,
        curves: impl Iterator<Item = &'c Handle<dyn GeomCurve>>,
    ) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        let mut v1 = Vertex::default();
        for (index, c) in curves.enumerate() {
            let curve: &dyn GeomCurve = c.get().ok_or_else(|| null_object("aC"))?;
            if index == 0 {
                v1 = self.coil_vertex(curve.d0(curve.first_parameter())?, Orientation::Forward)?;
            }
            let v2 = self.coil_vertex(curve.d0(curve.last_parameter())?, Orientation::Reversed)?;
            let mut bme = BRepBuilderAPIMakeEdge::new();
            bme.init_curve_vertices(c, &v1, &v2)?;
            if !bme.is_done() {
                self.error_status = 3;
                return Ok(false);
            }
            let edge: Edge = bme.edge()?;
            builder.update_edge_tolerance(&edge, self.tol_reached)?;
            builder.add(the_part, &edge)?;
            v1 = v2;
            v1.set_orientation(Orientation::Forward);
        }
        Ok(true)
    }

    // aBB.MakeVertex(V, P, myTolReached) with an orientation.
    fn coil_vertex(&self, point: Point3, orientation: Orientation) -> Result<Vertex, Failure> {
        let mut vertex = Vertex::default();
        BRepBuilder::default().make_vertex_at(&mut vertex, point, self.tol_reached)?;
        vertex.set_orientation(orientation);
        Ok(vertex)
    }
}

/// Standard_NoSuchObject of `NCollection_List::First` on an empty list.
pub(super) fn no_such_object() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "NCollection_List::First: empty list; expected at least one helix part",
        "",
    )
}
