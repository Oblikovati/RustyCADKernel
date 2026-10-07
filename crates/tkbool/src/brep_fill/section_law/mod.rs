// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_SectionLaw.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_SectionLaw`: the sections of a sweep, one GeomFill section law per edge.
mod nsections;
mod nsections_surface;
mod shape_law;
mod vertex_tol;

pub use nsections::BRepFillNSections;
pub use shape_law::BRepFillShapeLaw;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkgeomalgo::geom_fill::GeomFillSectionLaw;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The protected data of `BRepFill_SectionLaw` (`myLaws`, `uclosed`, `vclosed`,
/// `myDone`, `myIndices`) and its private wire explorer, embedded in each concrete law
/// and returned by [`BRepFillSectionLaw::section_law_base`].
#[derive(Default)]
pub struct BRepFillSectionLawBase {
    /// `myLaws`: `None` while not built (a null handle natively).
    pub(crate) laws: Option<Vec<Handle<dyn GeomFillSectionLaw>>>,
    pub(crate) uclosed: bool,
    pub(crate) vclosed: bool,
    pub(crate) done: bool,
    pub(crate) indices: DataMap<Shape, i32, ShapeMapHasher>,
    /// `myIterator`, advanced by `CurrentEdge` through a shared law.
    iterator: SharedState<WireExplorer>,
}

impl BRepFillSectionLawBase {
    /// The laws; Standard_NullObject while they are not built.
    pub(crate) fn built_laws(&self) -> Result<&[Handle<dyn GeomFillSectionLaw>], Failure> {
        self.laws.as_deref().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_SectionLaw: the section laws are not built; expected a law built with Build = true",
                "BRepFill_SectionLaw",
            )
        })
    }
}

/// Standard_OutOfRange for an index outside `1..=count`.
pub(crate) fn index_error(member: &str, index: i32, count: usize) -> Failure {
    let message = format!("{member}: index {index}; expected 1..={count}");
    Failure::with_type("Standard_OutOfRange", &message, "BRepFill_SectionLaw")
}

/// `BRepFill_SectionLaw`: the abstract section law of a sweep, reached through
/// `Handle<dyn BRepFillSectionLaw>`; the concrete laws are [`BRepFillShapeLaw`] and
/// [`BRepFillNSections`]. Every member takes `&self`, the laws being shared.
/// `let n = law.nb_law()?;`
pub trait BRepFillSectionLaw: Transient {
    /// The embedded protected data.
    fn section_law_base(&self) -> &BRepFillSectionLawBase;

    /// `NbLaw()`: the number of elementary laws.
    fn nb_law(&self) -> Result<i32, Failure> {
        Ok(self.section_law_base().built_laws()?.len() as i32)
    }

    /// `Law(Index)`: the elementary law of index `index`.
    fn law(&self, index: i32) -> Result<Handle<dyn GeomFillSectionLaw>, Failure> {
        let laws = self.section_law_base().built_laws()?;
        let found = usize::try_from(index - 1).ok().and_then(|i| laws.get(i));
        found
            .cloned()
            .ok_or_else(|| index_error("BRepFill_SectionLaw::Law", index, laws.len()))
    }

    /// `IndexOfEdge(anEdge)`: the index of the law of an edge (negative for a
    /// REVERSED edge of BRepFill_NSections); Standard_NoSuchObject for another edge.
    fn index_of_edge(&self, an_edge: &Shape) -> Result<i32, Failure> {
        self.section_law_base().indices.find(an_edge).copied()
    }

    /// `IsConstant()`: the section does not change along the sweep.
    fn is_constant(&self) -> bool;

    /// `IsUClosed()`; `law.is_u_closed()`.
    fn is_u_closed(&self) -> bool {
        self.section_law_base().uclosed
    }

    /// `IsVClosed()`; `law.is_v_closed()`.
    fn is_v_closed(&self) -> bool {
        self.section_law_base().vclosed
    }

    /// `IsDone()`; `law.is_done()`.
    fn is_done(&self) -> bool {
        self.section_law_base().done
    }

    /// `IsVertex()`: the section is a vertex.
    fn is_vertex(&self) -> bool;

    /// `ConcatenedLaw()`: one law for the whole section.
    fn concatened_law(&self) -> Result<Handle<dyn GeomFillSectionLaw>, Failure>;

    /// `Continuity(Index, TolAngular)`: the continuity between the edges `index` and
    /// `index + 1` (the last and the first for index 0 or the last index).
    fn continuity(&self, index: i32, tol_angular: f64) -> Result<Continuity, Failure>;

    /// `VertexTol(Index, Param)`: the gap at a vertex of the section at `param`.
    fn vertex_tol(&self, index: i32, param: f64) -> Result<f64, Failure>;

    /// `Vertex(Index, Param)`: the vertex `index` of the section at `param`.
    fn vertex(&self, index: i32, param: f64) -> Result<Vertex, Failure>;

    /// `D0(U, S)`: the section at `u`.
    fn d0(&self, u: f64) -> Result<Shape, Failure>;

    /// `Init(W)`: starts the walk of `CurrentEdge` on the wire; `law.init(&wire)?`.
    fn init(&self, w: &Wire) -> Result<(), Failure> {
        self.section_law_base().iterator.write().init(w)
    }

    /// `CurrentEdge()`: the next edge of the walk, skipping degenerated edges (a null
    /// edge after the end); `law.current_edge()?`.
    fn current_edge(&self) -> Result<Edge, Failure> {
        let mut iterator = self.section_law_base().iterator.write();
        let mut edge = Edge::default();
        let mut suivant: bool = false;
        if iterator.more() {
            edge = iterator.current().clone();
            suivant = BRepTool::degenerated(&edge)?;
        }
        while suivant {
            iterator.next()?;
            edge = iterator.current().clone();
            suivant = BRepTool::degenerated(&edge)? && iterator.more();
        }
        if iterator.more() {
            iterator.next()?;
        }
        Ok(edge)
    }
}
impl_transient_interface!(
    dyn BRepFillSectionLaw,
    "BRepFill_SectionLaw",
    StandardTransient
);
