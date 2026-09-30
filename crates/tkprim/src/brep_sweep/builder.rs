// Rust translation of BRepSweep_Builder.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepSweep_Builder`: the topological services of the sweeps, on the BRep builder.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// `BRepSweep_Builder`: makes the empty containers of the sweeps and adds sub-shapes
/// with a given orientation (docs/prim-design.md section 1: the slot to fill is passed
/// by `&mut`).
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkprim::brep_sweep::BRepSweepBuilder;
/// let builder = BRepSweepBuilder::new(BRepBuilder::default());
/// let mut wire = Shape::default();
/// builder.make_wire(&mut wire)?;
/// assert!(!wire.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepSweepBuilder {
    my_builder: BRepBuilder,
}

/// Casts `slot` to its typed shape (`TopoDS::X(slot)`, Standard_TypeMismatch for
/// another type), lets `make` rebuild it, and writes it back.
fn remake<T: AsRef<Shape> + Into<Shape>>(
    slot: &mut Shape,
    cast: fn(&Shape) -> Result<T, Failure>,
    make: impl FnOnce(&mut T),
) -> Result<(), Failure> {
    let mut typed = cast(slot)?;
    make(&mut typed);
    *slot = typed.into();
    Ok(())
}

impl BRepSweepBuilder {
    /// `BRepSweep_Builder(aBuilder)`; `BRepSweepBuilder::new(BRepBuilder::default())`.
    pub fn new(builder: BRepBuilder) -> Self {
        Self {
            my_builder: builder,
        }
    }

    /// `Builder()`: the BRep builder; `builder.builder()`.
    pub fn builder(&self) -> &BRepBuilder {
        &self.my_builder
    }

    /// `MakeCompound(aCompound)`: an empty compound in `slot`;
    /// `builder.make_compound(&mut shape)?`.
    pub fn make_compound(&self, slot: &mut Shape) -> Result<(), Failure> {
        remake(slot, topods::compound, |c| self.my_builder.make_compound(c))
    }

    /// `MakeCompSolid(aCompSolid)`: an empty composite solid in `slot`;
    /// `builder.make_comp_solid(&mut shape)?`.
    pub fn make_comp_solid(&self, slot: &mut Shape) -> Result<(), Failure> {
        remake(slot, topods::comp_solid, |c| {
            self.my_builder.make_comp_solid(c)
        })
    }

    /// `MakeSolid(aSolid)`: an empty solid in `slot`; `builder.make_solid(&mut shape)?`.
    pub fn make_solid(&self, slot: &mut Shape) -> Result<(), Failure> {
        remake(slot, topods::solid, |s| self.my_builder.make_solid(s))
    }

    /// `MakeShell(aShell)`: an empty shell in `slot`; `builder.make_shell(&mut shape)?`.
    pub fn make_shell(&self, slot: &mut Shape) -> Result<(), Failure> {
        remake(slot, topods::shell, |s| self.my_builder.make_shell(s))
    }

    /// `MakeWire(aWire)`: an empty wire in `slot`; `builder.make_wire(&mut shape)?`.
    pub fn make_wire(&self, slot: &mut Shape) -> Result<(), Failure> {
        remake(slot, topods::wire, |w| self.my_builder.make_wire(w))
    }

    /// `Add(aShape1, aShape2, Orient)`: adds `shape2` with the orientation `orient` to
    /// `shape1`; `builder.add_oriented(&mut wire, &edge, Orientation::Reversed)?`.
    pub fn add_oriented(
        &self,
        shape1: &mut Shape,
        shape2: &Shape,
        orient: Orientation,
    ) -> Result<(), Failure> {
        let component = shape2.oriented(orient);
        self.my_builder.add(shape1, &component)
    }

    /// `Add(aShape1, aShape2)`: adds `shape2` as it is to `shape1`;
    /// `builder.add(&mut shell, &face)?`.
    pub fn add(&self, shape1: &mut Shape, shape2: &Shape) -> Result<(), Failure> {
        self.my_builder.add(shape1, shape2)
    }
}
