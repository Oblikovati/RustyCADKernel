// Rust translation of BRepFill_OffsetWire::FixHoles
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The open offset wires joined end to end (within 100 times the greatest vertex
//! tolerance of the spine) and closed when possible.
use super::BRepFillOffsetWire;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// The nearest end of another open wire to an end of the base wire.
#[derive(Clone, Copy)]
struct NearestEnd {
    dist: f64,
    index: usize,
    is_first: bool,
}

impl NearestEnd {
    /// No wire found yet (RealLast).
    fn none() -> Self {
        Self {
            dist: f64::MAX,
            index: 0,
            is_first: false,
        }
    }
}

/// The two vertices of an open wire; Standard_Failure for a wrong wire.
fn wire_ends(w: &Wire) -> Result<(Vertex, Vertex), Failure> {
    let (v1, v2) = topexp::wire_vertices(w);
    if v1.is_null() || v2.is_null() {
        return Err(Failure::new("BRepFill_OffsetWire::FixHoles(): Wrong wire."));
    }
    Ok((v1, v2))
}

/// The greatest vertex tolerance of the shape times 100.
fn max_tolerance(spine: &Shape) -> Result<f64, Failure> {
    let mut max_tol: f64 = 0.;
    let mut explo = Explorer::new(spine, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    while explo.more() {
        let tol: f64 = BRepTool::vertex_tolerance(&topods::vertex(explo.current())?)?;
        if tol > max_tol {
            max_tol = tol;
        }
        explo.next()?;
    }
    Ok(max_tol * 100.)
}

/// The wire without its repeated edges (each repetition removed once).
fn remove_duplicated_edges(a_wire: &mut Shape) -> Result<(), Failure> {
    let mut ee_map: DataMap<Shape, List<Shape>, ShapeMapHasher> = DataMap::default();
    let mut it = ShapeIterator::new(a_wire, true, true);
    while it.more() {
        let an_edge = it.value()?.clone();
        match ee_map.change_seek(&an_edge) {
            Some(le) => {
                le.append(an_edge);
            }
            None => {
                ee_map.bind(an_edge, List::default());
            }
        }
        it.next();
    }
    a_wire.set_free(true)?;
    let b = BRepBuilder::default();
    for (_, le) in ee_map.items() {
        for e in le.iter() {
            b.remove(a_wire, e)?;
        }
    }
    Ok(())
}

/// The nearest ends of the other open wires to the first and last vertices of the base.
fn nearest_ends(
    unclosed: &[Wire],
    (pf, pl): (Point3, Point3),
) -> Result<(NearestEnd, NearestEnd), Failure> {
    let (mut near_f, mut near_l) = (NearestEnd::none(), NearestEnd::none());
    for (i, w) in unclosed.iter().enumerate().skip(1) {
        let (v1, v2) = wire_ends(w)?;
        let (p1, p2) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
        for (near, base) in [(&mut near_f, pf), (&mut near_l, pl)] {
            for (p, is_first) in [(p1, true), (p2, false)] {
                let dist: f64 = base.distance(&p);
                if dist < near.dist {
                    *near = NearestEnd {
                        dist,
                        index: i,
                        is_first,
                    };
                }
            }
        }
    }
    Ok((near_f, near_l))
}

/// Which end of the base a joined wire is attached to.
#[derive(Clone, Copy, PartialEq)]
enum BaseEnd {
    First,
    Last,
}

/// The wire at an end of the base appended to it: its edges (reversed when needed),
/// its edge at the joined end sharing the base vertex when they are within their
/// tolerances, else linked by a new edge.
fn join_wire(
    base: &mut Wire,
    the_wire: &Wire,
    near: NearestEnd,
    (base_end, base_vertex): (BaseEnd, &mut Vertex),
) -> Result<(), Failure> {
    let bb = BRepBuilder::default();
    let (v1, v2) = topexp::wire_vertices(the_wire);
    let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(the_wire, ShapeEnum::Vertex, ShapeEnum::Edge, &mut ve_map)?;
    let joined: &Vertex = if near.is_first { &v1 } else { &v2 };
    let mut the_edge: Edge = topods::edge(
        ve_map
            .find_from_key(joined.as_ref())?
            .first()
            .ok_or_else(no_edge)?,
    )?;
    // Reversed for a wire joined by its first vertex at the base start, or by its last
    // vertex at the base end.
    let reverse: bool = (base_end == BaseEnd::First) == near.is_first;
    let mut it = ShapeIterator::new(the_wire, true, true);
    while it.more() {
        let mut an_edge: Edge = topods::edge(it.value()?)?;
        if reverse {
            an_edge.reverse();
        }
        if !an_edge.is_same(&the_edge) {
            bb.add(base, &an_edge)?;
        }
        it.next();
    }
    let the_vertex: Vertex = joined.clone();
    let common_tol: f64 =
        BRepTool::vertex_tolerance(base_vertex)?.max(BRepTool::vertex_tolerance(&the_vertex)?);
    if near.dist <= common_tol {
        the_edge.set_free(true)?;
        base_vertex.set_orientation(the_vertex.orientation());
        bb.remove(&mut the_edge, &the_vertex)?;
        bb.add(&mut the_edge, base_vertex)?;
        bb.update_vertex_tolerance(base_vertex, common_tol)?;
        if reverse {
            the_edge.reverse();
        }
        bb.add(base, &the_edge)?;
        return Ok(());
    }
    if reverse {
        the_edge.reverse();
    }
    bb.add(base, &the_edge)?;
    let new_edge: Edge = match base_end {
        BaseEnd::First => BRepLibMakeEdge::from_vertices(&the_vertex, base_vertex)?.edge()?,
        BaseEnd::Last => BRepLibMakeEdge::from_vertices(base_vertex, &the_vertex)?.edge()?,
    };
    bb.add(base, &new_edge)
}

/// The base closed when its ends are within their tolerances (sharing the first
/// vertex) or within `max_tol` (linked by a new edge).
fn try_to_close(base: &mut Wire, max_tol: f64) -> Result<(), Failure> {
    let bb = BRepBuilder::default();
    let (mut vf, vl) = topexp::wire_vertices(base);
    let common_tol: f64 = BRepTool::vertex_tolerance(&vf)?.max(BRepTool::vertex_tolerance(&vl)?);
    let mut ve_map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(base, ShapeEnum::Vertex, ShapeEnum::Edge, &mut ve_map)?;
    let _efirst: Edge = topods::edge(
        ve_map
            .find_from_key(vf.as_ref())?
            .first()
            .ok_or_else(no_edge)?,
    )?;
    let mut elast: Edge = topods::edge(
        ve_map
            .find_from_key(vl.as_ref())?
            .first()
            .ok_or_else(no_edge)?,
    )?;
    let dist: f64 = BRepTool::pnt(&vf)?.distance(&BRepTool::pnt(&vl)?);
    if dist <= common_tol {
        elast.set_free(true)?;
        vf.set_orientation(vl.orientation());
        bb.remove(&mut elast, &vl)?;
        bb.add(&mut elast, &vf)?;
        bb.update_vertex_tolerance(&vf, common_tol)?;
        base.set_closed(true)?;
    } else if dist <= max_tol {
        let new_edge: Edge = BRepLibMakeEdge::from_vertices(&vf, &vl)?.edge()?;
        bb.add(base, &new_edge)?;
        base.set_closed(true)?;
    }
    Ok(())
}

/// The closed, open and isolated wires of the offset.
#[derive(Default)]
struct WireSorts {
    closed: Vec<Wire>,
    unclosed: Vec<Wire>,
    isolated: Vec<Wire>,
}

impl BRepFillOffsetWire {
    /// `FixHoles()` (private): the open wires joined to their nearest open wires and
    /// closed; the result is the single wire left, else the compound of the closed and
    /// isolated wires.
    pub(super) fn fix_holes(&mut self) -> Result<(), Failure> {
        let max_tol: f64 = max_tolerance(self.my_spine.as_ref())?;
        let mut sorts: WireSorts = self.sorted_wires()?;
        while !sorts.unclosed.is_empty() {
            join_first_unclosed(&mut sorts, max_tol)?;
        }
        self.my_shape = result_of(sorts)?;
        Ok(())
    }

    /// The wires of the offset without repeated edges, closed or open.
    fn sorted_wires(&self) -> Result<WireSorts, Failure> {
        let mut sorts = WireSorts::default();
        let mut explo = Explorer::new(&self.my_shape, ShapeEnum::Wire, ShapeEnum::Shape)?;
        while explo.more() {
            let mut a_wire: Shape = explo.current().clone();
            remove_duplicated_edges(&mut a_wire)?;
            let w: Wire = topods::wire(&a_wire)?;
            if a_wire.closed()? {
                sorts.closed.push(w);
            } else {
                sorts.unclosed.push(w);
            }
            explo.next()?;
        }
        Ok(sorts)
    }
}

/// The first open wire joined to the nearest open wires at its ends, closed when
/// possible, and sorted.
fn join_first_unclosed(sorts: &mut WireSorts, max_tol: f64) -> Result<(), Failure> {
    let mut base: Wire = sorts.unclosed[0].clone();
    let (mut vf, mut vl) = wire_ends(&base)?;
    let (pf, pl) = (BRepTool::pnt(&vf)?, BRepTool::pnt(&vl)?);
    let (mut near_f, mut near_l) = nearest_ends(&sorts.unclosed, (pf, pl))?;
    if near_f.dist > max_tol {
        near_f.index = 0;
    }
    if near_l.dist > max_tol {
        near_l.index = 0;
    }
    let mut try_to_close_it = true;
    if near_f.dist <= max_tol
        && near_l.dist <= max_tol
        && near_f.index == near_l.index
        && near_f.is_first == near_l.is_first
    {
        if near_f.dist < near_l.dist {
            near_l.dist = f64::MAX;
            near_l.index += 1;
        } else {
            near_f.dist = f64::MAX;
            near_f.index += 1;
        }
        try_to_close_it = false;
    }
    if near_f.dist <= max_tol {
        let the_wire = sorts.unclosed[near_f.index].clone();
        join_wire(&mut base, &the_wire, near_f, (BaseEnd::First, &mut vf))?;
    }
    if near_l.dist <= max_tol && near_l.index != near_f.index {
        let the_wire = sorts.unclosed[near_l.index].clone();
        join_wire(&mut base, &the_wire, near_l, (BaseEnd::Last, &mut vl))?;
    }
    if try_to_close_it {
        try_to_close(&mut base, max_tol)?;
    }
    update_sorts(sorts, base, (near_f, near_l), max_tol)
}

/// The joined wires removed from the open ones and the base moved to the closed (or,
/// joined to nothing, isolated) wires.
fn update_sorts(
    sorts: &mut WireSorts,
    base: Wire,
    (near_f, mut near_l): (NearestEnd, NearestEnd),
    max_tol: f64,
) -> Result<(), Failure> {
    if near_f.dist <= max_tol {
        sorts.unclosed.remove(near_f.index);
    }
    if near_l.dist <= max_tol && near_l.index != near_f.index {
        if near_f.dist <= max_tol && near_l.index > near_f.index {
            near_l.index -= 1;
        }
        sorts.unclosed.remove(near_l.index);
    }
    if base.closed()? {
        sorts.closed.push(base);
        sorts.unclosed.remove(0);
    } else if near_f.dist > max_tol && near_l.dist > max_tol {
        sorts.isolated.push(base);
        sorts.unclosed.remove(0);
    }
    Ok(())
}

/// The single wire, or the compound of the closed then isolated wires.
fn result_of(sorts: WireSorts) -> Result<Shape, Failure> {
    if sorts.closed.len() + sorts.isolated.len() == 1 {
        let w = sorts
            .closed
            .into_iter()
            .chain(sorts.isolated)
            .next()
            .ok_or_else(no_edge)?;
        return Ok(w.into_shape());
    }
    let bb = BRepBuilder::default();
    let mut r = Compound::default();
    bb.make_compound(&mut r);
    for w in sorts.closed.iter().chain(sorts.isolated.iter()) {
        bb.add(&mut r, w)?;
    }
    Ok(r.into_shape())
}

/// Standard_NoSuchObject of a vertex without edge (never met: a wire vertex has one).
fn no_edge() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "BRepFill_OffsetWire::FixHoles: vertex without edge; expected an edge",
        "NCollection_List",
    )
}
