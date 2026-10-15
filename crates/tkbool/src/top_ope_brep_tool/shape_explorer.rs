// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_ShapeExplorer.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_ShapeExplorer`: a `TopExp_Explorer` counting the explored shapes.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use std::io::{self, Write};
use std::ops::Deref;

/// An explorer with the index of its current shape (from 1, 0 when exhausted at
/// start); the explorer members are reached through `Deref`;
/// `let ex = TopOpeBRepToolShapeExplorer::new(&s, ShapeEnum::Edge, ShapeEnum::Shape)?;`.
#[derive(Default)]
pub struct TopOpeBRepToolShapeExplorer {
    explorer: Explorer,
    my_index: i32,
}

impl Deref for TopOpeBRepToolShapeExplorer {
    type Target = Explorer;

    fn deref(&self) -> &Explorer {
        &self.explorer
    }
}

impl TopOpeBRepToolShapeExplorer {
    /// `TopOpeBRepTool_ShapeExplorer(S, ToFind, ToAvoid)` (ToAvoid natively defaulted
    /// to SHAPE); `TopOpeBRepToolShapeExplorer::new(&s, ShapeEnum::Face, ShapeEnum::Shape)?`.
    pub fn new(s: &Shape, to_find: ShapeEnum, to_avoid: ShapeEnum) -> Result<Self, Failure> {
        let explorer = Explorer::new(s, to_find, to_avoid)?;
        let my_index = i32::from(explorer.more());
        Ok(Self { explorer, my_index })
    }

    /// `Init(S, ToFind, ToAvoid)`; `ex.init(&s, ShapeEnum::Edge, ShapeEnum::Shape)?`.
    pub fn init(
        &mut self,
        s: &Shape,
        to_find: ShapeEnum,
        to_avoid: ShapeEnum,
    ) -> Result<(), Failure> {
        self.explorer.init(s, to_find, to_avoid)?;
        self.my_index = i32::from(self.explorer.more());
        Ok(())
    }

    /// `Next()`: the index advanced while there is a current shape; `ex.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, as TopExp_Explorer.
    pub fn next(&mut self) -> Result<(), Failure> {
        if self.explorer.more() {
            self.my_index += 1;
        }
        self.explorer.next()
    }

    /// `Index()`; `ex.index()`.
    pub fn index(&self) -> i32 {
        self.my_index
    }

    /// `DumpCurrent(OS)`: `TYPE(index,ORIENTATION) ` of the current shape;
    /// `ex.dump_current(&mut out)?`.
    pub fn dump_current(&self, os: &mut impl Write) -> io::Result<()> {
        if !self.explorer.more() {
            return Ok(());
        }
        let current = self.explorer.current();
        if let Ok(t) = current.shape_type() {
            top_abs::print_shape_type(t, os)?;
        }
        write!(os, "({},", self.my_index)?;
        top_abs::print_orientation(current.orientation(), os)?;
        write!(os, ") ")
    }
}
