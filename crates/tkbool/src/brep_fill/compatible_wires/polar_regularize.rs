// Rust translation of BRepFill_CompatibleWires::SameNumberByPolarMethod (second part)
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The end of the polar method: each section rebuilt from the edge that matches the
//! first edge of the first section and oriented like it, the looping sections closed,
//! the edge counts checked and the generated map filled.
use super::BRepFillCompatibleWires;
use super::helpers::{VertexLists, add_new_edge};
use super::intersect::{build_connected_edges, transform};
use super::polar::Polar;
use crate::brep_fill::ThruSectionErrorStatus;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;

/// The point at 9/10 of an edge from its first vertex (in its orientation) and,
/// with `both`, at 1/10.
fn tenth_points(edge: &Edge) -> Result<(Point3, Point3), Failure> {
    let (vf, vl) = topexp::vertices(edge, true);
    let u1: f64 = BRepTool::parameter(&vf, edge)?;
    let u2: f64 = BRepTool::parameter(&vl, edge)?;
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    Ok((
        curve.value(0.1 * (u1 + 9. * u2))?,
        curve.value(0.1 * (9. * u1 + u2))?,
    ))
}

/// The direction from the center of the first section to a point.
fn from_center(polar: &Polar, point: Point3) -> Result<Vector3, Failure> {
    Vector3::between(polar.pos[polar.ideb - 1], point).normalized()
}

/// The two tenth points of a candidate edge of section `i`, carried back to the
/// plane of the first section.
fn carried_points(polar: &Polar, edge: &Edge, i: usize) -> Result<(Point3, Point3), Failure> {
    let (mut pp1, mut pp2) = tenth_points(edge)?;
    let mut rang: usize = i;
    while rang > polar.ideb {
        pp1 = transform(
            polar.with_rotation,
            pp1,
            polar.plane(rang),
            polar.plane(rang - 1),
        )?;
        pp2 = transform(
            polar.with_rotation,
            pp2,
            polar.plane(rang),
            polar.plane(rang - 1),
        )?;
        rang -= 1;
    }
    Ok((pp1, pp2))
}

impl BRepFillCompatibleWires {
    /// Every section after the first rebuilt by `regularize_section`; false when a
    /// status stops the method.
    pub(super) fn regularize(
        &mut self,
        polar: &mut Polar,
        map_vlv: &VertexLists,
        v_closed: bool,
    ) -> Result<bool, Failure> {
        // The first section as it is now (the native local `wire`).
        let wire: Wire = topods::wire(&self.my_work[polar.ideb - 1])?;
        let ibout: usize = if v_closed { polar.ifin - 1 } else { polar.ifin };
        for i in polar.ideb + 1..=ibout {
            match self.regularize_section(polar, map_vlv, &wire, i)? {
                Some(rebuilt) => self.my_work[i - 1] = rebuilt.into_shape(),
                None => return Ok(false),
            }
        }
        Ok(true)
    }

    /// The vertices of section `i` that the ends of the first edge of the first
    /// section report to; `None` (Failed status) when a chain is too short.
    fn reported_ends(
        &mut self,
        polar: &Polar,
        map_vlv: &VertexLists,
        first: &Edge,
        i: usize,
    ) -> Result<Option<(Vertex, Vertex)>, Failure> {
        let (vf, vl) = topexp::vertices(first, true);
        let chain_f: Vec<Shape> = map_vlv.find(&vf)?.iter().cloned().collect();
        let chain_l: Vec<Shape> = map_vlv.find(&vl)?.iter().cloned().collect();
        // The chains are advanced `i - ideb` times, while both have an element.
        let steps: usize = i - polar.ideb;
        let (Some(v1), Some(v2)) = (chain_f.get(steps), chain_l.get(steps)) else {
            // Correspondence chain shorter than the section index; fail gracefully.
            self.my_status = ThruSectionErrorStatus::Failed;
            return Ok(None);
        };
        Ok(Some((topods::vertex(v1)?, topods::vertex(v2)?)))
    }

    /// Section `i` rebuilt: the edge between the reported ends that best follows the
    /// first edge of `wire`, then the edges connected to it, each oriented like the
    /// edge of `wire` at the same rank (from the first one again, as natively).
    fn regularize_section(
        &mut self,
        polar: &Polar,
        map_vlv: &VertexLists,
        wire: &Wire,
        i: usize,
    ) -> Result<Option<Wire>, Failure> {
        let mut explorer = WireExplorer::with_wire(wire)?;
        let first: Edge = explorer.current().clone();
        let (pps, _) = tenth_points(&first)?;
        let Some((v1, v2)) = self.reported_ends(polar, map_vlv, &first, i)? else {
            return Ok(None);
        };
        let section: Wire = topods::wire(&self.my_work[i - 1])?;
        let Some(esol) = best_matching_edge(polar, &section, (&v1, &v2), pps, i)? else {
            self.my_status = ThruSectionErrorStatus::ProfilesInconsistent;
            return Ok(None);
        };
        let mut maker = BRepLibMakeWire::new();
        maker.add_edge(&esol)?;
        let connected: Vec<Shape> = build_connected_edges(&section, &esol, &v2)?;
        for candidate in &connected {
            if !explorer.more() {
                break;
            }
            let (reference, _) = tenth_points(explorer.current())?;
            let mut edge: Edge = topods::edge(candidate)?;
            let (pp1, pp2) = carried_points(polar, &edge, i)?;
            let ns: Vector3 = from_center(polar, reference)?;
            let scal1: f64 = from_center(polar, pp1)?.dot(ns);
            let scal2: f64 = from_center(polar, pp2)?.dot(ns);
            if scal2 > scal1 {
                edge.reverse();
            }
            maker.add_edge(&edge)?;
            explorer.next()?;
        }
        Ok(Some(maker.wire()?))
    }

    /// The looping sections closed (the last one is the first, its edges mapped), the
    /// equal edge counts checked and each original edge mapped to its newest pieces.
    pub(super) fn finish_polar(
        &mut self,
        polar: &mut Polar,
        v_closed: bool,
    ) -> Result<(), Failure> {
        let nb: usize = self.my_work.len();
        if v_closed {
            let mut iter1 = ShapeIterator::new(&self.my_work[nb - 1], true, true);
            let mut iter2 = ShapeIterator::new(&self.my_work[0], true, true);
            while iter1.more() {
                let an_edge: Shape = iter1.value()?.clone();
                let a_new_edge: Shape = iter2.value()?.clone();
                if !an_edge.is_same(&a_new_edge) {
                    polar.edge_new_edges.bind(an_edge, vec![a_new_edge]);
                }
                iter1.next();
                iter2.next();
            }
            self.my_work[nb - 1] = self.my_work[0].clone();
        }
        if !self.same_edge_counts(polar.ideb, polar.ifin)? {
            self.my_status = ThruSectionErrorStatus::Failed;
            return Ok(());
        }
        let keys: Vec<Shape> = self.my_map.items().map(|(key, _)| key.clone()).collect();
        for key in keys {
            let mut newest: List<Shape> = List::default();
            add_new_edge(&key, &polar.edge_new_edges, &mut newest);
            *self.my_map.change_find(&key)? = newest;
        }
        Ok(())
    }
}

/// The edge of the section between the two reported vertices whose tenth point,
/// carried back to the first plane, is nearest in direction to `pps` (reversed when
/// its other tenth point is nearer); `None` when no edge joins them.
fn best_matching_edge(
    polar: &Polar,
    section: &Wire,
    ends: (&Vertex, &Vertex),
    pps: Point3,
    i: usize,
) -> Result<Option<Edge>, Failure> {
    let (v1, v2) = ends;
    let mut esol: Option<Edge> = None;
    let mut scalmax: f64 = 0.;
    let mut iterator = ShapeIterator::new(section, true, true);
    while iterator.more() {
        let edge: Edge = topods::edge(iterator.value()?)?;
        iterator.next();
        let (vvf, vvl) = topexp::vertices(&edge, true);
        if !((v1.is_same(&vvf) && v2.is_same(&vvl)) || (v2.is_same(&vvf) && v1.is_same(&vvl))) {
            continue;
        }
        let (pp1, pp2) = carried_points(polar, &edge, i)?;
        let ns: Vector3 = from_center(polar, pps)?;
        let scal1: f64 = from_center(polar, pp1)?.dot(ns);
        if scal1 > scalmax {
            scalmax = scal1;
            esol = Some(edge.clone());
        }
        let scal2: f64 = from_center(polar, pp2)?.dot(ns);
        if scal2 > scalmax {
            scalmax = scal2;
            esol = Some(topods::edge(&edge.reversed())?);
        }
    }
    Ok(esol)
}
