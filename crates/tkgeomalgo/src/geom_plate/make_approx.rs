// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_MakeApprox.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_MakeApprox`: the B-spline approximation of a plate surface.
use super::live;
use super::make_approx_eval::MakeApproxEval;
use super::plate_criteria::{GeomPlatePlateG0Criterion, GeomPlatePlateG1Criterion};
use super::surface::GeomPlateSurface;
use rustycad_tkernel::ncollection::{Array1, Array2, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adv_approx::AdvApproxDichoCutting;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkgeombase::adv_app2var::{
    AdvApp2VarApproxAFunc2Var, AdvApp2VarCriterion, AdvApp2VarCriterionRepartition,
    AdvApp2VarCriterionType,
};
use rustycad_tkmath::geom_abs::{IsoType, Shape};
use rustycad_tkmath::gp::{Xy, Xyz};

/// The B-spline approximation of a `GeomPlate_Surface` on its constraint box enlarged
/// by a coefficient, with at most `Nbmax` Bezier patches of degree `dgmax` at most
/// (`GeomPlate_MakeApprox`).
///
/// `let app = GeomPlateMakeApprox::with_order(&plate, 1e-4, 1, 3, 1e-3, 0, Shape::C1, 1.1)?;`
pub struct GeomPlateMakeApprox {
    surface: Handle<GeomBSplineSurface>,
    app_error: f64,
    crit_error: f64,
}

/// The approximation options both constructors share.
struct ApproxOptions {
    tol3d: f64,
    nbmax: i32,
    dgmax: i32,
    continuity: Shape,
    precision_code: i32,
    domain: [f64; 4],
}

/// `RealBounds` of the plate times `EnlargeCoeff`.
fn enlarged_box(plate: &GeomPlateSurface, enlarge_coeff: f64) -> Result<[f64; 4], Failure> {
    let (u0, u1, v0, v1) = plate.real_bounds()?;
    Ok([
        enlarge_coeff * u0,
        enlarge_coeff * u1,
        enlarge_coeff * v0,
        enlarge_coeff * v1,
    ])
}

fn filled1(value: f64) -> Result<Array1<'static, f64>, Failure> {
    let mut array: Array1<'static, f64> =
        Array1::with_bounds(1, 1).map_err(errors::array_bounds)?;
    array.init(value);
    Ok(array)
}

fn filled2(value: f64) -> Result<Array2<'static, f64>, Failure> {
    let mut array: Array2<'static, f64> =
        Array2::with_bounds(1, 1, 1, 4).map_err(errors::array_bounds)?;
    array.init(value);
    Ok(array)
}

/// Runs `AdvApp2Var_ApproxAFunc2Var` on the plate for one 3d subspace, with or
/// without a criterion, then raises a kept failure of the evaluator first.
fn approximate(
    plate: &Handle<GeomPlateSurface>,
    options: &ApproxOptions,
    crit: Option<&dyn AdvApp2VarCriterion>,
) -> Result<AdvApp2VarApproxAFunc2Var, Failure> {
    let (nul1, nul2) = (filled1(0.0)?, filled2(0.0)?);
    let (eps3d, epsfr) = (filled1(options.tol3d)?, filled2(options.tol3d)?);
    let dec = AdvApproxDichoCutting;
    let ev = MakeApproxEval::new(&plate.cast::<dyn GeomSurface>());
    let [u0, u1, v0, v1] = options.domain;
    let (tols, fronts) = (
        (Some(&nul1), Some(&nul1), Some(&eps3d)),
        (Some(&nul2), Some(&nul2), Some(&epsfr)),
    );
    let (c, p) = (options.continuity, options.precision_code);
    let (dg, nb) = (options.dgmax, options.nbmax);
    let app = match crit {
        Some(crit) => AdvApp2VarApproxAFunc2Var::with_criterion(
            0,
            0,
            1,
            tols.0,
            tols.1,
            tols.2,
            fronts.0,
            fronts.1,
            fronts.2,
            u0,
            u1,
            v0,
            v1,
            IsoType::IsoV,
            c,
            c,
            p,
            dg,
            dg,
            nb,
            &ev,
            crit,
            &dec,
            &dec,
        ),
        None => AdvApp2VarApproxAFunc2Var::new(
            0,
            0,
            1,
            tols.0,
            tols.1,
            tols.2,
            fronts.0,
            fronts.1,
            fronts.2,
            u0,
            u1,
            v0,
            v1,
            IsoType::IsoV,
            c,
            c,
            p,
            dg,
            dg,
            nb,
            &ev,
            &dec,
            &dec,
        ),
    };
    ev.take_failure()?;
    app
}

/// The target points (order 0) or normals (order 1) of the plate at its order 0
/// constraint parameters.
fn constraint_targets(
    plate: &GeomPlateSurface,
    crit_order: i32,
) -> Result<(Sequence<Xy>, Sequence<Xyz>), Failure> {
    let mut seq2d: Sequence<Xy> = Sequence::default();
    let mut seq3d: Sequence<Xyz> = Sequence::default();
    plate.constraints(&mut seq2d);
    for p2d in seq2d.iter() {
        if crit_order == 0 {
            let pp = plate.d0(p2d.x(), p2d.y())?;
            seq3d.append(Xyz::new(pp.x(), pp.y(), pp.z()));
        } else {
            let d1 = plate.d1(p2d.x(), p2d.y())?;
            let v3h = d1.d1u ^ d1.d1v;
            seq3d.append(Xyz::new(v3h.x(), v3h.y(), v3h.z()));
        }
    }
    Ok((seq2d, seq3d))
}

impl GeomPlateMakeApprox {
    fn from_approximation(
        app: &AdvApp2VarApproxAFunc2Var,
        with_criterion: bool,
    ) -> Result<Self, Failure> {
        let crit_error: f64 = if with_criterion {
            app.crit_error(3, 1)?
        } else {
            0.0
        };
        Ok(Self {
            surface: app.surface(1)?,
            app_error: app.max_error(3, 1)?,
            crit_error,
        })
    }

    /// Approximates `surf_plate` within `tol3d` under the criterion `plate_crit`
    /// (`GeomPlate_MakeApprox(SurfPlate, PlateCrit, Tol3d, Nbmax, dgmax, Continuity,
    /// EnlargeCoeff)`; the native defaults are C1 and 1.1).
    /// `GeomPlateMakeApprox::with_criterion(&plate, &crit, 1e-4, 1, 3, Shape::C1, 1.1)?`.
    pub fn with_criterion(
        surf_plate: &Handle<GeomPlateSurface>,
        plate_crit: &dyn AdvApp2VarCriterion,
        tol3d: f64,
        nbmax: i32,
        dgmax: i32,
        continuity: Shape,
        enlarge_coeff: f64,
    ) -> Result<Self, Failure> {
        let plate: &GeomPlateSurface = live(surf_plate, "the plate surface")?;
        let options = ApproxOptions {
            tol3d,
            nbmax,
            dgmax,
            continuity,
            precision_code: 0,
            domain: enlarged_box(plate, enlarge_coeff)?,
        };
        let app = approximate(surf_plate, &options, Some(plate_crit))?;
        Self::from_approximation(&app, true)
    }

    /// Approximates `surf_plate` within `tol3d` under the plate's own criterion
    /// (`GeomPlate_MakeApprox(SurfPlate, Tol3d, Nbmax, dgmax, dmax, CritOrder,
    /// Continuity, EnlargeCoeff)`; the native defaults are 0, C1 and 1.1): none for
    /// `crit_order` -1 (precision code 1), the G0 or G1 criterion on the constraint
    /// points for 0 or 1, with the bound `10 * dmax` when `tol3d` is below it.
    ///
    /// Another `crit_order` computes nothing natively and leaves the errors
    /// uninitialized; the port gives a null surface and null errors.
    /// `GeomPlateMakeApprox::with_order(&plate, 1e-4, 1, 3, 1e-3, 0, Shape::C1, 1.1)?`.
    #[allow(clippy::too_many_arguments)] // The native eight-argument constructor.
    pub fn with_order(
        surf_plate: &Handle<GeomPlateSurface>,
        tol3d: f64,
        nbmax: i32,
        dgmax: i32,
        dmax: f64,
        crit_order: i32,
        continuity: Shape,
        enlarge_coeff: f64,
    ) -> Result<Self, Failure> {
        let plate: &GeomPlateSurface = live(surf_plate, "the plate surface")?;
        let (seq2d, seq3d) = if crit_order >= 0 {
            constraint_targets(plate, crit_order)?
        } else {
            (Sequence::default(), Sequence::default())
        };
        let mut options = ApproxOptions {
            tol3d,
            nbmax,
            dgmax,
            continuity,
            precision_code: 0,
            domain: enlarged_box(plate, enlarge_coeff)?,
        };
        let seuil: f64 = if (crit_order == 0 || crit_order == 1) && tol3d < 10.0 * dmax {
            10.0 * dmax
        } else {
            tol3d
        };
        let (absolute, regular) = (
            AdvApp2VarCriterionType::Absolute,
            AdvApp2VarCriterionRepartition::Regular,
        );
        match crit_order {
            -1 => {
                options.precision_code = 1;
                let app = approximate(surf_plate, &options, None)?;
                Self::from_approximation(&app, false)
            }
            0 => {
                let crit0 =
                    GeomPlatePlateG0Criterion::new(&seq2d, &seq3d, seuil, absolute, regular);
                let app = approximate(surf_plate, &options, Some(&crit0))?;
                Self::from_approximation(&app, true)
            }
            1 => {
                let crit1 =
                    GeomPlatePlateG1Criterion::new(&seq2d, &seq3d, seuil, absolute, regular);
                let app = approximate(surf_plate, &options, Some(&crit1))?;
                Self::from_approximation(&app, true)
            }
            _ => Ok(Self {
                surface: Handle::default(),
                app_error: 0.0,
                crit_error: 0.0,
            }),
        }
    }

    /// The approximating B-spline surface (`Surface`); `app.surface()`.
    pub fn surface(&self) -> Handle<GeomBSplineSurface> {
        self.surface.clone()
    }

    /// The approximation error over the whole surface (`ApproxError`);
    /// `app.approx_error()`.
    pub fn approx_error(&self) -> f64 {
        self.app_error
    }

    /// The criterion error at the constraints, 0 without criterion
    /// (`CriterionError`); `app.criterion_error()`.
    pub fn criterion_error(&self) -> f64 {
        self.crit_error
    }
}
