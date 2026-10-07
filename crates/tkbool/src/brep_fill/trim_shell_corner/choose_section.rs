// Rust translation of BRepFill_TrimShellCorner::ChooseSection
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The choice of the section wire among the section edges of two faces at a corner.
use super::BRepFillTrimShellCorner;
use super::gap::vertex_edge_map;
use super::section_wire::{
    compute_average_plane_and_max_deviation, find_common_vertex_edges,
    find_edge_close_to_bisector_plane, find_middle_edges, update_section_edge,
};
use crate::brep_fill::transition_style::TransitionStyle;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Axis3, Plane3};
use rustycad_tktopalgo::brep_extrema::BRepExtremaExtCC;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeVertex, BRepLibMakeWire};
use std::f64::consts::PI;

/// The section wire chosen and its plane (the native singularity flag of the plane is
/// never read by the caller).
pub(super) struct SectionChoice {
    pub(super) wire: Shape,
    pub(super) plane: Plane3,
}

/// A copy of the children of a compound in a new compound.
fn copied_compound(comp: &Shape) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut old_comp = Compound::default();
    builder.make_compound(&mut old_comp);
    let mut iter = ShapeIterator::new(comp, true, true);
    while iter.more() {
        builder.add(&mut old_comp, iter.value()?)?;
        iter.next();
    }
    Ok(old_comp)
}

/// The two section edges joined at the vertex in the middle of their closest points,
/// each one cut there.
fn join_at_extrema(
    first_edge: &Edge,
    last_edge: &Edge,
    (the_first_vertex, the_last_vertex): (&Vertex, &Vertex),
) -> Result<bool, Failure> {
    let extrema = BRepExtremaExtCC::with_edges(first_edge, last_edge)?;
    if !extrema.is_done() || extrema.nb_ext()? <= 0 {
        return Ok(false);
    }
    let mut imin: i32 = 1;
    for i in 2..=extrema.nb_ext()? {
        if extrema.square_distance(i)? < extrema.square_distance(imin)? {
            imin = i;
        }
    }
    let a_min_dist: f64 = extrema.square_distance(imin)?.sqrt();
    let (p_first, p_last) = (extrema.point_on_e1(imin)?, extrema.point_on_e2(imin)?);
    let mut mid_pnt = rustycad_tkmath::gp::Point3::default();
    mid_pnt.set_xyz((p_first.xyz() + p_last.xyz()) / 2.);
    let mut a_common_vertex: Vertex = BRepLibMakeVertex::new(mid_pnt)?.vertex()?;
    BRepBuilder::default().update_vertex_tolerance(&a_common_vertex, 1.001 * a_min_dist / 2.)?;
    update_section_edge(
        first_edge,
        the_first_vertex,
        &mut a_common_vertex,
        extrema.parameter_on_e1(imin)?,
    )?;
    update_section_edge(
        last_edge,
        the_last_vertex,
        &mut a_common_vertex,
        extrema.parameter_on_e2(imin)?,
    )?;
    Ok(true)
}

impl BRepFillTrimShellCorner {
    /// `ChooseSection(Comp, theFirstVertex, theLastVertex, resWire, resPlane,
    /// IsSingular)`: for a right corner with known ends, the wire of the section edges
    /// closest to the bisector plane between them; otherwise the wires of the section
    /// edges, the one whose average plane is closest to the bisector plane; `None` when
    /// no wire can be built.
    pub(super) fn choose_section(
        &self,
        comp: &Shape,
        the_first_vertex: &Vertex,
        the_last_vertex: &Vertex,
    ) -> Result<Option<SectionChoice>, Failure> {
        if self.my_transition == TransitionStyle::Right
            && !the_first_vertex.is_null()
            && !the_last_vertex.is_null()
        {
            return self.right_corner_section(comp, the_first_vertex, the_last_vertex);
        }
        let mut old_comp: Compound = copied_compound(comp)?;
        let (wseq, an_error) = build_section_wires(&mut old_comp)?;
        if an_error || Explorer::new(&old_comp, ShapeEnum::Edge, ShapeEnum::Shape)?.more() {
            return Ok(None);
        }
        if wseq.len() == 1 {
            let average = compute_average_plane_and_max_deviation(&wseq[0])?;
            return Ok(Some(SectionChoice {
                wire: wseq[0].clone(),
                plane: average.plane.unwrap_or_default(),
            }));
        }
        let mut choice = SectionChoice {
            wire: Shape::default(),
            plane: Plane3::default(),
        };
        let mut min_angle: f64 = f64::MAX;
        for a_wire in &wseq {
            let Some(a_pln) = compute_average_plane_and_max_deviation(a_wire)?.plane else {
                continue;
            };
            let mut angle: f64 = a_pln.axis().angle(self.my_axe_of_bis_plane.axis());
            if angle > PI / 2. {
                angle = PI - angle;
            }
            if angle < min_angle {
                min_angle = angle;
                choice.wire = a_wire.clone();
                choice.plane = a_pln;
            }
        }
        Ok(Some(choice))
    }

    /// The right-corner case: the section wire goes from its known first vertex to its
    /// known last vertex.
    fn right_corner_section(
        &self,
        comp: &Shape,
        the_first_vertex: &Vertex,
        the_last_vertex: &Vertex,
    ) -> Result<Option<SectionChoice>, Failure> {
        let builder = BRepBuilder::default();
        let axis: Axis1 = self.my_axe_of_bis_plane.axis();
        let mut old_comp: Compound = copied_compound(comp)?;
        let first_edge: Edge =
            find_edge_close_to_bisector_plane(the_first_vertex, &mut old_comp, &axis)?;
        if first_edge.is_null() {
            return Ok(None);
        }
        if !ShapeIterator::new(&old_comp, true, true).more() {
            let iter = ShapeIterator::new(comp, true, true);
            builder.add(&mut old_comp, iter.value()?)?;
        }
        let last_edge: Edge =
            find_edge_close_to_bisector_plane(the_last_vertex, &mut old_comp, &axis)?;
        if last_edge.is_null() {
            return Ok(None);
        }
        let mut new_wire = Wire::default();
        builder.make_wire(&mut new_wire);
        builder.add(&mut new_wire, &first_edge)?;
        if !first_edge.is_same(&last_edge) {
            self.join_section_ends(
                &mut new_wire,
                (&first_edge, &last_edge),
                (the_first_vertex, the_last_vertex),
                &mut old_comp,
            )?;
        }
        let plane = Plane3::from_position(Axis3::from_axis2(self.my_axe_of_bis_plane));
        Ok(Some(SectionChoice {
            wire: new_wire.into_shape(),
            plane,
        }))
    }

    /// The last edge added to the wire after the first one: directly when they share a
    /// vertex, after the middle edges closest to the bisector plane, or after both are
    /// cut at their closest points (nothing more is added when that fails).
    fn join_section_ends(
        &self,
        new_wire: &mut Wire,
        (first_edge, last_edge): (&Edge, &Edge),
        (the_first_vertex, the_last_vertex): (&Vertex, &Vertex),
        old_comp: &mut Compound,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        if find_common_vertex_edges(first_edge, last_edge, the_first_vertex, the_last_vertex)
            .is_some()
        {
            return builder.add(new_wire, last_edge);
        }
        let (v1, v2) = topexp::vertices(first_edge, false);
        let vertex1: Vertex = if the_first_vertex.is_same(&v1) {
            v2
        } else {
            v1
        };
        let (v1, v2) = topexp::vertices(last_edge, false);
        let vertex2: Vertex = if the_last_vertex.is_same(&v1) { v2 } else { v1 };
        let mut middle_edges: List<Shape> = List::default();
        if find_middle_edges(
            &vertex1,
            &vertex2,
            &self.my_axe_of_bis_plane.axis(),
            old_comp,
            &mut middle_edges,
        )? {
            for e in middle_edges.iter() {
                builder.add(new_wire, e)?;
            }
            return builder.add(new_wire, last_edge);
        }
        if join_at_extrema(first_edge, last_edge, (the_first_vertex, the_last_vertex))? {
            builder.add(new_wire, last_edge)?;
        }
        Ok(())
    }
}

/// The wires of the section edges taken out of the compound in turn (each extended at
/// its free ends: through the only edge there, else through the edge of least average
/// plane deviation) and whether a wire lost its ends.
fn build_section_wires(old_comp: &mut Compound) -> Result<(Vec<Shape>, bool), Failure> {
    let builder = BRepBuilder::default();
    let mut wseq: Vec<Shape> = Vec::new();
    loop {
        let explo = Explorer::new(old_comp, ShapeEnum::Edge, ShapeEnum::Shape)?;
        if !explo.more() {
            return Ok((wseq, false));
        }
        let first_edge: Edge = topods::edge(explo.current())?;
        let mut new_wire: Wire = BRepLibMakeWire::from_edge(&first_edge)?.wire()?;
        builder.remove(old_comp, &first_edge)?;
        if new_wire.closed()? {
            wseq.push(new_wire.into_shape());
            continue;
        }
        let an_error: bool = extend_section_wire(&mut new_wire, old_comp)?;
        wseq.push(new_wire.into_shape());
        if an_error {
            return Ok((wseq, true));
        }
    }
}

/// The wire extended with the edges of the compound at its ends until it closes or
/// nothing connects; true when the wire has no two ends.
fn extend_section_wire(new_wire: &mut Wire, old_comp: &mut Compound) -> Result<bool, Failure> {
    let builder = BRepBuilder::default();
    loop {
        let (e0, e1) = topexp::wire_vertices(new_wire);
        if e0.is_null() || e1.is_null() {
            return Ok(true);
        }
        let ve_map = vertex_edge_map(old_comp)?;
        let vedges: [List<Shape>; 2] = [
            ve_map.seek(&e0).cloned().unwrap_or_default(),
            ve_map.seek(&e1).cloned().unwrap_or_default(),
        ];
        if vedges[0].is_empty() && vedges[1].is_empty() {
            return Ok(false);
        }
        let mut modified: bool = false;
        for vedge in &vedges {
            if vedge.extent() == 1 {
                let an_edge: Edge = topods::edge(vedge.first().unwrap_or(&Shape::default()))?;
                *new_wire = BRepLibMakeWire::from_wire_edge(new_wire, &an_edge)?.wire()?;
                builder.remove(old_comp, &an_edge)?;
                modified = true;
            }
        }
        if !modified {
            let edges: &List<Shape> = if vedges[0].is_empty() {
                &vedges[1]
            } else {
                &vedges[0]
            };
            let best: Edge = least_deviation_edge(new_wire, edges)?;
            *new_wire = BRepLibMakeWire::from_wire_edge(new_wire, &best)?.wire()?;
            builder.remove(old_comp, &best)?;
        }
        if new_wire.closed()? {
            return Ok(false);
        }
    }
}

/// The edge whose addition to the wire gives the least average plane deviation (the
/// first of equal ones); Standard_OutOfRange when none is less than RealLast (the
/// native index 0).
fn least_deviation_edge(new_wire: &Wire, edges: &List<Shape>) -> Result<Edge, Failure> {
    let mut best: Option<Edge> = None;
    let mut min_deviation: f64 = f64::MAX;
    for e in edges.iter() {
        let edge: Edge = topods::edge(e)?;
        let a_wire: Wire = BRepLibMakeWire::from_wire_edge(new_wire, &edge)?.wire()?;
        let deviation: f64 = compute_average_plane_and_max_deviation(&a_wire)?.deviation;
        if deviation < min_deviation {
            min_deviation = deviation;
            best = Some(edge);
        }
    }
    best.ok_or_else(|| Failure::with_type("Standard_OutOfRange", &format!("BRepFill_TrimShellCorner ChooseSection: no edge of {} below RealLast deviation; expected an index in 1..={}", edges.extent(), edges.extent()), "NCollection_Sequence"))
}
