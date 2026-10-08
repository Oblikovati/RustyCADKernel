// Rust translation of BRepAlgo_Loop::CutEdge of ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_Loop.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cut of an edge at vertices: the vertices sorted along it, completed with its
//! ends, then a copy of the edge between each FORWARD vertex and the REVERSED one after
//! it, the tiny pieces dropped.
use super::BRepAlgoLoop;
use super::statics::{bubble, update_closed_edge};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// The ends of the edge `we` (FORWARD) and its range, for CutEdge.
struct EdgeEnds {
    vf: Vertex,
    vl: Vertex,
    f: f64,
    l: f64,
}

/// The vertices `sv` completed with the ends of `we`: for a closed edge, the closing
/// vertex (or the intersection on it) at both ends; else each end not already there.
fn complete_with_ends(we: &Edge, sv: &mut Vec<Shape>, ends: &mut EdgeEnds) -> Result<(), Failure> {
    if !ends.vf.as_ref().is_null() && ends.vf.as_ref().is_same(ends.vl.as_ref()) {
        let vcei = update_closed_edge(we, sv)?;
        if !vcei.as_ref().is_null() {
            ends.vf = topods::vertex(&vcei.as_ref().oriented(Orientation::Forward))?;
            ends.vl = topods::vertex(&vcei.as_ref().oriented(Orientation::Reversed))?;
        }
        sv.insert(0, ends.vf.as_ref().clone());
        sv.push(ends.vl.as_ref().clone());
        return Ok(());
    }
    if !ends.vf.as_ref().is_null() && sv.first().is_none_or(|v| !ends.vf.as_ref().is_same(v)) {
        sv.insert(0, ends.vf.as_ref().clone());
    }
    if !ends.vl.as_ref().is_null() && sv.last().is_none_or(|v| !ends.vl.as_ref().is_same(v)) {
        sv.push(ends.vl.as_ref().clone());
    }
    Ok(())
}

/// The parameter of a cut vertex: the range bound at the end vertex, else its
/// parameter taken INTERNAL.
fn cut_parameter(v: &Vertex, end: &Vertex, bound: f64, we: &Edge) -> Result<f64, Failure> {
    if v.as_ref().is_same(end.as_ref()) {
        return Ok(bound);
    }
    BRepTool::parameter(
        &topods::vertex(&v.as_ref().oriented(Orientation::Internal))?,
        we,
    )
}

/// The copy of `we` between `v1` and `v2` with the orientation of `e`.
fn piece(e: &Edge, we: &Edge, v1: &Vertex, v2: &Vertex, ends: &EdgeEnds) -> Result<Shape, Failure> {
    let b = BRepBuilder::default();
    let mut new_edge = we.as_ref().empty_copied()?;
    b.add(&mut new_edge, &v1.as_ref().oriented(Orientation::Forward))?;
    b.add(&mut new_edge, &v2.as_ref().oriented(Orientation::Reversed))?;
    let u1 = cut_parameter(v1, &ends.vf, ends.f, we)?;
    let u2 = cut_parameter(v2, &ends.vl, ends.l, we)?;
    b.range(&topods::edge(&new_edge)?, u1, u2, false)?;
    Ok(new_edge.oriented(e.as_ref().orientation()))
}

/// The pieces between each FORWARD vertex and a REVERSED one right after it.
fn pieces(
    e: &Edge,
    we: &Edge,
    mut sv: Vec<Shape>,
    ends: &EdgeEnds,
    ne: &mut List<Shape>,
) -> Result<(), Failure> {
    sv.reverse();
    loop {
        while sv
            .last()
            .is_some_and(|v| v.orientation() != Orientation::Forward)
        {
            sv.pop();
        }
        let Some(v1) = sv.pop() else { break };
        let Some(next) = sv.last() else { break };
        if next.orientation() == Orientation::Reversed {
            let v2 = sv.pop().unwrap_or_default();
            ne.append(piece(
                e,
                we,
                &topods::vertex(&v1)?,
                &topods::vertex(&v2)?,
                ends,
            )?);
        }
    }
    Ok(())
}

impl BRepAlgoLoop {
    /// Whether a cut piece is kept: a range longer than Precision::Confusion() and
    /// UV ends more than 0.001 apart unless the edge is closed.
    fn keeps_piece(&self, ee: &Shape) -> Result<bool, Failure> {
        let edge = topods::edge(ee)?;
        let (fpar, lpar) = BRepTool::range(&edge)?;
        if lpar - fpar <= precision::confusion() {
            return Ok(false);
        }
        let tol = 0.001; // 5.e-05; //5.e-07;
        let (pf, pl) = BRepTool::uv_points_on_face(&edge, &self.my_face)?;
        Ok(!(pf.distance(pl) <= tol && !BRepTool::is_closed(ee)?))
    }

    /// `CutEdge(E, VonE, NE)`: the pieces of `e` between the vertices `von_e` (sorted on
    /// it, completed with its ends) appended to `ne`; `e` itself without vertex or when
    /// they are its ends; `l.cut_edge(&e, &lv, &mut ne)?`.
    pub fn cut_edge(
        &self,
        e: &Edge,
        von_e: &List<Shape>,
        ne: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let we = topods::edge(&e.as_ref().oriented(Orientation::Forward))?;
        let mut sv: Vec<Shape> = von_e.iter().cloned().collect();
        bubble(&we, &mut sv)?;
        if sv.is_empty() {
            ne.append(e.as_ref().clone());
            return Ok(());
        }
        let (f, l) = BRepTool::range(&we)?;
        let (vf, vl) = topexp::vertices(&we, false);
        if sv.len() == 2 && sv[0].is_equal(vf.as_ref()) && sv[1].is_equal(vl.as_ref()) {
            ne.append(e.as_ref().clone());
            return Ok(());
        }
        let mut ends = EdgeEnds { vf, vl, f, l };
        complete_with_ends(&we, &mut sv, &mut ends)?;
        pieces(e, &we, sv, &ends, ne)?;
        ne.try_retain(|ee| self.keeps_piece(ee))
    }
}
