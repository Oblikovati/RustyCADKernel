// Rust translation of TopOpeBRepTool_CurveTool::MakeCurves and of the static
// CheckApproxResults, MakeCurve3DfromWLineApprox, MakeCurve2DfromWLineApproxAndPlane and
// MakeCurve2DfromWLineApprox (ModelingAlgorithms/TKBool/TopOpeBRepTool/
// TopOpeBRepTool_CurveTool.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The approximation of the curves of an intersection line (3d curve and p-curves on
//! both faces, B-splines of degree 1 through the walked points) by smooth B-splines.
use super::curvature::is_bad;
use super::improve::{Poles, improve};
use super::pcurve::check_pcurve;
use super::{TopOpeBRepToolCurveTool, array_failure};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkgeomalgo::approx_int::ApproxIntRange;
use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurveEval};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkgeombase::geom_lib::{Check2dBSplineCurve, CheckBSplineCurve};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Plane3, Point2, Point3};
use rustycad_tktopalgo::brep_approx::{BRepApproxApprox, BRepApproxApproxLine};

/// The tolerance of the tangent check of the made B-splines.
const TOLCHECK: f64 = 1.0e-7;
/// The angular tolerance of the tangent check of the made B-splines.
const TOLANGCHECK: f64 = 1.0e-6;

/// The 3d curve and the p-curves on both faces of a section edge.
#[derive(Clone, Default)]
pub struct SectionCurves {
    /// The 3d curve.
    pub c3d: Handle<dyn GeomCurve>,
    /// The p-curve on the first face.
    pub pc1: Handle<dyn Geom2dCurve>,
    /// The p-curve on the second face.
    pub pc2: Handle<dyn Geom2dCurve>,
}

/// The output of a done `MakeCurves`: the approximated curves (null when not asked)
/// and the reached tolerances.
#[derive(Clone, Default)]
pub struct MadeCurves {
    /// `C3Dnew`, `PC1new`, `PC2new`.
    pub curves: SectionCurves,
    /// `TolReached3d`.
    pub tol_reached3d: f64,
    /// `TolReached2d`.
    pub tol_reached2d: f64,
}

/// The approximation options: tolerances, point count, maximal degree and
/// parametrization.
struct ApproxOptions {
    tol3d: f64,
    tol2d: f64,
    nb_pnt_max: i32,
    degmax: i32,
    parametrization: ParametrizationType,
}

/// Which faces are planes with their p-curves asked (the plane p-curve is then made
/// from the 3d poles); the first one wins.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PlaneCase {
    First,
    Second,
    None,
}

/// Standard_NullObject of a curve that is not a B-spline (natively a null dereference).
fn not_bspline(name: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!(
            "TopOpeBRepTool_CurveTool::MakeCurves: {name} is null or not a B-spline; expected a B-spline curve"
        ),
        "TopOpeBRepTool_CurveTool",
    )
}

/// The poles of the asked input curves (`HC3D`, `HPC1`, `HPC2` before the improvement).
fn input_poles(
    hc3d: &Handle<GeomBSplineCurve>,
    hpc: [(&Handle<Geom2dBSplineCurve>, bool); 2],
) -> Result<Poles, Failure> {
    let c3d: Vec<Point3> = hc3d
        .get()
        .ok_or_else(|| not_bspline("C3D"))?
        .poles_array(|p| p.iter().copied().collect());
    let pc = |(h, comp): (&Handle<Geom2dBSplineCurve>, bool),
              name: &str|
     -> Result<Option<Vec<Point2>>, Failure> {
        if !comp {
            return Ok(None);
        }
        Ok(Some(
            h.get()
                .ok_or_else(|| not_bspline(name))?
                .poles()
                .iter()
                .copied()
                .collect(),
        ))
    };
    Ok(Poles {
        c3d,
        pc1: pc(hpc[0], "PC1")?,
        pc2: pc(hpc[1], "PC2")?,
    })
}

impl ApproxOptions {
    /// The options of the tool, tightened for bad poles (`IsBad`): the 3d tolerance at
    /// most Precision::Approximation(), 40 points, degree 4 and centripetal.
    fn of(tool: &TopOpeBRepToolCurveTool, bad: bool) -> Self {
        let (tol3d, tol2d) = tool.my_geom_tool.get_tolerances();
        let options = Self {
            tol3d,
            tol2d,
            nb_pnt_max: tool.my_geom_tool.nb_pnt_max(),
            degmax: 8,
            parametrization: ParametrizationType::ChordLength,
        };
        if !bad {
            return options;
        }
        let tt: f64 = (10. * tol3d).min(precision::approximation());
        Self {
            tol3d: tt,
            tol2d: tt * tol2d / tol3d,
            nb_pnt_max: 40,
            degmax: 4,
            parametrization: ParametrizationType::Centripetal,
        }
    }
}

/// `CheckApproxResults(Approx)`: at least 2 poles and 1 curve, and distinct knots.
fn check_approx_results(amc: &MultiBSpCurve) -> Result<bool, Failure> {
    if amc.nb_poles() < 2 || amc.nb_curves()? < 1 {
        return Ok(false);
    }
    let knots = amc.knots()?;
    for i in knots.lower()..knots.upper() {
        if knots[i + 1] - knots[i] <= epsilon(knots[i]) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `MakeCurve3DfromWLineApprox(Approx, 1)`: the 3d B-spline of the first curve.
fn make_curve3d(amc: &MultiBSpCurve) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut poles3d: Array1<'static, Point3> =
        Array1::with_bounds(1, amc.nb_poles()).map_err(array_failure)?;
    amc.curve_3d(1, &mut poles3d)?;
    Ok(Handle::new(GeomBSplineCurve::new(
        &poles3d,
        &amc.knots()?,
        &amc.multiplicities()?,
        amc.degree()?,
        false,
    )?)
    .cast())
}

/// `MakeCurve2DfromWLineApproxAndPlane(Approx, Pl)`: the 2d B-spline of the plane
/// parameters of the 3d poles.
fn make_curve2d_on_plane(
    amc: &MultiBSpCurve,
    pl: &Plane3,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let np = amc.nb_poles();
    let mut poles3d: Array1<'static, Point3> = Array1::with_bounds(1, np).map_err(array_failure)?;
    let mut poles2d: Array1<'static, Point2> = Array1::with_bounds(1, np).map_err(array_failure)?;
    amc.curve_3d(1, &mut poles3d)?;
    for i in 1..=np {
        let (u, v) = elslib::plane_parameters(pl.position(), poles3d[i]);
        poles2d[i] = Point2::new(u, v);
    }
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles2d,
        &amc.knots()?,
        &amc.multiplicities()?,
        amc.degree()?,
        false,
    )?)
    .cast())
}

/// `MakeCurve2DfromWLineApprox(Approx, CI)`: the 2d B-spline of the curve `ci` (the
/// poles left at the origin when there is no such curve, as natively).
fn make_curve2d(amc: &MultiBSpCurve, ci: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut poles2d: Array1<'static, Point2> =
        Array1::with_bounds(1, amc.nb_poles()).map_err(array_failure)?;
    if (1..=amc.nb_curves()?).contains(&ci) {
        amc.curve_2d(ci, &mut poles2d)?;
    }
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles2d,
        &amc.knots()?,
        &amc.multiplicities()?,
        amc.degree()?,
        false,
    )?)
    .cast())
}

/// The approximated curves of the done approximation, by plane case.
fn approximated_curves(
    amc: &MultiBSpCurve,
    case: PlaneCase,
    (bas1, bas2): (&BRepAdaptorSurface, &BRepAdaptorSurface),
    (comp_pc1, comp_pc2): (bool, bool),
) -> Result<SectionCurves, Failure> {
    let mut out = SectionCurves {
        c3d: make_curve3d(amc)?,
        ..SectionCurves::default()
    };
    match case {
        PlaneCase::First => {
            out.pc1 = make_curve2d_on_plane(amc, &bas1.plane()?)?;
            if comp_pc2 {
                out.pc2 = make_curve2d(amc, 2)?;
            }
        }
        PlaneCase::Second => {
            if comp_pc1 {
                out.pc1 = make_curve2d(amc, 2)?;
            }
            out.pc2 = make_curve2d_on_plane(amc, &bas2.plane()?)?;
        }
        PlaneCase::None => {
            if comp_pc1 {
                out.pc1 = make_curve2d(amc, 2)?;
            }
            if comp_pc2 {
                out.pc2 = make_curve2d(amc, if comp_pc1 { 3 } else { 2 })?;
            }
        }
    }
    Ok(out)
}

/// The tangent fix of the ends of the made B-splines (`GeomLib_CheckBSplineCurve` and
/// `GeomLib_Check2dBSplineCurve` with TOLCHECK and TOLANGCHECK), in place.
fn fix_tangents(curves: &SectionCurves) -> Result<(), Failure> {
    let curve: Handle<GeomBSplineCurve> = curves.c3d.clone().cast();
    if !curve.is_null() {
        let mut cbsc = CheckBSplineCurve::new(&curve, TOLCHECK, TOLANGCHECK)?;
        let (bf, bl) = cbsc.need_tangent_fix();
        cbsc.fix_tangent(bf, bl)?;
    }
    for pc in [&curves.pc1, &curves.pc2] {
        let curve2d: Handle<Geom2dBSplineCurve> = pc.clone().cast();
        if curve2d.is_null() {
            continue;
        }
        let mut cbsc2d = Check2dBSplineCurve::new(&curve2d, TOLCHECK, TOLANGCHECK)?;
        let (bf, bl) = cbsc2d.need_tangent_fix();
        cbsc2d.fix_tangent(bf, bl)?;
    }
    Ok(())
}

impl TopOpeBRepToolCurveTool {
    /// `MakeCurves(parmin, parmax, C3D, PC1, PC2, S1, S2, C3Dnew, PC1new, PC2new,
    /// TolReached3d, TolReached2d)`: the approximation of the B-splines of degree 1 of
    /// an intersection line between the indices `parmin`..`parmax` (truncated) on the
    /// faces `s1`, `s2`; `None` when the 3d curve is not asked or the approximation
    /// fails (the native outputs are then left, or nullified). Natively the input curves
    /// of more than 100 poles are replaced by reduced ones (a const_cast kept here as
    /// `&mut`), while the approximation still reads the original poles up to the reduced
    /// count; `let made = ct.make_curves(1., 10., &mut curves, &f1, &f2)?;`.
    pub fn make_curves(
        &self,
        parmin: f64,
        parmax: f64,
        input: &mut SectionCurves,
        s1: &Shape,
        s2: &Shape,
    ) -> Result<Option<MadeCurves>, Failure> {
        let gt = &self.my_geom_tool;
        if !gt.comp_c3d() {
            return Ok(None);
        }
        let comps = (gt.comp_pc1(), gt.comp_pc2());
        let iparmin = parmin as i32;
        let hc3d: Handle<GeomBSplineCurve> = input.c3d.clone().cast();
        let hpc1: Handle<Geom2dBSplineCurve> = input.pc1.clone().cast();
        let hpc2: Handle<Geom2dBSplineCurve> = input.pc2.clone().cast();
        let poles = input_poles(&hc3d, [(&hpc1, comps.0), (&hpc2, comps.1)])?;
        let improved = improve(&poles, gt.get_tolerances(), input)?;
        let iparmax: i32 = improved.unwrap_or(parmax as i32);
        let options = ApproxOptions::of(self, is_bad(&poles.c3d)?);
        let (f1, f2) = (topods::face(s1)?, topods::face(s2)?);
        let bas1: Handle<BRepAdaptorSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f1, true)?);
        let bas2: Handle<BRepAdaptorSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f2, true)?);
        let case = plane_case(adaptor(&bas1)?, adaptor(&bas2)?, comps)?;
        let approx = run_approx(
            &options,
            (&bas1, &bas2),
            BRepApproxApproxLine::from_curves(&hc3d, &hpc1, &hpc2),
            approx_range(case, comps, (iparmin, iparmax)),
        )?;
        let Some(curves) = checked_curves(
            &approx,
            case,
            (adaptor(&bas1)?, adaptor(&bas2)?),
            comps,
            (s1, s2),
        )?
        else {
            return Ok(None);
        };
        let made = MadeCurves {
            curves,
            tol_reached3d: approx.tol_reached3d(),
            tol_reached2d: approx.tol_reached2d(),
        };
        fix_tangents(&made.curves)?;
        Ok(Some(made))
    }
}

/// The adaptor of a handle made just before (never null).
fn adaptor(h: &Handle<BRepAdaptorSurface>) -> Result<&BRepAdaptorSurface, Failure> {
    h.get().ok_or_else(|| Failure::with_type("Standard_NullObject", "TopOpeBRepTool_CurveTool::MakeCurves: null BRepAdaptor_Surface; expected the adaptor of a face", "TopOpeBRepTool_CurveTool"))
}

/// The plane case of the faces (the plane p-curve made from the 3d poles).
fn plane_case(
    bas1: &BRepAdaptorSurface,
    bas2: &BRepAdaptorSurface,
    (comp_pc1, comp_pc2): (bool, bool),
) -> Result<PlaneCase, Failure> {
    if comp_pc1 && bas1.get_type()? == SurfaceType::Plane {
        return Ok(PlaneCase::First);
    }
    if comp_pc2 && bas2.get_type()? == SurfaceType::Plane {
        return Ok(PlaneCase::Second);
    }
    Ok(PlaneCase::None)
}

/// The approximated parts: XYZ always, the p-curve on a plane face never.
fn approx_range(
    case: PlaneCase,
    (comp_pc1, comp_pc2): (bool, bool),
    (indicemin, indicemax): (i32, i32),
) -> ApproxIntRange {
    ApproxIntRange {
        approx_xyz: true,
        approx_u1v1: comp_pc1 && case != PlaneCase::First,
        approx_u2v2: comp_pc2 && case != PlaneCase::Second,
        indicemin,
        indicemax,
    }
}

/// The approximation of the line on the faces (degree 4 at least, projection only,
/// with tangency).
fn run_approx(
    options: &ApproxOptions,
    (bas1, bas2): (&Handle<BRepAdaptorSurface>, &Handle<BRepAdaptorSurface>),
    line: BRepApproxApproxLine,
    range: ApproxIntRange,
) -> Result<BRepApproxApprox, Failure> {
    let mut approx = BRepApproxApprox::new();
    approx.set_parameters(
        options.tol3d,
        options.tol2d,
        4,
        options.degmax,
        0,
        options.nb_pnt_max,
        true,
        options.parametrization,
    );
    approx.perform_on_surfaces(bas1, bas2, &Handle::new(line), range)?;
    Ok(approx)
}

/// The curves of a done and valid approximation whose p-curves are within the face
/// bounds; `None` otherwise.
fn checked_curves(
    approx: &BRepApproxApprox,
    case: PlaneCase,
    bas: (&BRepAdaptorSurface, &BRepAdaptorSurface),
    comps: (bool, bool),
    (s1, s2): (&Shape, &Shape),
) -> Result<Option<SectionCurves>, Failure> {
    if !approx.is_done() || !check_approx_results(approx.value(1))? {
        return Ok(None);
    }
    let curves = approximated_curves(approx.value(1), case, bas, comps)?;
    if comps.0 && !check_pcurve(&curves.pc1, &topods::face(s1)?)? {
        return Ok(None);
    }
    if comps.1 && !check_pcurve(&curves.pc2, &topods::face(s2)?)? {
        return Ok(None);
    }
    Ok(Some(curves))
}
