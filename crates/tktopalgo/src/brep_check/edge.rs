// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Edge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Edge`: the checker of an edge, its reference curve (`Minimum`), and its
//! flags. The context checks are in `edge_context.rs`, `Tolerance` and
//! `CheckPolygonOnTriangulation` in `edge_tolerance.rs`.
use super::errors::null_object;
use super::result::{BRepCheckResult, ResultBase, SharedStatusList, add_status, close_status_list};
use super::status::BRepCheckStatus;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepGCurve};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};

/// The own fields of `BRepCheck_Edge`.
pub(super) struct EdgeFields {
    /// `myCref`: the reference representation (a 3d curve, else a curve on surface).
    pub(super) cref: Handle<dyn BRepCurveRepresentation>,
    /// `myHCurve`: the adaptor of the reference curve on its range.
    pub(super) hcurve: Handle<dyn Adaptor3dCurve>,
    /// `myGctrl`: the geometric controls are done.
    pub(super) gctrl: bool,
    /// `myIsExactMethod`: the distances are exact (`BRepLib_ValidateEdge`).
    pub(super) is_exact_method: bool,
}

/// `BRepCheck_Edge`: checks the representations of an edge (a unique 3d curve, valid
/// ranges and flags) and, in the context of a face, the distance between its 3d curve
/// and its curves on the face surface; in a solid, that it is shared by two faces.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::topods::Edge;
/// use rustycad_tktopalgo::brep_check::{BRepCheckEdge, BRepCheckResult, BRepCheckStatus};
/// let mut e = Edge::default();
/// BRepBuilder::default().make_edge(&mut e).unwrap();
/// let checker = BRepCheckEdge::new(&e).unwrap();
/// assert_eq!(checker.status().unwrap().first(), Some(&BRepCheckStatus::No3DCurve));
/// ```
pub struct BRepCheckEdge {
    transient: StandardTransient,
    base: ResultBase,
    pub(super) fields: SharedState<EdgeFields>,
}
impl_transient!(BRepCheckEdge, "BRepCheck_Edge", dyn BRepCheckResult, transient,
    interfaces: [dyn BRepCheckResult]);

impl BRepCheckEdge {
    /// `BRepCheck_Edge(E)`: `Init(E)` (so the reference curve is chosen), geometric
    /// controls on, approximate method; `BRepCheckEdge::new(&edge)?`.
    pub fn new(edge: &Edge) -> Result<Self, Failure> {
        let checker = Self {
            transient: StandardTransient::default(),
            base: ResultBase::new(),
            fields: SharedState::new(EdgeFields {
                cref: Handle::default(),
                hcurve: Handle::default(),
                gctrl: true,
                is_exact_method: false,
            }),
        };
        checker.init(edge)?;
        let mut fields = checker.fields.write();
        fields.gctrl = true;
        fields.is_exact_method = false;
        drop(fields);
        Ok(checker)
    }

    /// `GeometricControls()`; `checker.geometric_controls()`.
    pub fn geometric_controls(&self) -> bool {
        self.fields.read().gctrl
    }

    /// `GeometricControls(B)`; `checker.set_geometric_controls(false)`.
    pub fn set_geometric_controls(&self, controls: bool) {
        self.fields.write().gctrl = controls;
    }

    /// `SetExactMethod(theIsExact)`: the exact distance of `BRepLib_ValidateEdge` (only
    /// for a same-parameter edge); `checker.set_exact_method(true)`.
    pub fn set_exact_method(&self, is_exact: bool) {
        self.fields.write().is_exact_method = is_exact;
    }

    /// `IsExactMethod()`; `checker.is_exact_method()`.
    pub fn is_exact_method(&self) -> bool {
        self.fields.read().is_exact_method
    }

    /// `SetStatus(theStatus)`: adds `status` to the statuses of the edge itself;
    /// `checker.set_status(BRepCheckStatus::InvalidPolygonOnTriangulation)?`.
    pub fn set_status(&self, status: BRepCheckStatus) -> Result<(), Failure> {
        add_status(&self.base.own_list()?, status);
        Ok(())
    }

    /// The edge being checked.
    pub(super) fn edge(&self) -> Edge {
        Edge::unchecked(self.base.shape())
    }

    /// The reference choice of `Minimum`: the flags, the 3d curves, and the
    /// representation the adaptor is built on.
    fn choose_reference(&self, list: &SharedStatusList) -> Result<(), Failure> {
        let shape = self.base.shape();
        let tedge = edge_tshape(&shape, "BRepCheck_Edge::Minimum")?;
        let degenerated = tedge.degenerated();
        if !tedge.same_range() && tedge.same_parameter() {
            add_status(list, BRepCheckStatus::InvalidSameParameterFlag);
        }
        let curves = tedge.curves();
        let (mut cref, exist, unique) = first_curve3d(&curves)?;
        if !exist {
            add_status(list, BRepCheckStatus::No3DCurve);
        } else if !unique {
            add_status(list, BRepCheckStatus::Multiple3DCurve);
        }
        if cref.is_null() && !degenerated {
            let found = curves
                .iter()
                .find(|cr| cr.get().is_some_and(|r| r.is_curve_on_surface()));
            if let Some(cr) = found {
                cref = cr.clone();
            }
        } else if cref.is_some() && degenerated {
            add_status(list, BRepCheckStatus::InvalidDegeneratedFlag);
        }
        self.fields.write().cref = cref;
        Ok(())
    }

    /// The adaptor of the reference representation, or `InvalidRange` (and no
    /// reference) when its range is empty or beyond its curve.
    fn build_reference_curve(&self, list: &SharedStatusList) -> Result<(), Failure> {
        let cref = self.fields.read().cref.clone();
        let Some(r) = cref.get() else {
            return Ok(());
        };
        let gcurve = cref.cast::<dyn BRepGCurve>();
        let (first, last) = gcurve
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge::Minimum: myCref is not a BRep_GCurve"))?
            .range();
        let hcurve = if last <= first {
            None
        } else if r.is_curve3d() {
            self.reference_curve3d(r, (first, last))?
        } else {
            reference_curve_on_surface(r, (first, last))?
        };
        match hcurve {
            Some(adaptor) => self.fields.write().hcurve = adaptor,
            None => {
                self.fields.write().cref.nullify();
                add_status(list, BRepCheckStatus::InvalidRange);
            }
        }
        Ok(())
    }

    /// The 3d reference curve moved by the edge location (eap 6 Jun 2002 occ332: the
    /// curve is moved rather than the surface), on its range; `None` for a range beyond
    /// the curve (or its period).
    fn reference_curve3d(
        &self,
        r: &dyn BRepCurveRepresentation,
        (first, last): (f64, f64),
    ) -> Result<Option<Handle<dyn Adaptor3dCurve>>, Failure> {
        let location = (self.base.shape().location() * &r.location())?;
        let transform = location.transformation();
        let source = r.curve3d()?;
        let c3d = source
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge::Minimum: null 3d curve"))?
            .transformed(transform)?
            .cast::<dyn GeomCurve>();
        let curve = c3d
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge::Minimum: transformed 3d curve"))?;
        let bounds = curve_bounds3d(&c3d)?;
        if !range_fits(bounds, (first, last)) {
            return Ok(None);
        }
        // The transformed curve takes the parameters through the same transformation
        // again, as natively.
        let u1 = curve.transformed_parameter(first, transform);
        let u2 = curve.transformed_parameter(last, transform);
        let adaptor = GeomAdaptorCurve::with_range(&c3d, u1, u2)?;
        Ok(Some(Handle::new(adaptor).cast()))
    }
}

/// The bounds of a curve (of its basis when it is trimmed): periodic, period, first,
/// last.
type CurveBounds = (bool, f64, f64, f64);

/// Whether `[first, last]` fits the curve bounds within Precision::PConfusion() (a NaN
/// bound fits, as the native comparisons fail).
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn range_fits((periodic, period, f, l): CurveBounds, (first, last): (f64, f64)) -> bool {
    let eps: f64 = precision::p_confusion();
    if periodic {
        return !(last - first > period + eps);
    }
    !(first < f - eps || last > l + eps)
}

/// The bounds of a 3d curve, those of its basis for a Geom_TrimmedCurve.
fn curve_bounds3d(c3d: &Handle<dyn GeomCurve>) -> Result<CurveBounds, Failure> {
    let curve = c3d
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge: null curve"))?;
    let mut periodic = curve.is_periodic();
    let mut period = if periodic {
        curve.period()?
    } else {
        real_last()
    };
    let (mut f, mut l) = (curve.first_parameter(), curve.last_parameter());
    if let Some(trimmed) = c3d.cast::<GeomTrimmedCurve>().get() {
        let basis = trimmed.basis_curve();
        let b = basis
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge: null basis curve"))?;
        (f, l) = (b.first_parameter(), b.last_parameter());
        periodic = b.is_periodic();
        if periodic {
            period = b.period()?;
        }
    }
    Ok((periodic, period, f, l))
}

/// The bounds of a pcurve, those of its basis for a Geom2d_TrimmedCurve.
pub(super) fn curve_bounds2d(pc: &Handle<dyn Geom2dCurve>) -> Result<CurveBounds, Failure> {
    let curve = pc
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge: null pcurve"))?;
    let mut periodic = curve.is_periodic();
    let mut period = if periodic {
        curve.period()?
    } else {
        real_last()
    };
    let (mut f, mut l) = (curve.first_parameter(), curve.last_parameter());
    if let Some(trimmed) = pc.cast::<Geom2dTrimmedCurve>().get() {
        let basis = trimmed.basis_curve();
        let b = basis
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge: null basis pcurve"))?;
        (f, l) = (b.first_parameter(), b.last_parameter());
        periodic = b.is_periodic();
        if periodic {
            period = b.period()?;
        }
    }
    Ok((periodic, period, f, l))
}

/// Whether `[first, last]` fits the pcurve bounds (the InContext range check).
pub(super) fn pcurve_range_fits(
    pc: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
) -> Result<bool, Failure> {
    Ok(range_fits(curve_bounds2d(pc)?, range))
}

/// The curve on surface reference: the pcurve on the surface moved by the
/// representation location only (not the edge location, as natively).
fn reference_curve_on_surface(
    r: &dyn BRepCurveRepresentation,
    (first, last): (f64, f64),
) -> Result<Option<Handle<dyn Adaptor3dCurve>>, Failure> {
    let source = r.surface()?;
    let sref = source
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge::Minimum: null surface"))?
        .transformed(r.location().transformation())?
        .cast::<dyn GeomSurface>();
    let pcref = r.pcurve()?;
    if !range_fits(curve_bounds2d(&pcref)?, (first, last)) {
        return Ok(None);
    }
    Ok(Some(curve_on_surface(&pcref, (first, last), &sref)?.cast()))
}

/// `new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(pc, f, l), new
/// GeomAdaptor_Surface(S))`.
pub(super) fn curve_on_surface(
    pc: &Handle<dyn Geom2dCurve>,
    (first, last): (f64, f64),
    surface: &Handle<dyn GeomSurface>,
) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
    let curve: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(pc, first, last)?).cast();
    let adapted: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surface)?).cast();
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(
        &curve, &adapted,
    )?))
}

/// The first 3d curve representation with a curve, whether a 3d curve representation
/// exists, and whether it is unique.
fn first_curve3d(
    curves: &[Handle<dyn BRepCurveRepresentation>],
) -> Result<(Handle<dyn BRepCurveRepresentation>, bool, bool), Failure> {
    let mut cref: Handle<dyn BRepCurveRepresentation> = Handle::default();
    let (mut exist, mut unique) = (false, true);
    for cr in curves {
        let Some(r) = cr.get().filter(|r| r.is_curve3d()) else {
            continue;
        };
        if exist {
            unique = false;
        }
        exist = true;
        if cref.is_null() && r.curve3d()?.is_some() {
            cref = cr.clone();
        }
    }
    Ok((cref, exist, unique))
}

impl BRepCheckResult for BRepCheckEdge {
    fn result_base(&self) -> &ResultBase {
        &self.base
    }

    /// `InContext(S)`: see `edge_context.rs`.
    fn in_context(&self, context: &Shape) -> Result<(), Failure> {
        self.check_in_context(context)
    }

    /// `Minimum()`: the flags, a unique 3d curve (`No3DCurve`, `Multiple3DCurve`), the
    /// degenerated flag and the range of the reference representation, whose adaptor
    /// is built for the context checks.
    fn minimum(&self) -> Result<(), Failure> {
        if self.base.is_minimum() {
            return Ok(());
        }
        let list = self.base.bind_own_list();
        self.fields.write().cref.nullify();
        self.choose_reference(&list)?;
        self.build_reference_curve(&list)?;
        close_status_list(&list);
        self.base.set_minimum(true);
        Ok(())
    }

    /// `Blind()`: marks the edge checked (the body was removed natively, skv 2004).
    fn blind(&self) {
        self.base.set_blind(true);
    }
}
