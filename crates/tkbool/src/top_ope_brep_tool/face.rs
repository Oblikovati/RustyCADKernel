// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_face.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_face`: the face of a wire on the surface of a reference face, made
//! finite (its wires reversed) when the wire bounds an infinite region.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Face, ShapeIterator, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The finite face of a wire; `let mut f = TopOpeBRepToolFace::new(); f.init(&w, &fref)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepToolFace {
    my_w: Wire,
    /// `myfinite`: uninitialized natively until `Init`; false here.
    myfinite: bool,
    my_ffinite: Face,
}

/// `FUN_reverse(f, frev)` (static): an empty copy of the face holding its reversed
/// sub-shapes.
fn fun_reverse(f: &Face) -> Result<Face, Failure> {
    let b = BRepBuilder::default();
    let mut frev = topods::face(&f.empty_copied()?)?;
    for c in ShapeIterator::new(f, true, true) {
        b.add(&mut frev, &c.reversed())?;
    }
    Ok(frev)
}

/// Standard_Failure of a face not done.
fn not_done() -> Failure {
    Failure::with_type(
        "Standard_Failure",
        "TopOpeBRepTool_face NOT DONE",
        "TopOpeBRepTool_face",
    )
}

impl TopOpeBRepToolFace {
    /// `TopOpeBRepTool_face()`; `TopOpeBRepToolFace::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init(W, Fref)`: the face of the wire on an empty copy of `fref` (natural
    /// restriction set), finite when the infinite UV point is not IN it (else its
    /// reversal); `f.init(&w, &fref)?`.
    pub fn init(&mut self, w: &Wire, fref: &Face) -> Result<bool, Failure> {
        self.my_ffinite = Face::default();
        self.my_w = w.clone();
        let mut fres = topods::face(&fref.empty_copied()?)?;
        let b = BRepBuilder::default();
        b.add(&mut fres, w)?;
        b.natural_restriction(&fres, true)?;
        let fclass = BRepTopAdaptorFClass2d::new(&fres, 0.)?;
        let infinite = fclass.perform_infinite_point()? == State::In;
        self.myfinite = !infinite;
        self.my_ffinite = if self.myfinite {
            fres
        } else {
            fun_reverse(&fres)?
        };
        Ok(true)
    }

    /// `IsDone()`; `f.is_done()`.
    pub fn is_done(&self) -> bool {
        !self.my_ffinite.is_null()
    }

    /// `Finite()`: Standard_Failure when not done; `f.finite()?`.
    pub fn finite(&self) -> Result<bool, Failure> {
        if !self.is_done() {
            return Err(not_done());
        }
        Ok(self.myfinite)
    }

    /// `Ffinite()`: Standard_Failure when not done; `f.ffinite()?`.
    pub fn ffinite(&self) -> Result<&Face, Failure> {
        if !self.is_done() {
            return Err(not_done());
        }
        Ok(&self.my_ffinite)
    }

    /// `W()`; `f.w()`.
    pub fn w(&self) -> &Wire {
        &self.my_w
    }

    /// `RealF()`: the face of the wire (the finite face, reversed back when the wire
    /// bounds an infinite region); `f.real_f()?`.
    pub fn real_f(&self) -> Result<Face, Failure> {
        if self.myfinite {
            return Ok(self.my_ffinite.clone());
        }
        fun_reverse(&self.my_ffinite)
    }
}
