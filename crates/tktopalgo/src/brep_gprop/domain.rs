// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Domain.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_Domain`: the boundary arcs of a face for the integrators.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `BRepGProp_Domain`: an iterator over the edges of a face (explored FORWARD), which
/// skips the INTERNAL and EXTERNAL edges in `Next` only: the first edge of an
/// exploration is returned whatever its orientation, as natively.
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::BRepGPropDomain;
/// let domain = BRepGPropDomain::new();
/// assert!(!domain.more());
/// ```
#[derive(Clone, Default)]
pub struct BRepGPropDomain {
    explorer: Explorer,
}

impl BRepGPropDomain {
    /// `BRepGProp_Domain()`: an empty domain (no edge); `BRepGPropDomain::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepGProp_Domain(F)`: the edges of `face`; `BRepGPropDomain::with_face(&face)?`.
    pub fn with_face(face: &Face) -> Result<Self, Failure> {
        let mut domain = Self::default();
        domain.init_face(face)?;
        Ok(domain)
    }

    /// `Init(F)`: restarts on the edges of `face` explored FORWARD;
    /// `domain.init_face(&face)?`.
    pub fn init_face(&mut self, face: &Face) -> Result<(), Failure> {
        let forward = face.oriented(Orientation::Forward);
        self.explorer
            .init(&forward, ShapeEnum::Edge, ShapeEnum::Shape)
    }

    /// `More()`: true while an edge remains; `domain.more()`.
    pub fn more(&self) -> bool {
        self.explorer.more()
    }

    /// `Init()`: restarts the exploration of the face already set; `domain.init()?`.
    pub fn init(&mut self) -> Result<(), Failure> {
        self.explorer.re_init()
    }

    /// `Value()`: the current edge; past the last edge the explorer's current shape is
    /// the face and the cast raises Standard_TypeMismatch, as natively;
    /// `domain.value()?`.
    pub fn value(&self) -> Result<Edge, Failure> {
        topods::edge(self.explorer.current())
    }

    /// `Next()`: moves to the next FORWARD or REVERSED edge; `domain.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol.
    pub fn next(&mut self) -> Result<(), Failure> {
        self.explorer.next()?;
        while self.explorer.more() {
            let orientation = self.explorer.current().orientation();
            if orientation == Orientation::Forward || orientation == Orientation::Reversed {
                return Ok(());
            }
            self.explorer.next()?;
        }
        Ok(())
    }
}
