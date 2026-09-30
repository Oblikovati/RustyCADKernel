// Rust translation of BRepLib::SameParameter(theEdge, theTolerance, theNewTol,
// IsUseOldEdge) and GetCurve3d (BRepLib.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge `BRepLib::SameParameter`: each curve on surface of the edge checked
//! against its 3d curve, and replaced by a same-parameter approximation when needed.
use super::c0_pcurve::{PCurveState, smooth_c0_pcurve};
use super::copied_edge;
use super::measure::compute_tol;
use crate::brep_check::BRepCheck;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepCurveRepresentation, BRepGCurve, BRepTool};
use rustycad_tkbrep::topods::{Edge, TShape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::approx::SameParameter as ApproxSameParameter;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::toploc::Location;

/// `NCONTROL`: the number of sample intervals of ComputeTol.
const NCONTROL: i32 = 22;

/// An error above which the curve on surface is given up (`BigError`).
const BIG_ERROR: f64 = 1.0e10;

/// The 3d curve of `GetCurve3d`: the first 3d curve representation, its range and
/// location, and the representations of the edge.
struct Curve3d {
    curve: Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
    location: Location,
    list: Vec<Handle<dyn BRepCurveRepresentation>>,
}

/// The static `GetCurve3d`.
fn get_curve3d(edge: &Edge) -> Result<Curve3d, Failure> {
    let list: Vec<_> = edge_tshape(edge, "BRepLib::SameParameter")?
        .curves()
        .to_vec();
    let mut found = Curve3d {
        curve: Handle::default(),
        first: 0.,
        last: 0.,
        location: Location::default(),
        list: Vec::new(),
    };
    for representation in &list {
        let gcurve = as_gcurve(representation);
        if let Some(g) = gcurve.get().filter(|g| g.is_curve3d()) {
            (found.curve, found.first, found.last) = (g.curve3d()?, g.first(), g.last());
            found.location = g.location();
            break;
        }
    }
    found.list = list;
    Ok(found)
}

/// The adaptors shared by the checks of one edge: the 3d curve on its range, the
/// current p-curve and its surface (`HC`, `HC2d`, `HS`).
pub(super) struct Adaptors {
    pub(super) gac: Handle<GeomAdaptorCurve>,
    pub(super) gac2d: Handle<Geom2dAdaptorCurve>,
    pub(super) gas: Handle<GeomAdaptorSurface>,
    pub(super) f3d: f64,
    pub(super) l3d: f64,
}

impl Adaptors {
    /// `ComputeTol(HC, HC2d, HS, NCONTROL)` on the current p-curve.
    pub(super) fn error(&self) -> Result<f64, Failure> {
        compute_tol(&self.hc(), &self.hc2d(), &self.hs(), NCONTROL)
    }

    pub(super) fn hc(&self) -> Handle<dyn Adaptor3dCurve> {
        self.gac.clone().cast()
    }

    pub(super) fn hc2d(&self) -> Handle<dyn Adaptor2dCurve2d> {
        self.gac2d.clone().cast()
    }

    pub(super) fn hs(&self) -> Handle<dyn Adaptor3dSurface> {
        self.gas.clone().cast()
    }

    /// `GAC2d.Load(pc, f3d, l3d)`.
    pub(super) fn load_pcurve(&self, pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.gac2d
            .get()
            .expect("an adaptor")
            .load_range(pcurve, self.f3d, self.l3d)
    }
}

/// The running state of the edge: `IsSameP`, `maxdist` and `YaPCu`.
struct EdgeState {
    is_same_p: bool,
    maxdist: f64,
    ya_pcu: bool,
}

/// `BRepLib::SameParameter(theEdge, theTolerance, theNewTol, IsUseOldEdge)`: nothing
/// (a null edge) for a SameParameter edge or one without 3d curve; else the edge (a
/// copy unless `use_old_edge`) with each curve on surface made to follow the 3d curve
/// on its range (clipped to a non-periodic curve), the range set, SameRange set, and,
/// when every curve follows within the tolerance, SameParameter set with the edge
/// tolerance the deviation reached (written to `new_tol`, at least Confusion);
/// `brep_lib::same_parameter_edge_with_tolerance(&edge, 1e-5, &mut new_tol, true)?`.
pub fn same_parameter_edge_with_tolerance(
    edge: &Edge,
    tolerance: f64,
    new_tol: &mut f64,
    use_old_edge: bool,
) -> Result<Edge, Failure> {
    if BRepTool::same_parameter(edge)? {
        return Ok(Edge::default());
    }
    let mut c3d = get_curve3d(edge)?;
    if c3d.curve.is_null() {
        return Ok(Edge::default());
    }
    let new_edge = if use_old_edge {
        edge.clone()
    } else {
        // will be modified a little bit later, so copy anyway
        let copy = copied_edge(edge)?;
        c3d = get_curve3d(&copy)?;
        copy
    };
    let adaptors = load_curve3d(&c3d)?;
    let prec_c3d: f64 = BRepCheck::prec_curve(adaptors.gac.get().expect("an adaptor"))?;
    let mut state = EdgeState {
        is_same_p: true,
        maxdist: 0.,
        ya_pcu: false,
    };
    let edge_tol: f64 = BRepTool::edge_tolerance(&new_edge)?;
    let same_range: bool = BRepTool::same_range(&new_edge)?;
    let context = EdgeContext {
        tolerance,
        prec_c3d,
        edge_tol,
        same_range,
    };
    for representation in &c3d.list {
        let gcurve = as_gcurve(representation);
        if let Some(g) = gcurve.get().filter(|g| g.is_curve_on_surface()) {
            state.ya_pcu = true;
            check_curve_on_surface(g, &adaptors, &context, &mut state)?;
        }
    }
    finish_edge(&new_edge, &adaptors, &state, new_tol)?;
    Ok(new_edge)
}

/// The range of the 3d curve (clipped to the bounds of a non-periodic curve, unless it
/// trims a periodic one) and the 3d curve moved by its location, loaded in `HC`.
fn load_curve3d(c3d: &Curve3d) -> Result<Adaptors, Failure> {
    let curve = c3d.curve.get().expect("checked non-null");
    // modified by NIZHNY-OCC486  Tue Aug 27 17:15:13 2002 :
    let trimmed_periodical: bool = match c3d.curve.cast::<GeomTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve().get().is_some_and(|b| b.is_periodic()),
        None => false,
    };
    let (mut f3d, mut l3d) = (c3d.first, c3d.last);
    if !curve.is_periodic() && !trimmed_periodical {
        let (udeb, ufin) = (curve.first_parameter(), curve.last_parameter());
        if udeb > f3d {
            f3d = udeb;
        }
        if l3d > ufin {
            l3d = ufin;
        }
    }
    let mut moved = c3d.curve.clone();
    if !c3d.location.is_identity() {
        moved = curve.transformed(c3d.location.transformation())?.cast();
    }
    let gac = GeomAdaptorCurve::default();
    gac.load_range(&moved, f3d, l3d)?;
    Ok(Adaptors {
        gac: Handle::new(gac),
        gac2d: Handle::new(Geom2dAdaptorCurve::default()),
        gas: Handle::new(GeomAdaptorSurface::default()),
        f3d,
        l3d,
    })
}

/// The inputs that stay the same for all curves on surface of the edge.
struct EdgeContext {
    tolerance: f64,
    prec_c3d: f64,
    edge_tol: f64,
    same_range: bool,
}

/// One curve on surface: its surface (moved by its location) loaded in `HS`, then its
/// one or two p-curves checked in turn (stopping at a null one or a failed measure).
fn check_curve_on_surface(
    g: &dyn BRepGCurve,
    adaptors: &Adaptors,
    context: &EdgeContext,
    state: &mut EdgeState,
) -> Result<(), Failure> {
    let pcurves: [Handle<dyn Geom2dCurve>; 2] = [
        g.pcurve()?,
        if g.is_curve_on_closed_surface() {
            g.pcurve2()?
        } else {
            Handle::default()
        },
    ];
    let mut surface: Handle<dyn GeomSurface> = g.surface()?;
    let location = g.location();
    if !location.is_identity() {
        let s = surface.get().ok_or_else(|| {
            crate::brep_lib::errors::null_object("BRepLib::SameParameter: null surface")
        })?;
        surface = s.transformed(location.transformation())?.cast();
    }
    adaptors.gas.get().expect("an adaptor").load(&surface)?;
    // Eval tol2d to compute SameRange
    let tol_same_range: f64 = std_max(
        adaptors
            .gac
            .get()
            .expect("an adaptor")
            .resolution(context.tolerance)?,
        precision::p_confusion(),
    );
    let mut flags = (false, false); // isANA, isBSP
    for (i, pc) in pcurves.iter().enumerate() {
        if pc.is_null() {
            break;
        }
        let pcurve = PCurveInput {
            g,
            index: i,
            pc,
            surface: &surface,
            tol_same_range,
        };
        if !check_pcurve(&pcurve, adaptors, context, state, &mut flags)? {
            break;
        }
    }
    Ok(())
}

/// One p-curve of a curve on surface and what its check needs.
pub(super) struct PCurveInput<'a> {
    pub(super) g: &'a dyn BRepGCurve,
    pub(super) index: usize,
    pub(super) pc: &'a Handle<dyn Geom2dCurve>,
    pub(super) surface: &'a Handle<dyn GeomSurface>,
    pub(super) tol_same_range: f64,
}

/// One p-curve: brought to the 3d range when the edge is not SameRange, measured,
/// smoothed when it is a C0 B-spline, then approximated by Approx_SameParameter;
/// false when the measure fails (the maximal distance is then that error).
fn check_pcurve(
    input: &PCurveInput<'_>,
    adaptors: &Adaptors,
    context: &EdgeContext,
    state: &mut EdgeState,
    flags: &mut (bool, bool),
) -> Result<bool, Failure> {
    let mut pc_state = PCurveState {
        cur_pc: input.pc.clone(),
        updatepc: false,
        error: 0.,
        goodpc: true,
    };
    if !context.same_range {
        geom_lib::same_range(
            input.tol_same_range,
            input.pc,
            input.g.range(),
            (adaptors.f3d, adaptors.l3d),
            &mut pc_state.cur_pc,
        )?;
        pc_state.updatepc = pc_state.cur_pc != *input.pc;
    }
    adaptors.load_pcurve(&pc_state.cur_pc)?;
    pc_state.error = adaptors.error()?;
    if pc_state.error > BIG_ERROR {
        state.maxdist = pc_state.error;
        return Ok(false);
    }
    let gac2d = adaptors.gac2d.get().expect("an adaptor");
    if gac2d.get_type()? == CurveType::BSplineCurve && gac2d.continuity()? == Continuity::C0 {
        smooth_c0_pcurve(input, adaptors, context.tolerance, &mut pc_state, flags)?;
    }
    if pc_state.goodpc {
        approximate(
            input,
            adaptors,
            context.tolerance,
            &mut pc_state,
            state,
            *flags,
        )?;
    } else {
        state.is_same_p = false;
    }
    //  Modified by skv - Thu Jun  3 12:39:19 2004 OCC5898 Begin
    if !state.is_same_p {
        let prec_surf: f64 = BRepCheck::prec_surface(&adaptors.hs())?;
        let cur_tol: f64 = context.edge_tol + std_max(context.prec_c3d, prec_surf);
        if cur_tol >= pc_state.error {
            state.maxdist = std_max(state.maxdist, context.edge_tol);
            state.is_same_p = true;
        }
    }
    Ok(true)
}

/// Approx_SameParameter on the current p-curve: its reached tolerance, or the curve it
/// computes when that does not exceed the measured error; when it fails, the original
/// p-curve brought to the 3d range and the edge not SameParameter.
fn approximate(
    input: &PCurveInput<'_>,
    adaptors: &Adaptors,
    tolerance: f64,
    pc_state: &mut PCurveState,
    state: &mut EdgeState,
    (is_ana, is_bsp): (bool, bool),
) -> Result<(), Failure> {
    let tol: f64 = if is_ana && is_bsp { 1.0e-7 } else { tolerance };
    let same_p =
        ApproxSameParameter::from_adaptors(&adaptors.hc(), &adaptors.hc2d(), &adaptors.hs(), tol)?;
    if same_p.is_same_parameter() {
        state.maxdist = std_max(state.maxdist, same_p.tol_reached());
    } else if same_p.is_done() {
        let tolreached: f64 = same_p.tol_reached();
        if tolreached <= pc_state.error {
            pc_state.cur_pc = same_p.curve_2d();
            pc_state.updatepc = true;
            state.maxdist = std_max(state.maxdist, tolreached);
        } else {
            state.maxdist = std_max(state.maxdist, pc_state.error);
        }
    } else {
        // Approx_SameParameter has failed.
        // Consequently, the situation might be,
        // when 3D and 2D-curve do not have same-range.
        geom_lib::same_range(
            input.tol_same_range,
            input.pc,
            input.g.range(),
            (adaptors.f3d, adaptors.l3d),
            &mut pc_state.cur_pc,
        )?;
        set_pcurve(input, &pc_state.cur_pc)?;
        state.is_same_p = false;
        return Ok(());
    }
    if pc_state.updatepc {
        set_pcurve(input, &pc_state.cur_pc)?;
    }
    Ok(())
}

/// `GCurve->PCurve(curPC)` for the first p-curve, `PCurve2` for the second.
fn set_pcurve(input: &PCurveInput<'_>, pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    if input.index == 0 {
        input.g.set_pcurve(pcurve)
    } else {
        input.g.set_pcurve2(pcurve)
    }
}

/// The end of the edge SameParameter: the 3d range on every representation,
/// SameRange, and, when SameParameter holds, the tolerance reached (with curves on
/// surface) and the flag.
fn finish_edge(
    edge: &Edge,
    adaptors: &Adaptors,
    state: &EdgeState,
    new_tol: &mut f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.range(edge, adaptors.f3d, adaptors.l3d, false)?;
    builder.same_range(edge, true)?;
    if !state.is_same_p {
        return Ok(());
    }
    // Reduce eventually the tolerance of the edge, as
    // all its representations are processed (except for some associated
    // to planes and not stored in the edge !)
    if state.ya_pcu {
        // Avoid setting too small tolerances.
        let maxdist: f64 = std_max(state.maxdist, precision::confusion());
        *new_tol = maxdist;
        let tedge = edge_tshape(edge, "BRepLib::SameParameter")?;
        tedge.set_modified(true);
        tedge.set_tolerance(maxdist);
    }
    builder.same_parameter(edge, true)
}
