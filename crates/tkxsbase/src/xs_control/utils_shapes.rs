// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Utils (.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871: the shape conversions.

//! The shape conversions of [`XSControlUtils`]: compounds, sorted compounds, lists of
//! shapes and shape binders.
use super::XSControlUtils;
use crate::transfer::TransferBinder;
use crate::transfer_brep::{TransferBRep, TransferBRepShapeBinder, TransferBRepShapeMapper};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, HShape, Shape, ShapeIterator, Shell, Wire};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle, StandardTransient, exceptions},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_extend::ShapeExtendExplorer;

/// A list of shapes (`NCollection_HSequence<TopoDS_Shape>`).
pub type XSControlShapeSeq = HSequence<Shape>;

/// The shapes collected by `SortedCompound` from the sub-shapes of a compound (or a
/// solid): a null shape for none, the shape alone for one, else their compound.
struct Collected {
    builder: BRepBuilder,
    compound: Compound,
    nb: i32,
    last: Shape,
}

impl Collected {
    fn new() -> Self {
        let (builder, mut compound) = (BRepBuilder::default(), Compound::default());
        builder.make_compound(&mut compound);
        Self {
            builder,
            compound,
            nb: 0,
            last: Shape::default(),
        }
    }
    fn add(&mut self, sh: &Shape) -> Result<(), Failure> {
        self.nb += 1;
        self.last = sh.clone();
        self.builder.add(&mut self.compound, sh)
    }
    fn result(self) -> Shape {
        match self.nb {
            0 => Shape::default(),
            1 => self.last,
            _ => self.compound.into_shape(),
        }
    }
}

impl XSControlUtils {
    /// `CompoundFromSeq(seqval)`: a compound of the shapes of a list;
    /// `Standard_NullObject` for a null list.
    pub fn compound_from_seq(&self, seqval: &Handle<XSControlShapeSeq>) -> Result<Shape, Failure> {
        ShapeExtendExplorer::new().compound_from_seq(seqval)
    }
    /// `ShapeType(shape, compound)`: the type of a shape, for a compound (with
    /// `compound`) the common type of its items (EDGE and WIRE give WIRE, FACE and SHELL
    /// give SHELL, different types give COMPOUND); SHAPE for null.
    pub fn shape_type(&self, shape: &Shape, compound: bool) -> Result<ShapeEnum, Failure> {
        ShapeExtendExplorer::new().shape_type(shape, compound)
    }
    /// `SortedCompound(shape, type, explore, compound)`: the sub-shapes of `shape` of
    /// type `type` (an edge as a wire, a face as a closed-checked shell), exploring the
    /// others with `explore`; for a compound its items sorted (the sorted compounds
    /// flattened unless `compound`). A null shape for none, the shape alone for one.
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
        if typ == kind {
            return Ok(shape.clone());
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
        let mut collected = Collected::new();
        let mut exp = Explorer::new(shape, kind, ShapeEnum::Shape)?;
        while exp.more() {
            collected.add(&exp.current().clone())?;
            exp.next()?;
        }
        Ok(collected.result())
    }
    /// The items of a compound (or solid) sorted, the sorted compounds flattened with
    /// `flatten`.
    fn sorted_items(
        &self,
        shape: &Shape,
        kind: ShapeEnum,
        explore: bool,
        compound: bool,
        flatten: bool,
    ) -> Result<Shape, Failure> {
        let mut collected = Collected::new();
        let mut it = ShapeIterator::new(shape, true, true);
        while it.more() {
            let sh = self.sorted_compound(it.value()?, kind, explore, compound)?;
            it.next();
            if sh.is_null() {
                continue;
            }
            if flatten && sh.shape_type()? == ShapeEnum::Compound {
                let mut it2 = ShapeIterator::new(&sh, true, true);
                while it2.more() {
                    collected.add(&it2.value()?.clone())?;
                    it2.next();
                }
            } else {
                collected.add(&sh)?;
            }
        }
        Ok(collected.result())
    }
    /// `ShapeValue(seqv, num)`: the shape `num` of a list (null out of range or for a
    /// null list).
    pub fn shape_value(&self, seqval: &Handle<XSControlShapeSeq>, num: i32) -> Shape {
        let Some(list) = seqval.get() else {
            return Shape::default();
        };
        let index = usize::try_from(num).unwrap_or(0);
        if index == 0 {
            return Shape::default();
        }
        list.sequence(|s| s.value(index).ok().cloned())
            .unwrap_or_default()
    }
    /// `NewSeqShape`.
    pub fn new_seq_shape(&self) -> Handle<XSControlShapeSeq> {
        Handle::new(HSequence::default())
    }
    /// `AppendShape(seqv, shape)`: `Standard_NullObject` for a null list.
    pub fn append_shape(
        &self,
        seqval: &Handle<XSControlShapeSeq>,
        shape: &Shape,
    ) -> Result<(), Failure> {
        let list = seqval
            .get()
            .ok_or_else(|| exceptions::null_object("XSControl_Utils::AppendShape: null list"))?;
        list.append(shape.clone());
        Ok(())
    }
    /// `ShapeBinder(shape, hs = true)`: the shape as a handle shape (`hs`), else as a
    /// shape binder.
    pub fn shape_binder(&self, shape: &Shape, hs: bool) -> Handle<StandardTransient> {
        if hs {
            return Handle::new(HShape::new(shape)).cast();
        }
        Handle::new(TransferBRepShapeBinder::with_result(shape)).cast()
    }
    /// `BinderShape(tr)`: the shape of a binder, a shape mapper or a handle shape; null
    /// otherwise.
    pub fn binder_shape(&self, tr: &Handle<StandardTransient>) -> Shape {
        let binder = tr.cast::<dyn TransferBinder>();
        if binder.is_some() {
            return TransferBRep::shape_result(&binder);
        }
        if let Some(sm) = tr.cast::<TransferBRepShapeMapper>().get() {
            return sm.value().clone();
        }
        tr.cast::<HShape>()
            .get()
            .map(HShape::shape)
            .unwrap_or_default()
    }
}

/// The pseudo-equalities of `SortedCompound`: an edge as a wire, a face as a shell
/// (closed when it is); `None` otherwise.
fn pseudo_equal(shape: &Shape, typ: ShapeEnum, kind: ShapeEnum) -> Result<Option<Shape>, Failure> {
    let builder = BRepBuilder::default();
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
        let closed = BRepTool::is_closed(&shell)?;
        shell.set_closed(closed)?;
        return Ok(Some(shell.into_shape()));
    }
    Ok(None)
}
