// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_LocationLaw.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_LocationLaw`: the placement of the sections along the edges of a path,
//! one GeomFill location law per non-degenerated edge.
mod acr_law;
mod continuity;
mod edge3d_law;
mod on_surf_law;
mod positions;

pub use acr_law::BRepFillACRLaw;
pub use edge3d_law::{BRepFillDraftLaw, BRepFillEdge3DLaw};
pub use on_surf_law::BRepFillEdgeOnSurfLaw;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex, Wire};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::{GeomFillLocationLaw, GeomFillPipeError};

/// The protected data of `BRepFill_LocationLaw` (`myPath`, `myTol`, `myLaws`,
/// `myLength`, `myEdges`, `myDisc`) and its private `myType`, embedded in each concrete
/// law and returned by [`BRepFillLocationLaw::location_law_base`].
pub struct BRepFillLocationLawBase {
    pub(crate) path: Wire,
    pub(crate) tol: f64,
    /// `myLaws`: `None` when nullified (BRepFill_EdgeOnSurfLaw without a result).
    pub(crate) laws: Option<Vec<Handle<dyn GeomFillLocationLaw>>>,
    /// `myLength`, indices `1..=NbEdge + 1` at `0..=NbEdge`, -1 while not computed.
    pub(crate) length: SharedState<Vec<f64>>,
    pub(crate) edges: Vec<Shape>,
    /// `myDisc`: the holes found by `NbHoles`, `None` while not searched or none.
    pub(crate) disc: SharedState<Option<Vec<i32>>>,
    my_type: i32,
}

impl BRepFillLocationLawBase {
    /// `Init(Path)`: one null law, one null edge and one unknown length per
    /// non-degenerated edge of the path, the tolerance 1e-4 and the tangent mode.
    pub(crate) fn new(path: &Wire) -> Result<Self, Failure> {
        let mut nb_edge: usize = 0;
        let mut explorer = WireExplorer::with_wire(path)?;
        while explorer.more() {
            if !BRepTool::degenerated(explorer.current())? {
                nb_edge += 1;
            }
            explorer.next()?;
        }
        let mut length: Vec<f64> = vec![-1.; nb_edge + 1];
        length[0] = 0.;
        Ok(Self {
            path: path.clone(),
            tol: 1.0e-4,
            laws: Some(vec![Handle::default(); nb_edge]),
            length: SharedState::new(length),
            edges: vec![Shape::default(); nb_edge],
            disc: SharedState::new(None),
            // TangentIsMain.
            my_type: 1,
        })
    }

    /// The laws; Standard_NullObject when nullified.
    pub(crate) fn built_laws(&self) -> Result<&[Handle<dyn GeomFillLocationLaw>], Failure> {
        self.laws.as_deref().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_LocationLaw: the laws are null; expected a law with a result",
                "BRepFill_LocationLaw",
            )
        })
    }

    /// The law `index` (1-based) as a live reference.
    pub(crate) fn law_at(&self, index: i32) -> Result<&dyn GeomFillLocationLaw, Failure> {
        let laws = self.built_laws()?;
        let law = usize::try_from(index - 1).ok().and_then(|i| laws.get(i));
        let law = law.ok_or_else(|| index_error("BRepFill_LocationLaw::Law", index, laws.len()))?;
        law.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_LocationLaw: a null law; expected a built law",
                "BRepFill_LocationLaw",
            )
        })
    }

    /// The edge `index` (1-based).
    pub(crate) fn edge_at(&self, index: i32) -> Result<Edge, Failure> {
        let found = usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.edges.get(i));
        topods::edge(
            found.ok_or_else(|| {
                index_error("BRepFill_LocationLaw::Edge", index, self.edges.len())
            })?,
        )
    }

    /// The number of laws.
    pub(crate) fn count(&self) -> Result<i32, Failure> {
        Ok(self.built_laws()?.len() as i32)
    }

    /// The protected `TangentIsMain()`: the tangent is the main direction (`myType`
    /// 1); `base.tangent_is_main()`.
    pub fn tangent_is_main(&mut self) {
        self.my_type = 1;
    }

    /// The protected `NormalIsMain()` (`myType` 2); `base.normal_is_main()`.
    pub fn normal_is_main(&mut self) {
        self.my_type = 2;
    }

    /// The protected `BiNormalIsMain()` (`myType` 3); `base.binormal_is_main()`.
    pub fn binormal_is_main(&mut self) {
        self.my_type = 3;
    }

    /// `myType` (1 tangent, 2 normal, 3 binormal is main; never read natively).
    pub fn main_direction(&self) -> i32 {
        self.my_type
    }
}

/// Standard_OutOfRange for an index outside `1..=count`.
pub(crate) fn index_error(member: &str, index: i32, count: usize) -> Failure {
    let message = format!("{member}: index {index}; expected 1..={count}");
    Failure::with_type("Standard_OutOfRange", &message, "BRepFill_LocationLaw")
}

/// The curve of an edge on its range, as a trimmed curve reversed for a REVERSED edge,
/// with its new range (the start of the subclass constructors).
pub(crate) fn oriented_edge_curve(
    edge: &Edge,
) -> Result<(Handle<dyn rustycad_tkg3d::geom::GeomCurve>, f64, f64), Failure> {
    use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
    let (curve, first, last) = BRepTool::curve(edge)?;
    if edge.orientation() != Orientation::Reversed {
        return Ok((curve, first, last));
    }
    let trimmed = GeomTrimmedCurve::new(&curve, first, last, true, true)?;
    // Reversed to avoid damaging the topology.
    trimmed.reverse()?;
    let (f, l) = (trimmed.first_parameter(), trimmed.last_parameter());
    Ok((Handle::new(trimmed).cast::<dyn GeomCurve>(), f, l))
}

/// `BRepFill_LocationLaw`: the abstract location law of a sweep, reached through
/// `Handle<dyn BRepFillLocationLaw>`; the concrete laws are BRepFill_Edge3DLaw,
/// BRepFill_DraftLaw, BRepFill_EdgeOnSurfLaw and BRepFill_ACRLaw. Every member takes
/// `&self`: the lengths and holes are computed on demand behind locks.
/// `let n = law.nb_law()?;`
pub trait BRepFillLocationLaw: Transient {
    /// The embedded protected data.
    fn location_law_base(&self) -> &BRepFillLocationLawBase;

    /// `GetStatus()`: the first error status of the laws.
    fn get_status(&self) -> Result<GeomFillPipeError, Failure> {
        let base = self.location_law_base();
        let mut status = GeomFillPipeError::PipeOk;
        for ii in 1..=base.count()? {
            if status != GeomFillPipeError::PipeOk {
                break;
            }
            status = base.law_at(ii)?.error_status();
        }
        Ok(status)
    }

    /// `TransformInG0Law()`: each law placed to continue the previous one (G0).
    fn transform_in_g0_law(&self) -> Result<(), Failure> {
        continuity::transform_in_g0_law(self.location_law_base())
    }

    /// `TransformInCompatibleLaw(AngularTolerance)`: each law turned about its
    /// tangent to continue the normal of the previous one.
    fn transform_in_compatible_law(&self, angular_tolerance: f64) -> Result<(), Failure> {
        continuity::transform_in_compatible_law(self.location_law_base(), angular_tolerance)
    }

    /// `DeleteTransform()`: the identity placement on every law, the holes forgotten.
    fn delete_transform(&self) -> Result<(), Failure> {
        continuity::delete_transform(self.location_law_base())
    }

    /// `NbHoles(Tol)`: the vertices of the path where the law is not G0.
    fn nb_holes(&self, tol: f64) -> Result<i32, Failure> {
        continuity::nb_holes(self.location_law_base(), tol)
    }

    /// `Holes(Interval)`: the indices of the holes found by `NbHoles`, written from 1.
    fn holes(&self, interval: &mut [i32]) {
        if let Some(disc) = self.location_law_base().disc.read().as_ref() {
            for (slot, hole) in interval.iter_mut().zip(disc) {
                *slot = *hole;
            }
        }
    }

    /// `NbLaw()`; `law.nb_law()?`.
    fn nb_law(&self) -> Result<i32, Failure> {
        self.location_law_base().count()
    }

    /// `Law(Index)`: the GeomFill law of edge `index`.
    fn law(&self, index: i32) -> Result<Handle<dyn GeomFillLocationLaw>, Failure> {
        let laws = self.location_law_base().built_laws()?;
        let found = usize::try_from(index - 1).ok().and_then(|i| laws.get(i));
        found
            .cloned()
            .ok_or_else(|| index_error("BRepFill_LocationLaw::Law", index, laws.len()))
    }

    /// `Wire()`: the path; `law.wire()`.
    fn wire(&self) -> &Wire {
        &self.location_law_base().path
    }

    /// `Edge(Index)`: the edge of law `index`.
    fn edge(&self, index: i32) -> Result<Edge, Failure> {
        self.location_law_base().edge_at(index)
    }

    /// `Vertex(Index)`: the first vertex of edge `index` in its orientation, the last
    /// vertex of the last edge for `count + 1`, a null vertex beyond.
    fn vertex(&self, index: i32) -> Result<Vertex, Failure> {
        let base = self.location_law_base();
        let n: i32 = base.edges.len() as i32;
        let reversed = |e: &Edge| e.orientation() == Orientation::Reversed;
        if index <= n {
            let e: Edge = base.edge_at(index)?;
            return Ok(if reversed(&e) {
                topexp::last_vertex(&e, false)
            } else {
                topexp::first_vertex(&e, false)
            });
        }
        if index == n + 1 {
            let e: Edge = base.edge_at(index - 1)?;
            return Ok(if reversed(&e) {
                topexp::first_vertex(&e, false)
            } else {
                topexp::last_vertex(&e, false)
            });
        }
        Ok(Vertex::default())
    }

    /// `PerformVertex(Index, InputVertex, TolMin, OutputVertex, Location)`: the vertex
    /// of the sweep from a vertex of the section placed at the vertex `index` of the
    /// path (the middle of both placements at a G1 junction with `location` 0).
    fn perform_vertex(
        &self,
        index: i32,
        input: &Vertex,
        tol_min: f64,
        location: i32,
    ) -> Result<Vertex, Failure> {
        positions::perform_vertex(self.location_law_base(), index, input, tol_min, location)
    }

    /// `CurvilinearBounds(Index, First, Last)`: the abscissas of edge `index` on the path.
    fn curvilinear_bounds(&self, index: i32) -> Result<(f64, f64), Failure> {
        positions::curvilinear_bounds(self.location_law_base(), index)
    }

    /// `IsClosed()`: the path is closed (flag or same end vertices).
    fn is_closed(&self) -> Result<bool, Failure> {
        let path = &self.location_law_base().path;
        if path.closed()? {
            return Ok(true);
        }
        let (v1, v2) = topexp::wire_vertices(path);
        Ok(v1.is_same(&v2))
    }

    /// `IsG1(Index, SpatialTolerance, AngularTolerance)`: -1 not G0, 0 G0 (or no
    /// derivative), 1 G1 at the vertex `index` of the path.
    fn is_g1(
        &self,
        index: i32,
        spatial_tolerance: f64,
        angular_tolerance: f64,
    ) -> Result<i32, Failure> {
        continuity::is_g1(
            self.location_law_base(),
            index,
            spatial_tolerance,
            angular_tolerance,
        )
    }

    /// `D0(Abscissa, Section)`: the section moved (as a copy) to the placement at the
    /// abscissa, a null shape outside the path.
    fn d0(&self, abscissa: f64, section: &Shape) -> Result<Shape, Failure> {
        positions::d0(self.location_law_base(), abscissa, section)
    }

    /// `Parameter(Abscissa, Index, Param)`: the edge and parameter at an abscissa
    /// (index 0 outside the path).
    fn parameter(&self, abscissa: f64) -> Result<(i32, f64), Failure> {
        positions::parameter(self.location_law_base(), abscissa)
    }

    /// `Abscissa(Index, Param)`: the abscissa on the path of a parameter of edge `index`.
    fn abscissa(&self, index: i32, param: f64) -> Result<f64, Failure> {
        positions::abscissa(self.location_law_base(), index, param)
    }
}
impl_transient_interface!(
    dyn BRepFillLocationLaw,
    "BRepFill_LocationLaw",
    StandardTransient
);
