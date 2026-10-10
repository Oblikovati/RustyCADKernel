// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_ACRLaw.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_ACRLaw`: a guide location law along the edges of a path, each copy set
//! to the reduced curvilinear abscissas of its edge.
use super::{BRepFillLocationLaw, BRepFillLocationLawBase, oriented_edge_curve};
use crate::brep_fill::package::compute_acr;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::Wire;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_fill::{GeomFillLocationGuide, GeomFillLocationLaw};

/// The guide location law of a path (`BRepFill_ACRLaw`): the guide law's origin is set
/// to the abscissas of each edge before it is copied onto the edge (the input law keeps
/// the abscissas of the last edge, as natively); `let law = BRepFillACRLaw::new(&path, &guide)?;`.
pub struct BRepFillACRLaw {
    parent: StandardTransient,
    base: BRepFillLocationLawBase,
    /// `OrigParam`: the reduced abscissas of the edge ends, indices 0..=NbEdge.
    orig_param: Vec<f64>,
}
impl_transient!(
    BRepFillACRLaw,
    "BRepFill_ACRLaw",
    dyn BRepFillLocationLaw,
    parent,
    interfaces: [dyn BRepFillLocationLaw]
);

/// The number of edges met by the wire explorer (degenerated ones included).
fn explored_edges(path: &Wire) -> Result<usize, Failure> {
    let mut count: usize = 0;
    let mut explorer = WireExplorer::with_wire(path)?;
    while explorer.more() {
        count += 1;
        explorer.next()?;
    }
    Ok(count)
}

/// Standard_OutOfRange for an abscissa table access.
fn acr_error(error: impl std::fmt::Display) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill_ACRLaw: {error}; expected an abscissa of the path"),
        "BRepFill_ACRLaw",
    )
}

impl BRepFillACRLaw {
    /// `BRepFill_ACRLaw(Path, Law)`.
    pub fn new(path: &Wire, the_law: &Handle<GeomFillLocationGuide>) -> Result<Self, Failure> {
        let base = BRepFillLocationLawBase::new(path)?;
        let nb_edge: usize = explored_edges(path)?;
        let mut orig: Array1<'static, f64> =
            Array1::with_bounds(0, nb_edge as i32).map_err(acr_error)?;
        compute_acr(path, &mut orig)?;
        let mut orig_param: Vec<f64> = orig.as_slice().to_vec();
        orig_param[0] = 0.;
        let mut law = Self {
            parent: StandardTransient::default(),
            base,
            orig_param,
        };
        law.place_guides(the_law)?;
        Ok(law)
    }

    /// One copy of the guide law per non-degenerated edge, its origin set to the
    /// abscissas `OrigParam(ipath - 1)` and `OrigParam(ipath)`.
    fn place_guides(&mut self, the_law: &Handle<GeomFillLocationGuide>) -> Result<(), Failure> {
        let Some(guide) = the_law.get() else {
            return Err(Failure::with_type(
                "Standard_NullObject",
                "BRepFill_ACRLaw: null guide law; expected a law",
                "BRepFill_ACRLaw",
            ));
        };
        let mut ipath: usize = 0;
        let mut explorer = WireExplorer::with_wire(&self.base.path)?;
        while explorer.more() {
            let edge = explorer.current().clone();
            explorer.next()?;
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            ipath += 1;
            self.base.edges[ipath - 1] = edge.as_ref().clone();
            let (curve, first, last) = oriented_edge_curve(&edge)?;
            let ac: Handle<dyn Adaptor3dCurve> =
                Handle::new(GeomAdaptorCurve::with_range(&curve, first, last)?).cast();
            // The parameters for the case multi-edges.
            let t1: f64 = *self
                .orig_param
                .get(ipath - 1)
                .ok_or_else(|| acr_error(ipath - 1))?;
            let t2: f64 = *self.orig_param.get(ipath).ok_or_else(|| acr_error(ipath))?;
            guide.set_origine(t1, t2);
            let copy: Handle<dyn GeomFillLocationLaw> = guide.copy()?;
            if let Some(laws) = self.base.laws.as_mut() {
                laws[ipath - 1] = copy.clone();
            }
            if let Some(copy) = copy.get() {
                copy.set_curve(&ac)?;
            }
        }
        Ok(())
    }
}

impl BRepFillLocationLaw for BRepFillACRLaw {
    fn location_law_base(&self) -> &BRepFillLocationLawBase {
        &self.base
    }
}
