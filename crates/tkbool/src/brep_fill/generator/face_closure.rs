// Rust translation of the curves on surface, the wire and the history of each face of
// BRepFill_Generator::Perform (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Generator.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The p-curves of the four edges of a ruled face, its wire and its record in the
//! generated map; the copies of the input edges when the input is not mutable.
use super::BRepFillGenerator;
use super::kpart_detect::{KPART_NONE, KPART_PLANE};
use super::lateral::FaceFrame;
use super::ruled_face::FaceSections;
use super::section_pair::SectionPair;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction2, Point2};

/// A 2d line through `(u, v)` (`new Geom2d_Line(gp_Pnt2d(u, v), direction)`).
fn line2d(u: f64, v: f64, direction: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(Point2::new(u, v), direction)).cast::<dyn Geom2dCurve>()
}

/// A 2d Bezier segment (`new Geom2d_BezierCurve(Extrem)`).
fn segment2d(start: Point2, end: Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut poles = one_based::<Point2>(2)?;
    put(&mut poles, 1, start)?;
    put(&mut poles, 2, end)?;
    Ok(Handle::new(Geom2dBezierCurve::new(&poles)?).cast::<dyn Geom2dCurve>())
}

/// The p-curves of the section edges (iso V lines, not on a plane) and of the
/// lateral edges (iso U lines in the general case, segments for a particular case,
/// both on the seam of a periodic face).
pub(super) fn set_pcurves(
    s: &FaceSections,
    face: &Face,
    frame: &FaceFrame,
    laterals: (&Edge, &Edge),
) -> Result<(), Failure> {
    if frame.itype != KPART_PLANE {
        section_pcurve(&s.edge1, face, frame, frame.v_first)?;
        section_pcurve(&s.edge2, face, frame, frame.v_last)?;
    }
    let builder = BRepBuilder::default();
    let t: f64 = precision::confusion();
    let (f1, l1, f2, l2) = (frame.u_first, frame.u_last, frame.v_first, frame.v_last);
    let (edge3, edge4) = laterals;
    if frame.itype == KPART_NONE {
        if s.periodic {
            let (c1, c2) = (line2d(l1, 0., Direction2::Y), line2d(f1, 0., Direction2::Y));
            return builder.update_edge_pcurves_on_face(edge3, (&c1, &c2), face, t);
        }
        builder.update_edge_pcurve_on_face(edge3, &line2d(f1, 0., Direction2::Y), face, t)?;
        return builder.update_edge_pcurve_on_face(edge4, &line2d(l1, 0., Direction2::Y), face, t);
    }
    if s.periodic {
        let c1 = segment2d(Point2::new(l1, f2), Point2::new(l1, l2))?;
        let c2 = segment2d(Point2::new(f1, f2), Point2::new(f1, l2))?;
        return builder.update_edge_pcurves_on_face(edge3, (&c1, &c2), face, t);
    }
    if frame.itype == KPART_PLANE {
        return Ok(());
    }
    builder.update_edge_pcurve_on_face(
        edge3,
        &segment2d(Point2::new(f1, f2), Point2::new(f1, l2))?,
        face,
        t,
    )?;
    builder.update_edge_pcurve_on_face(
        edge4,
        &segment2d(Point2::new(l1, f2), Point2::new(l1, l2))?,
        face,
        t,
    )
}

/// The iso V = `v` p-curve of a section edge, along -U on `[-l1, -f1]` for a
/// REVERSED edge and along +U on `[f1, l1]` otherwise.
fn section_pcurve(edge: &Edge, face: &Face, frame: &FaceFrame, v: f64) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let t: f64 = precision::confusion();
    let (f1, l1) = (frame.u_first, frame.u_last);
    if edge.orientation() == Orientation::Reversed {
        builder.update_edge_pcurve_on_face(edge, &line2d(0., v, Direction2::NEG_X), face, t)?;
        return builder.range_on_face(edge, face, -l1, -f1);
    }
    builder.update_edge_pcurve_on_face(edge, &line2d(0., v, Direction2::X), face, t)?;
    builder.range_on_face(edge, face, f1, l1)
}

impl BRepFillGenerator {
    /// The edges flagged non SameParameter and non SameRange, the wire
    /// `Edge1 Edge4 -Edge2 Edge3` (a degenerated section edge of a plane is left out)
    /// added to the face, and the face to the shell.
    pub(super) fn close_face(
        &mut self,
        s: &FaceSections,
        face: &mut Face,
        itype: i32,
        laterals: (&Edge, &Edge),
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let (edge3, edge4) = laterals;
        for edge in [&s.edge1, &s.edge2, edge3, edge4] {
            builder.same_parameter(edge, false)?;
        }
        for edge in [&s.edge1, &s.edge2, edge3, edge4] {
            builder.same_range(edge, false)?;
        }
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        if !s.degen1 || itype != KPART_PLANE {
            builder.add(&mut wire, &s.edge1)?;
        }
        builder.add(&mut wire, edge4)?;
        if !s.degen2 || itype != KPART_PLANE {
            builder.add(&mut wire, &s.edge2.reversed())?;
        }
        builder.add(&mut wire, edge3)?;
        builder.add(face, &wire)?;
        builder.add(&mut self.my_shell, face)
    }

    /// Appends the face to the faces generated by `red` (the native `aREd`), unless
    /// the first edge is a degenerated edge of a plane.
    pub(super) fn record_generated(
        &mut self,
        s: &FaceSections,
        itype: i32,
        red: &Edge,
        face: &Face,
    ) {
        if s.degen1 && itype == KPART_PLANE {
            return;
        }
        let key: Shape = red.as_ref().clone();
        if !self.my_map.is_bound(&key) {
            self.my_map.bind(key.clone(), List::default());
        }
        if let Some(faces) = self.my_map.change_seek(&key) {
            faces.append(face.as_ref().clone());
        }
    }

    /// With an immutable input: each section edge is replaced by its copy, made on
    /// first use (for the first wire only on the first side) when the p-curves or
    /// the flags will modify it.
    pub(super) fn copy_input_edges(
        &mut self,
        s: &mut FaceSections,
        pair: &SectionPair,
        first_wire: bool,
        itype: i32,
        modif_wires: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        if !s.degen1 {
            if let Some(copy) = self.my_old_new_shapes.seek(&s.edge1) {
                s.edge1 = topods::edge(copy)?;
            } else if first_wire
                && (itype != KPART_PLANE
                    || BRepTool::same_parameter(&s.edge1)?
                    || BRepTool::same_range(&s.edge1)?)
            {
                s.edge1 = self.create_new_edge(&s.edge1, &pair.wire1, modif_wires)?;
            }
        }
        if !s.degen2 {
            if let Some(copy) = self.my_old_new_shapes.seek(&s.edge2) {
                s.edge2 = topods::edge(copy)?;
            } else if itype != KPART_PLANE
                || BRepTool::same_parameter(&s.edge2)?
                || BRepTool::same_range(&s.edge2)?
            {
                s.edge2 = self.create_new_edge(&s.edge2, &pair.wire2, modif_wires)?;
            }
        }
        Ok(())
    }

    /// `CreateNewEdge`: an empty copy of the edge with its sub-shapes, bound in the
    /// old-new map; its wire is recorded as modified.
    fn create_new_edge(
        &mut self,
        edge: &Edge,
        wire: &Wire,
        modif_wires: &mut IndexedMapOfShape,
    ) -> Result<Edge, Failure> {
        let builder = BRepBuilder::default();
        let mut new_edge: Edge = topods::edge(&edge.empty_copied()?)?;
        let mut iterator = ShapeIterator::new(edge, true, true);
        while iterator.more() {
            builder.add(&mut new_edge, iterator.value()?)?;
            iterator.next();
        }
        self.my_old_new_shapes
            .bind(edge.as_ref().clone(), new_edge.as_ref().clone());
        if !modif_wires.contains(wire) {
            modif_wires.add(wire.as_ref().clone());
        }
        Ok(new_edge)
    }
}
