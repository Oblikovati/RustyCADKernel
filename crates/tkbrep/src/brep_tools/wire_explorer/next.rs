// Rust translation of BRepTools_WireExplorer::Next
// (ModelingData/TKBRep/BRepTools/BRepTools_WireExplorer.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_WireExplorer::Next`: the edge that follows the current one.
use super::WireExplorer;
use super::select::{StepSearch, next_param_on_pcurve, select_degenerated, select_double};
use crate::brep::BRepTool;
use crate::topexp;
use crate::topods::{self, Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Point2, Vector2};
use std::f64::consts::PI;

/// The reference of the angular choice: the 2D point of the current vertex on the
/// current edge and the direction leaving it backwards along that edge.
struct Reference {
    point: Point2,
    direction: Vector2,
}

/// The running state of the angular choice (the native locals of `Next`).
struct AngularChoice {
    k: usize,
    k_min: usize,
    is_degenerated: bool,
    d_min: f64,
    min_angle: f64,
    cur_angle: f64,
}

impl WireExplorer {
    /// `Next()`: the edge starting at the last vertex of the current edge; the explorer
    /// ends (null current edge) at an open end or when no edge follows. With several
    /// candidates, without a face: a degenerated edge, then an edge used twice, then the
    /// first other one; with a face: the one whose p-curve leaves the vertex closest in
    /// UV and at the smallest angle, degenerated edges first; `explorer.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible
    pub fn next(&mut self) -> Result<(), Failure> {
        self.vertex = topexp::last_vertex(&self.edge, true);
        if self.vertex.is_null() || !self.map.is_bound(&self.vertex) {
            self.edge = Edge::default();
            return Ok(());
        }
        let mut list = std::mem::take(self.map.change_find(&self.vertex)?);
        let result = self.choose_next(&mut list);
        *self.map.change_find(&self.vertex)? = list;
        result
    }

    /// The choice among the edges that start at the current vertex (`l`).
    fn choose_next(&mut self, list: &mut Vec<Shape>) -> Result<(), Failure> {
        match list.len() {
            0 => self.edge = Edge::default(),
            1 => self.take_single(list)?,
            _ if self.face.is_null() => self.take_without_face(list)?,
            _ => self.take_on_face(list)?,
        }
        Ok(())
    }

    /// One candidate: it must start at the current vertex and, on a face, a closed
    /// candidate must leave from the p-curve end nearest to the current edge's end.
    fn take_single(&mut self, list: &mut Vec<Shape>) -> Result<(), Failure> {
        let next = topods::edge(&list[0])?;
        let (first, last) = topexp::vertices(&next, true);
        if !first.is_same(&self.vertex) {
            self.edge = Edge::default();
            return Ok(());
        }
        if !self.face.is_null() && first.is_same(&last) && !self.closed_edge_follows(&next)? {
            self.edge = Edge::default();
            return Ok(());
        }
        self.edge = next;
        list.clear();
        Ok(())
    }

    /// Whether the closed `next` edge leaves from the end of its p-curve nearest to the
    /// end of the current edge's p-curve (OCC325); false when a p-curve is missing.
    fn closed_edge_follows(&self, next: &Edge) -> Result<bool, Failure> {
        let previous = BRepTool::curve_on_face(&self.edge, &self.face)?;
        let following = BRepTool::curve_on_face(next, &self.face)?;
        let (Some(prev_pc), Some(next_pc)) = (previous.pcurve.get(), following.pcurve.get()) else {
            return Ok(false);
        };
        let previous_end = if self.edge.orientation() == Orientation::Forward {
            previous.last
        } else {
            previous.first
        };
        let (next_first, next_last) = if next.orientation() == Orientation::Forward {
            (following.first, following.last)
        } else {
            (following.last, following.first)
        };
        let end = prev_pc.value(previous_end)?;
        let start = next_pc.value(next_first)?;
        let other = next_pc.value(next_last)?;
        Ok(end.square_distance(start) <= end.square_distance(other))
    }

    /// Several candidates without a face.
    fn take_without_face(&mut self, list: &mut Vec<Shape>) -> Result<(), Failure> {
        let mut edge = self.edge.clone();
        if select_degenerated(list, &mut edge)? {
            self.edge = edge;
            return Ok(());
        }
        let mut edge = self.edge.clone();
        if select_double(&self.doubles, list, &mut edge)? {
            self.edge = edge;
            return Ok(());
        }
        match list
            .iter()
            .position(|candidate| !candidate.is_same(&self.edge))
        {
            Some(position) => self.edge = topods::edge(&list.remove(position))?,
            None => self.edge = Edge::default(),
        }
        Ok(())
    }

    /// Several candidates on a face: the angular choice of the native loop.
    fn take_on_face(&mut self, list: &mut Vec<Shape>) -> Result<(), Failure> {
        let Some(reference) = self.reference()? else {
            self.edge = Edge::default();
            return Ok(());
        };
        let mut choice = AngularChoice {
            k: 1,
            k_min: 0,
            is_degenerated: true,
            d_min: f64::MAX,
            min_angle: 3.0 * PI,
            cur_angle: 3.0 * PI,
        };
        for _pass in 0..2 {
            for candidate in list.iter() {
                self.examine(candidate, &reference, &mut choice)?;
                choice.k += 1;
            }
            if choice.k_min != 0 {
                break;
            }
            choice.is_degenerated = false;
            choice.k = 1;
            choice.d_min = f64::MAX;
        }
        if choice.k_min == 0 {
            // Probably a wire unclosed in the 2D space.
            self.edge = Edge::default();
            return Ok(());
        }
        self.edge = topods::edge(&list.remove(choice.k_min - 1))?;
        Ok(())
    }

    /// The 2D point of the current vertex on the current edge and the direction to the
    /// next distinct point of its p-curve; none when the p-curve is missing or the
    /// direction is null.
    fn reference(&self) -> Result<Option<Reference>, Failure> {
        let found = BRepTool::curve_on_face(&self.edge, &self.face)?;
        let Some(pcurve) = found.pcurve.get() else {
            return Ok(None);
        };
        let reversed = self.edge.orientation() == Orientation::Reversed;
        let point = pcurve.d0(if reversed { found.first } else { found.last })?;
        let search = self.search(reversed);
        let parameter = next_param_on_pcurve(pcurve, point, (found.first, found.last), &search)?;
        let direction = Vector2::between(point, pcurve.d0(parameter)?);
        if direction.square_magnitude() < gp::resolution() {
            return Ok(None);
        }
        Ok(Some(Reference { point, direction }))
    }

    fn search(&self, reverse: bool) -> StepSearch {
        StepSearch {
            tol_u: self.tol_u,
            tol_v: self.tol_v,
            reverse,
        }
    }

    /// One candidate of the angular choice: an edge other than the current one, with
    /// both vertices and a p-curve, of the degeneracy of the pass.
    fn examine(
        &self,
        candidate: &Shape,
        reference: &Reference,
        choice: &mut AngularChoice,
    ) -> Result<(), Failure> {
        let edge = topods::edge(candidate)?;
        if edge.is_same(&self.edge) {
            return Ok(());
        }
        let (first, last) = topexp::vertices(&edge, true);
        if first.is_null() || last.is_null() {
            return Ok(());
        }
        let found = BRepTool::curve_on_face(&edge, &self.face)?;
        let Some(pcurve) = found.pcurve.get() else {
            return Ok(());
        };
        if first.is_same(&last) != choice.is_degenerated {
            return Ok(());
        }
        self.measure(&edge, pcurve, (found.first, found.last), reference, choice)
    }

    /// The angle and distance test of a candidate; `cur_angle` keeps its previous value
    /// for a candidate whose range is shorter than `Precision::PConfusion()`, as natively.
    fn measure(
        &self,
        edge: &Edge,
        pcurve: &dyn Geom2dCurve,
        (first, last): (f64, f64),
        reference: &Reference,
        choice: &mut AngularChoice,
    ) -> Result<(), Failure> {
        let reversed = edge.orientation() == Orientation::Reversed;
        let begin = pcurve.d0(if reversed { last } else { first })?;
        if (last - first).abs() > precision::p_confusion() {
            let search = self.search(!reversed);
            let parameter = next_param_on_pcurve(pcurve, begin, (first, last), &search)?;
            let Some(end) = leaving_point(pcurve, begin, parameter, reversed)? else {
                return Ok(());
            };
            choice.cur_angle = Vector2::between(begin, end)
                .angle(reference.direction)?
                .abs();
        }
        if choice.cur_angle > choice.min_angle {
            return Ok(());
        }
        let mut distance = reference.point.square_distance(begin);
        if distance <= precision::p_confusion() {
            distance = 0.0;
        }
        let near = (begin.x() - reference.point.x()).abs() < self.tol_u
            && (begin.y() - reference.point.y()).abs() < self.tol_v;
        if near && distance <= choice.d_min {
            choice.min_angle = choice.cur_angle;
            choice.k_min = choice.k;
            choice.d_min = distance;
        }
        Ok(())
    }
}

/// The point the candidate leaves towards: its p-curve at `parameter`, or, for a very
/// short curve, the start moved by the first derivative there; none when that still
/// coincides with the start.
fn leaving_point(
    pcurve: &dyn Geom2dCurve,
    begin: Point2,
    parameter: f64,
    reversed: bool,
) -> Result<Option<Point2>, Failure> {
    let end = pcurve.d0(parameter)?;
    if begin.square_distance(end) > gp::resolution() {
        return Ok(Some(end));
    }
    let (_, derivative) = pcurve.d1(parameter)?;
    let moved = if reversed {
        begin.xy() - derivative.xy()
    } else {
        begin.xy() + derivative.xy()
    };
    let moved = Point2::new(moved.x(), moved.y());
    if begin.square_distance(moved) <= gp::resolution() {
        return Ok(None);
    }
    Ok(Some(moved))
}
