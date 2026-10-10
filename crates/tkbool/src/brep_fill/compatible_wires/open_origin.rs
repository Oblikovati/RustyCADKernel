// Rust translation of BRepFill_CompatibleWires::SearchOrigin
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SearchOrigin`: open sections oriented like the previous one, by the distances of
//! their ends (straight sections) or the angle of their end-to-end chords.
use super::BRepFillCompatibleWires;
use super::helpers::plane_of_wire;
use crate::brep_fill::ThruSectionErrorStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The ends of the previous section (in its new orientation), whether it is a straight
/// line (no plane) and its first edge.
struct PreviousSection {
    pdeb: Point3,
    pfin: Point3,
    is_line: bool,
    first_edge: Edge,
}

/// The point at Precision::Confusion() after the start of the curve of an edge.
fn near_start(edge: &Edge) -> Result<Point3, Failure> {
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    curve.d0(curve.first_parameter()? + precision::confusion())
}

/// Whether the section keeps its direction: by the sums of the distances of the ends
/// for straight sections, else by the angle of the chords (taken from points near the
/// start of the first edges when a chord is null).
fn keeps_direction(
    prev: &PreviousSection,
    ends: (Point3, Point3),
    is_line: bool,
    first_edge: &Edge,
) -> Result<bool, Failure> {
    let (p1, mut p2) = ends;
    if prev.is_line || is_line {
        // The particular case of straight segments.
        let dist1: f64 = prev.pdeb.distance(&p1) + prev.pfin.distance(&p2);
        let dist2: f64 = prev.pdeb.distance(&p2) + prev.pfin.distance(&p1);
        return Ok(dist2 >= dist1);
    }
    let (p1o, mut p2o) = (prev.pdeb, prev.pfin);
    if p1.is_equal(&p2, precision::confusion()) || p1o.is_equal(&p2o, precision::confusion()) {
        p2o = near_start(&prev.first_edge)?;
        p2 = near_start(first_edge)?;
    }
    let a_straight: f64 = Vector3::between(p1o, p2o).angle(Vector3::between(p1, p2))?;
    Ok(a_straight < PI / 2.)
}

impl BRepFillCompatibleWires {
    /// `SearchOrigin()`: the open sections (all must be, by their Closed flag;
    /// NotSameTopology otherwise) oriented one after the other.
    pub(super) fn search_origin(&mut self) -> Result<(), Failure> {
        let nb_sects: usize = self.my_work.len();
        let ideb: usize = if self.my_degen1 { 2 } else { 1 };
        let ifin: usize = if self.my_degen2 {
            nb_sects - 1
        } else {
            nb_sects
        };
        let v_closed: bool = !self.my_degen1
            && !self.my_degen2
            && self.my_work[ideb - 1].is_same(&self.my_work[ifin - 1]);
        let mut all_open: bool = true;
        for i in ideb..=ifin {
            all_open = all_open && !self.my_work[i - 1].closed()?;
        }
        if !all_open {
            self.my_status = ThruSectionErrorStatus::NotSameTopology;
            return Ok(());
        }
        let wire1: Wire = topods::wire(&self.my_work[ideb - 1].oriented(Orientation::Forward))?;
        let (vdeb, vfin) = topexp::wire_vertices(&wire1);
        let mut prev = PreviousSection {
            pdeb: BRepTool::pnt(&vdeb)?,
            pfin: BRepTool::pnt(&vfin)?,
            is_line: plane_of_wire(&wire1)?.is_none(),
            first_edge: WireExplorer::with_wire(&wire1)?.current().clone(),
        };
        self.my_work[ideb - 1] = wire1.into_shape();
        for i in ideb + 1..=ifin {
            self.orient_open_section(&mut prev, i)?;
        }
        // Blocking sections?
        if v_closed {
            self.my_work[nb_sects - 1] = self.my_work[0].clone();
        }
        Ok(())
    }

    /// Section `i` oriented like the previous one and rebuilt.
    fn orient_open_section(&mut self, prev: &mut PreviousSection, i: usize) -> Result<(), Failure> {
        let wire: Wire = topods::wire(&self.my_work[i - 1].oriented(Orientation::Forward))?;
        let mut edges: Vec<Edge> = Vec::new();
        let mut explorer = WireExplorer::with_wire(&wire)?;
        let first_edge: Edge = explorer.current().clone();
        while explorer.more() {
            edges.push(explorer.current().clone());
            explorer.next()?;
        }
        let (vdeb, vfin) = topexp::wire_vertices(&wire);
        let is_line: bool = plane_of_wire(&wire)?.is_none();
        let ends: (Point3, Point3) = (BRepTool::pnt(&vdeb)?, BRepTool::pnt(&vfin)?);
        let parcours: bool = keeps_direction(prev, ends, is_line, &first_edge)?;
        let builder = BRepBuilder::default();
        let mut new_wire = Wire::default();
        builder.make_wire(&mut new_wire);
        if parcours {
            for edge in &edges {
                builder.add(&mut new_wire, edge)?;
            }
        } else {
            for edge in edges.iter().rev() {
                builder.add(&mut new_wire, &edge.reversed())?;
            }
        }
        // `newwire.Oriented(TopAbs_FORWARD)` natively, whose result is discarded.
        self.my_work[i - 1] = new_wire.into_shape();
        (prev.pdeb, prev.pfin) = if parcours { ends } else { (ends.1, ends.0) };
        prev.is_line = is_line;
        prev.first_edge = first_edge;
        Ok(())
    }
}
