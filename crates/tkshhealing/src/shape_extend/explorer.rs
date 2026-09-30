// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend/ShapeExtend_Explorer.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeExtend_Explorer`: conversions between compounds, sequences and lists of shapes,
//! and the sorting of a shape by type.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator, Shell, Wire};
use rustycad_tkernel::ncollection::{HSequence, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `occ::handle<NCollection_HSequence<TopoDS_Shape>>`.
pub type ShapeHSequence = Handle<HSequence<Shape>>;

/// The eight sequences `DispatchList` fills, one per shape type.
#[derive(Clone, Default)]
pub struct ShapeDispatch {
    /// The vertices.
    pub vertices: ShapeHSequence,
    /// The edges.
    pub edges: ShapeHSequence,
    /// The wires.
    pub wires: ShapeHSequence,
    /// The faces.
    pub faces: ShapeHSequence,
    /// The shells.
    pub shells: ShapeHSequence,
    /// The solids.
    pub solids: ShapeHSequence,
    /// The composite solids.
    pub compsols: ShapeHSequence,
    /// The compounds.
    pub compounds: ShapeHSequence,
}

/// `ShapeExtend_Explorer`: tools to explore shapes and to convert their collections.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkshhealing::shape_extend::ShapeExtendExplorer;
/// let explorer = ShapeExtendExplorer::new();
/// assert_eq!(explorer.shape_type(&Shape::default(), true).unwrap(), ShapeEnum::Shape);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeExtendExplorer;

/// A new empty sequence handle.
fn new_sequence() -> ShapeHSequence {
    Handle::new(HSequence::default())
}

/// The shapes of a sequence handle, in order.
fn items(sequence: &ShapeHSequence, member: &str) -> Result<Vec<Shape>, Failure> {
    let Some(sequence) = sequence.get() else {
        return Err(Failure::with_message(
            "Standard_NullObject",
            &format!("{member}: null sequence handle; expected a sequence of shapes"),
        ));
    };
    Ok(sequence.sequence(|s| s.iter().cloned().collect()))
}

impl ShapeExtendExplorer {
    /// `ShapeExtend_Explorer()`; `ShapeExtendExplorer::new()`.
    pub fn new() -> Self {
        Self
    }

    /// `CompoundFromSeq(seqval)`: a new compound of the shapes of the sequence;
    /// Standard_NullObject for a null handle (a null dereference natively);
    /// `explorer.compound_from_seq(&shapes)?`.
    pub fn compound_from_seq(&self, seqval: &ShapeHSequence) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        for shape in items(seqval, "ShapeExtend_Explorer::CompoundFromSeq")? {
            builder.add(&mut compound, &shape)?;
        }
        Ok(compound.into_shape())
    }

    /// `SeqFromCompound(comp, expcomp)`: the sub-shapes of a compound (those of its
    /// sub-compounds too with `expcomp`); a non-compound gives itself, a null shape an
    /// empty sequence; `explorer.seq_from_compound(&c, true)?`.
    pub fn seq_from_compound(
        &self,
        comp: &Shape,
        expcomp: bool,
    ) -> Result<ShapeHSequence, Failure> {
        let list = new_sequence();
        if comp.is_null() {
            return Ok(list);
        }
        if comp.shape_type()? != ShapeEnum::Compound {
            list.append(comp.clone());
            return Ok(list);
        }
        fill_list(&list, comp, expcomp)?;
        Ok(list)
    }

    /// `ListFromSeq(seqval, lisval, clear)`: appends the shapes of the sequence to the
    /// list (cleared first with `clear`); a null handle adds nothing;
    /// `explorer.list_from_seq(&seq, &mut list, true)`.
    pub fn list_from_seq(&self, seqval: &ShapeHSequence, lisval: &mut List<Shape>, clear: bool) {
        if clear {
            lisval.clear();
        }
        if let Some(sequence) = seqval.get() {
            sequence.sequence(|s| s.iter().for_each(|shape| _ = lisval.append(shape.clone())));
        }
    }

    /// `SeqFromList(lisval)`: a new sequence of the shapes of the list;
    /// `explorer.seq_from_list(&list)`.
    pub fn seq_from_list(&self, lisval: &List<Shape>) -> ShapeHSequence {
        let sequence = new_sequence();
        lisval
            .iter()
            .for_each(|shape| sequence.append(shape.clone()));
        sequence
    }

    /// `ShapeType(shape, compound)`: the type of the shape, TopAbs_SHAPE for a null one.
    /// With `compound`, a compound gives the common type of its content (a WIRE for edges
    /// and wires, a SHELL for faces and shells, COMPOUND for other mixes, SHAPE when
    /// empty); `explorer.shape_type(&s, true)?`.
    pub fn shape_type(&self, shape: &Shape, compound: bool) -> Result<ShapeEnum, Failure> {
        if shape.is_null() {
            return Ok(ShapeEnum::Shape);
        }
        let kind = shape.shape_type()?;
        if !compound || kind != ShapeEnum::Compound {
            return Ok(kind);
        }
        let mut result = ShapeEnum::Shape;
        let mut it = ShapeIterator::new(shape, true, true);
        while it.more() {
            let sub = it.value()?.clone();
            it.next();
            if sub.is_null() {
                continue;
            }
            let mut sub_kind = sub.shape_type()?;
            if sub_kind == ShapeEnum::Compound {
                sub_kind = self.shape_type(&sub, compound)?;
            }
            match merge_types(result, sub_kind) {
                Some(merged) => result = merged,
                None => return Ok(ShapeEnum::Compound),
            }
        }
        Ok(result)
    }
}

/// One step of `ShapeType`: the running type with the type of the next sub-shape, `None`
/// for two types that do not merge (the result is then COMPOUND).
fn merge_types(result: ShapeEnum, kind: ShapeEnum) -> Option<ShapeEnum> {
    use ShapeEnum::{Edge, Face, Shell, Wire};
    match (result, kind) {
        (ShapeEnum::Shape, _) | (Edge, Wire) | (Face, Shell) => Some(kind),
        (Wire, Edge) | (Shell, Face) => Some(result),
        _ if result != kind => None,
        _ => Some(result),
    }
}

/// The static `FillList(list, comp, expcomp)`: appends the sub-shapes of `comp`, those of
/// its sub-compounds recursively with `expcomp`.
fn fill_list(list: &ShapeHSequence, comp: &Shape, expcomp: bool) -> Result<(), Failure> {
    let mut it = ShapeIterator::new(comp, true, true);
    while it.more() {
        let sub = it.value()?.clone();
        it.next();
        if sub.shape_type()? == ShapeEnum::Compound && expcomp {
            fill_list(list, &sub, expcomp)?;
        } else {
            list.append(sub);
        }
    }
    Ok(())
}

impl ShapeExtendExplorer {
    /// `SortedCompound(shape, type, explore, compound)`: the part of `shape` of type
    /// `type`: the shape itself when of that type (an EDGE in a new wire, a FACE in a new
    /// shell for a WIRE or SHELL request); a compound or composite solid is sorted item by
    /// item (sub-compounds flattened unless `compound`); with `explore` the sub-shapes of
    /// that type (a SOLID gives the sorted SHELLs with `compound`). A single result is
    /// returned as such, none as a null shape; `explorer.sorted_compound(&s, ShapeEnum::Face, true, false)?`.
    pub fn sorted_compound(
        &self,
        shape: &Shape,
        kind: ShapeEnum,
        explore: bool,
        compound: bool,
    ) -> Result<Shape, Failure> {
        if shape.is_null() {
            return Ok(shape.clone());
        }
        let typ = shape.shape_type()?;
        if typ == ShapeEnum::Compound || typ == ShapeEnum::CompSolid {
            return self.sorted_items(shape, kind, explore, compound, !compound);
        }
        if let Some(pseudo) = pseudo_equal(shape, typ, kind)? {
            return Ok(pseudo);
        }
        if !explore {
            return Ok(Shape::default());
        }
        if typ == ShapeEnum::Solid && compound {
            return self.sorted_items(shape, kind, explore, compound, false);
        }
        explored(shape, kind)
    }

    /// The compound branches of `SortedCompound`: each item sorted, the items of a sorted
    /// compound added one by one with `flatten`; a null shape for no item, the item alone
    /// for one.
    fn sorted_items(
        &self,
        shape: &Shape,
        kind: ShapeEnum,
        explore: bool,
        compound: bool,
        flatten: bool,
    ) -> Result<Shape, Failure> {
        let (builder, mut result) = (BRepBuilder::default(), Compound::default());
        builder.make_compound(&mut result);
        let (mut nb, mut last) = (0, Shape::default());
        let mut it = ShapeIterator::new(shape, true, true);
        while it.more() {
            let sorted = self.sorted_compound(it.value()?, kind, explore, compound)?;
            it.next();
            if sorted.is_null() {
                continue;
            }
            // `sh = sh0` before the flattening loop, as natively.
            last = sorted.clone();
            let parts = if flatten && sorted.shape_type()? == ShapeEnum::Compound {
                children(&sorted)?
            } else {
                vec![sorted]
            };
            for part in parts {
                nb += 1;
                builder.add(&mut result, &part)?;
                last = part;
            }
        }
        Ok(single_or_compound(nb, last, result))
    }
}

/// `SortedCompound` for a type equal or pseudo-equal to the request: the shape itself, an
/// edge in a new wire, a face in a new shell; `None` otherwise.
fn pseudo_equal(shape: &Shape, typ: ShapeEnum, kind: ShapeEnum) -> Result<Option<Shape>, Failure> {
    let builder = BRepBuilder::default();
    if typ == kind {
        return Ok(Some(shape.clone()));
    }
    if typ == ShapeEnum::Edge && kind == ShapeEnum::Wire {
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        builder.add(&mut wire, shape)?;
        return Ok(Some(wire.into_shape()));
    }
    if typ == ShapeEnum::Face && kind == ShapeEnum::Shell {
        let mut shell = Shell::default();
        builder.make_shell(&mut shell);
        builder.add(&mut shell, shape)?;
        return Ok(Some(shell.into_shape()));
    }
    Ok(None)
}

/// The classic exploration of `SortedCompound`: the sub-shapes of type `kind`.
fn explored(shape: &Shape, kind: ShapeEnum) -> Result<Shape, Failure> {
    let (builder, mut result) = (BRepBuilder::default(), Compound::default());
    builder.make_compound(&mut result);
    let (mut nb, mut last) = (0, Shape::default());
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        nb += 1;
        last = explorer.current().clone();
        builder.add(&mut result, &last)?;
        explorer.next()?;
    }
    Ok(single_or_compound(nb, last, result))
}

/// The children of a shape, in iteration order.
fn children(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let mut parts = Vec::new();
    let mut it = ShapeIterator::new(shape, true, true);
    while it.more() {
        parts.push(it.value()?.clone());
        it.next();
    }
    Ok(parts)
}

/// `if (nb == 0) C.Nullify(); else if (nb == 1) return sh; return C;`.
fn single_or_compound(nb: i32, last: Shape, compound: Compound) -> Shape {
    match nb {
        0 => Shape::default(),
        1 => last,
        _ => compound.into_shape(),
    }
}

impl ShapeExtendExplorer {
    /// `DispatchList(list, vertices, ..., compounds)`: appends each non-null shape of the
    /// list to the sequence of its type; a null output handle gets a new sequence first
    /// (all eight, before the first shape), and a null list does nothing;
    /// `explorer.dispatch_list(&list, &mut dispatch)`.
    pub fn dispatch_list(
        &self,
        list: &ShapeHSequence,
        dispatch: &mut ShapeDispatch,
    ) -> Result<(), Failure> {
        if list.is_null() {
            return Ok(());
        }
        for sequence in dispatch.sequences_mut() {
            if sequence.is_null() {
                *sequence = new_sequence();
            }
        }
        for shape in items(list, "ShapeExtend_Explorer::DispatchList")? {
            if shape.is_null() {
                continue;
            }
            if let Some(target) = dispatch.of_type(shape.shape_type()?) {
                target.append(shape);
            }
        }
        Ok(())
    }
}

impl ShapeDispatch {
    /// The eight sequences, in the native argument order.
    fn sequences_mut(&mut self) -> [&mut ShapeHSequence; 8] {
        [
            &mut self.vertices,
            &mut self.edges,
            &mut self.wires,
            &mut self.faces,
            &mut self.shells,
            &mut self.solids,
            &mut self.compsols,
            &mut self.compounds,
        ]
    }

    /// The sequence of a shape type, none for TopAbs_SHAPE.
    fn of_type(&self, kind: ShapeEnum) -> Option<&HSequence<Shape>> {
        let sequence = match kind {
            ShapeEnum::Vertex => &self.vertices,
            ShapeEnum::Edge => &self.edges,
            ShapeEnum::Wire => &self.wires,
            ShapeEnum::Face => &self.faces,
            ShapeEnum::Shell => &self.shells,
            ShapeEnum::Solid => &self.solids,
            ShapeEnum::CompSolid => &self.compsols,
            ShapeEnum::Compound => &self.compounds,
            ShapeEnum::Shape => return None,
        };
        sequence.get()
    }
}
