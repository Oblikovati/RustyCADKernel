//! BRepGraph oracle, part 3, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_walk.cxx: the assembly edits, then the dense,
//! defs, refs, reverse and related iterators.
use super::brep::Scene;
use super::brep_graph::{index, node};
use super::common::{Reader, end, failure, flag, int, long, word};
use rustycad_tkbrep::brep_graph::defs_iterator::{DefsOfParent, DefsVertexOfEdge};
use rustycad_tkbrep::brep_graph::iterator::{DefIterator, RootProductIterator};
use rustycad_tkbrep::brep_graph::parent_traits::{self as traits, ParentTraits};
use rustycad_tkbrep::brep_graph::refs_iterator::{RefIterator, RefsOfParent, RefsVertexOfEdge};
use rustycad_tkbrep::brep_graph::related_iterator::RelatedIterator;
use rustycad_tkbrep::brep_graph::reverse_iterator::{
    CoEdgesOfEdge, EdgeParentTraits, EdgeParentsOf, EdgesOfVertex, FacesOfWire, IdsOfRefs,
    IdsOfRefsTraits, LookupParentRefsOf, LookupTraits, OccurrencesOfChild, ParentsOf,
    RefsCompSolidsOfSolid, RefsCompoundsOfChild, RefsEdgesOfVertex, RefsFacesOfWire,
    RefsProductsOfOccurrence, RefsShellsOfFace, RefsSolidsOfShell, RefsWiresOfCoEdge,
};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::{NodeTag, RefTag};

/// `bw_nodes`: the nodes of every valid kind, in kind then index order.
pub fn nodes(g: &BRepGraph) -> Vec<NodeId> {
    (0..12)
        .map(NodeKind)
        .filter(|kind| NodeKind::is_valid_kind(*kind))
        .flat_map(|kind| (0..g.topo().generic().nb(kind)).map(move |i| NodeId::new(kind, i)))
        .collect()
}

/// `bw_id`: a `NodeId` as `kind index`, a typed id as its index.
pub trait PrintId {
    fn print(self);
}
impl PrintId for NodeId {
    fn print(self) {
        node(self);
    }
}
impl<T> PrintId for TypedId<T> {
    fn print(self) {
        index(self);
    }
}

/// `bw_edit`: 0 Products().Add(root, location), 1 Add(), 2 AppendDocumentRoot,
/// 3 Append(parent, child, location, parent occurrence).
pub fn edit(s: &Scene, g: &mut BRepGraph, r: &mut Reader<'_>) {
    let code = r.integer();
    match code {
        0 => {
            let (kind, i) = (r.integer(), r.integer());
            let location = s.graph.loc(r.integer());
            let root = NodeId::new(NodeKind(kind), i as u32);
            index(g.editor().products().add(root, &location));
        }
        1 => index(g.editor().products().add_empty()),
        2 => {
            let product = ProductId::new(r.integer() as u32);
            match g.editor().products().append_document_root(product) {
                Ok(()) => word("-"),
                Err(error) => failure(&error),
            }
        }
        _ => append(s, g, r),
    }
}

fn append(s: &Scene, g: &mut BRepGraph, r: &mut Reader<'_>) {
    let (parent, child) = (r.integer(), r.integer());
    let location = s.graph.loc(r.integer());
    let occurrence = r.integer();
    let placing = if occurrence < 0 {
        OccurrenceId::default()
    } else {
        OccurrenceId::new(occurrence as u32)
    };
    let appended = g.editor().products().append(
        ProductId::new(parent as u32),
        ProductId::new(child as u32),
        &location,
        placing,
    );
    match appended {
        Ok((occurrence, reference)) => {
            index(occurrence);
            index(reference);
        }
        Err(error) => failure(&error),
    }
}

fn dense_iter<T: NodeTag>(g: &BRepGraph) {
    let mut it = DefIterator::<T>::new(g);
    let mut n = 0;
    while it.more() {
        index(it.current_id());
        n += 1;
        it.next();
    }
    int(n);
    int(DefIterator::<T, true>::new(g).count() as i32);
    let from = DefIterator::<T>::starting_at(g, TypedId::new(1));
    flag(from.more());
    if from.more() {
        index(from.current_id());
    }
}

fn dense_ref_iter<T: RefTag>(g: &BRepGraph) {
    let mut it = RefIterator::<T>::new(g);
    let mut n = 0;
    while it.more() {
        index(it.current_id());
        n += 1;
        it.next();
    }
    int(n);
    int(RefIterator::<T, true>::new(g).count() as i32);
    let from = RefIterator::<T>::starting_at(g, TypedId::new(1));
    flag(from.more());
    if from.more() {
        index(from.current_id());
    }
}

/// `bw_dense`.
pub fn dense(g: &BRepGraph) {
    dense_iter::<SolidTag>(g);
    dense_iter::<ShellTag>(g);
    dense_iter::<FaceTag>(g);
    dense_iter::<WireTag>(g);
    dense_iter::<EdgeTag>(g);
    dense_iter::<VertexTag>(g);
    dense_iter::<CoEdgeTag>(g);
    dense_iter::<CompoundTag>(g);
    dense_iter::<CompSolidTag>(g);
    dense_iter::<ProductTag>(g);
    dense_iter::<OccurrenceTag>(g);
    dense_ref_iter::<ShellRefTag>(g);
    dense_ref_iter::<FaceRefTag>(g);
    dense_ref_iter::<WireRefTag>(g);
    dense_ref_iter::<VertexRefTag>(g);
    dense_ref_iter::<SolidRefTag>(g);
    dense_ref_iter::<ChildRefTag>(g);
    dense_ref_iter::<OccurrenceRefTag>(g);
    RootProductIterator::new(g).for_each(index);
    end();
}

// `bw_parent` and `bw_parent_refs` of one trait over the parents 0..=Nb.
fn parent<P: ParentTraits>(g: &BRepGraph, p: TypedId<P::Parent>)
where
    P::Child: PrintId,
    P::Ref: PrintId,
{
    let mut count = DefsOfParent::<P>::new(g, p);
    let mut n = 0;
    while count.more() {
        n += 1;
        count.next();
    }
    int(n);
    let mut it = DefsOfParent::<P>::new(g, p);
    while it.more() {
        it.current_id().print();
        it.current_ref_id().print();
        long(u64::from(it.index()));
        it.next();
    }
}
fn parent_refs<P: ParentTraits>(g: &BRepGraph, p: TypedId<P::Parent>)
where
    P::Ref: PrintId,
{
    int(RefsOfParent::<P>::new(g, p).count() as i32);
    let mut it = RefsOfParent::<P>::new(g, p);
    while it.more() {
        it.current_id().print();
        long(u64::from(it.index()));
        it.next();
    }
}
fn both<P: ParentTraits>(g: &BRepGraph, nb: u32)
where
    P::Child: PrintId,
    P::Ref: PrintId,
{
    for i in 0..=nb {
        parent::<P>(g, TypedId::new(i));
        parent_refs::<P>(g, TypedId::new(i));
    }
}

/// `bw_children`.
pub fn children(g: &BRepGraph) {
    let t = g.topo();
    both::<traits::ShellOfSolidTraits>(g, t.solids().nb());
    both::<traits::FaceOfShellTraits>(g, t.shells().nb());
    both::<traits::WireOfFaceTraits>(g, t.faces().nb());
    for i in 0..=t.wires().nb() {
        parent::<traits::CoEdgeOfWireTraits>(g, WireId::new(i));
        parent::<traits::EdgeOfWireTraits>(g, WireId::new(i));
        parent_refs::<traits::CoEdgeOfWireTraits>(g, WireId::new(i));
    }
    both::<traits::SolidOfCompSolidTraits>(g, t.comp_solids().nb());
    both::<traits::ChildOfCompoundTraits>(g, t.compounds().nb());
    both::<traits::OccurrenceOfProductTraits>(g, t.products().nb());
    for i in 0..=t.edges().nb() {
        let mut defs = DefsVertexOfEdge::new(g, EdgeId::new(i));
        while defs.more() {
            index(defs.current_id());
            index(defs.current_ref_id());
            long(u64::from(defs.index()));
            defs.next();
        }
        let mut refs = RefsVertexOfEdge::new(g, EdgeId::new(i));
        while refs.more() {
            index(refs.current_id());
            long(u64::from(refs.index()));
            refs.next();
        }
        word("|");
    }
    end();
}

/// The reverse iterators `bw_ids` prints: the current id and the index.
trait IdWalk {
    fn more(&self) -> bool;
    fn step(&mut self);
    fn print_current(&self);
}
impl<T: NodeTag> IdWalk for ParentsOf<'_, T> {
    fn more(&self) -> bool {
        ParentsOf::more(self)
    }
    fn step(&mut self) {
        ParentsOf::next(self);
    }
    fn print_current(&self) {
        index(self.current_id());
        long(u64::from(self.index()));
    }
}
impl<P: EdgeParentTraits> IdWalk for EdgeParentsOf<'_, P> {
    fn more(&self) -> bool {
        EdgeParentsOf::more(self)
    }
    fn step(&mut self) {
        EdgeParentsOf::next(self);
    }
    fn print_current(&self) {
        index(self.current_id());
        long(u64::from(self.index()));
    }
}
impl<I: IdsOfRefsTraits> IdWalk for IdsOfRefs<'_, I> {
    fn more(&self) -> bool {
        IdsOfRefs::more(self)
    }
    fn step(&mut self) {
        IdsOfRefs::next(self);
    }
    fn print_current(&self) {
        index(self.current_id());
        long(u64::from(self.index()));
    }
}
/// `bw_ids`.
fn ids(mut it: impl IdWalk) {
    let mut n = 0;
    while it.more() {
        it.print_current();
        n += 1;
        it.step();
    }
    int(n);
}

/// The ref-based reverse iterators `bw_ref_ids` prints.
trait RefWalk {
    fn more(&self) -> bool;
    fn step(&mut self);
    fn print_current(&self);
}
impl<I: IdsOfRefsTraits> RefWalk for IdsOfRefs<'_, I> {
    fn more(&self) -> bool {
        IdsOfRefs::more(self)
    }
    fn step(&mut self) {
        IdsOfRefs::next(self);
    }
    fn print_current(&self) {
        index(self.current_parent_id());
        index(self.current_ref_id());
        long(u64::from(self.index()));
    }
}
impl<L: LookupTraits> RefWalk for LookupParentRefsOf<'_, L>
where
    L::Ref: PrintId,
{
    fn more(&self) -> bool {
        LookupParentRefsOf::more(self)
    }
    fn step(&mut self) {
        LookupParentRefsOf::next(self);
    }
    fn print_current(&self) {
        index(self.current_parent_id());
        self.current_ref_id().print();
        long(u64::from(self.index()));
    }
}
/// `bw_ref_ids`.
fn ref_ids(mut it: impl RefWalk) {
    while it.more() {
        it.print_current();
        it.step();
    }
    word("|");
}

fn edge_parents(g: &BRepGraph) {
    let t = g.topo();
    for i in 0..=t.edges().nb() {
        let e = EdgeId::new(i);
        ids(CoEdgesOfEdge::new(g, t.edges().co_edges(e)));
        for start in 0..3 {
            ids(t.edges().wires_of_from(e, start));
            ids(t.edges().faces_of_from(e, start));
        }
    }
    let wires: Vec<WireId> = (0..=t.wires().nb()).map(WireId::new).collect();
    for i in 0..=t.co_edges().nb() {
        ref_ids(RefsWiresOfCoEdge::new(g, &wires, CoEdgeId::new(i)));
    }
}

/// `bw_reverse`.
pub fn reverse(g: &BRepGraph) {
    let t = g.topo();
    for i in 0..t.vertices().nb() {
        let v = VertexId::new(i);
        let it = EdgesOfVertex::new(g, t.vertices().edges(v));
        long(it.size() as u64);
        ids(it);
        ref_ids(RefsEdgesOfVertex::new(g, t.vertices().edges(v), v));
    }
    edge_parents(g);
    for i in 0..t.wires().nb() {
        let refs = &t.wires().relations(WireId::new(i)).parent_wire_ref_ids;
        ref_ids(RefsFacesOfWire::new(g, refs));
        ids(FacesOfWire::starting_at(g, refs, 1));
    }
    for i in 0..t.faces().nb() {
        ref_ids(RefsShellsOfFace::new(
            g,
            &t.faces().relations(FaceId::new(i)).parent_face_ref_ids,
        ));
    }
    for i in 0..t.shells().nb() {
        let refs = &t.shells().relations(ShellId::new(i)).parent_shell_ref_ids;
        ref_ids(RefsSolidsOfShell::new(g, refs));
    }
    for i in 0..t.solids().nb() {
        let refs = &t.solids().relations(SolidId::new(i)).parent_solid_ref_ids;
        ref_ids(RefsCompSolidsOfSolid::new(g, refs));
    }
    for i in 0..t.occurrences().nb() {
        let refs = &t
            .occurrences()
            .relations(OccurrenceId::new(i))
            .parent_occurrence_ref_ids;
        ref_ids(RefsProductsOfOccurrence::new(g, refs));
    }
    for n in nodes(g) {
        ref_ids(RefsCompoundsOfChild::new(
            g,
            t.generic().compound_ref_ids(n),
        ));
        ids(OccurrencesOfChild::new(
            g,
            t.generic().occurrence_ref_ids(n),
        ));
    }
    end();
}

/// `bw_related`: every node, then the first index past each kind's end.
pub fn related(g: &BRepGraph) {
    let mut all = nodes(g);
    all.extend((0..12).map(|k| NodeId::new(NodeKind(k), g.topo().generic().nb(NodeKind(k)))));
    for n in all {
        match RelatedIterator::new(g, n) {
            Ok(mut it) => {
                while it.more() {
                    node(it.current());
                    int(it.current_relation() as i32);
                    if let Err(error) = it.next() {
                        failure(&error);
                        break;
                    }
                }
            }
            Err(error) => failure(&error),
        }
        word("|");
    }
    end();
}
