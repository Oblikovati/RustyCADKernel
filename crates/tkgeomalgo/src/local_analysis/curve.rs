// Rust translation of ModelingAlgorithms/TKGeomAlgo/LocalAnalysis/LocalAnalysis_CurveContinuity.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocalAnalysis_CurveContinuity`: the continuity between two curves at a point.
use super::{LocalAnalysisStatusErrorType, folded_angle, not_done};
use rustycad_tkernel::standard::{Failure, Handle, real};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeombase::geom_lprop::CLProps;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction3, Vector3};
use std::f64::consts::PI;

/// The tolerances of a curve continuity analysis, with the native defaults
/// (`EpsNul`...`Maxlen` of `LocalAnalysis_CurveContinuity`).
///
/// `CurveContinuityTolerances { eps_c0: 1e-4, ..Default::default() }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveContinuityTolerances {
    /// `EpsNul`: the null derivative length (0.001).
    pub eps_nul: f64,
    /// `EpsC0`: the C0 distance (0.001).
    pub eps_c0: f64,
    /// `EpsC1`: the C1 angle (0.001).
    pub eps_c1: f64,
    /// `EpsC2`: the C2 angle (0.001).
    pub eps_c2: f64,
    /// `EpsG1`: the G1 angle (0.001).
    pub eps_g1: f64,
    /// `EpsG2`: the G2 angle (0.001).
    pub eps_g2: f64,
    /// `Percent`: the relative curvature variation of G2 (0.01).
    pub percent: f64,
    /// `Maxlen`: the largest length, which bounds the null curvature (10000).
    pub maxlen: f64,
}

impl Default for CurveContinuityTolerances {
    fn default() -> Self {
        Self {
            eps_nul: 0.001,
            eps_c0: 0.001,
            eps_c1: 0.001,
            eps_c2: 0.001,
            eps_g1: 0.001,
            eps_g2: 0.001,
            percent: 0.01,
            maxlen: 10000.0,
        }
    }
}

/// The continuity between two curves at `u1` and `u2` (`LocalAnalysis_CurveContinuity`).
///
/// `let c = LocalAnalysisCurveContinuity::new(&c1, 1.0, &c2, 0.0, Shape::C1, &Default::default())?; c.is_c1()?`.
#[derive(Clone, Debug)]
pub struct LocalAnalysisCurveContinuity {
    my_cont_c0: f64,
    my_cont_c1: f64,
    my_cont_c2: f64,
    my_cont_g1: f64,
    my_cont_g2: f64,
    my_courb_c1: f64,
    my_courb_c2: f64,
    my_g2_variation: f64,
    my_lambda1: f64,
    my_lambda2: f64,
    my_type_cont: Shape,
    eps: CurveContinuityTolerances,
    my_is_done: bool,
    my_error_status: Option<LocalAnalysisStatusErrorType>,
}

impl LocalAnalysisCurveContinuity {
    /// Analyses the continuity of order `order` of `curv1` at `u1` and `curv2` at `u2`
    /// (`LocalAnalysis_CurveContinuity(Curv1, u1, Curv2, u2, Order, EpsNul, EpsC0, EpsC1,
    /// EpsC2, EpsG1, EpsG2, Percent, Maxlen)`); a failed analysis is not done and
    /// records a status error. Fails as the local properties do.
    /// `LocalAnalysisCurveContinuity::new(&c1, 1.0, &c2, 0.0, Shape::G2, &Default::default())?`.
    pub fn new(
        curv1: &Handle<dyn GeomCurve>,
        u1: f64,
        curv2: &Handle<dyn GeomCurve>,
        u2: f64,
        order: Shape,
        eps: &CurveContinuityTolerances,
    ) -> Result<Self, Failure> {
        let mut analysis = Self {
            my_cont_c0: 0.0,
            my_cont_c1: 0.0,
            my_cont_c2: 0.0,
            my_cont_g1: 0.0,
            my_cont_g2: 0.0,
            my_courb_c1: 0.0,
            my_courb_c2: 0.0,
            my_g2_variation: 0.0,
            my_lambda1: 0.0,
            my_lambda2: 0.0,
            my_type_cont: order,
            eps: *eps,
            my_is_done: true,
            my_error_status: None,
        };
        let level: i32 = match order {
            Shape::C0 => 0,
            Shape::C1 | Shape::G1 => 1,
            Shape::C2 | Shape::G2 => 2,
            _ => return Ok(analysis),
        };
        let mut curve1 = CLProps::with_parameter(curv1.clone(), u1, level, eps.eps_nul)?;
        let mut curve2 = CLProps::with_parameter(curv2.clone(), u2, level, eps.eps_nul)?;
        analysis.curv_c0(&curve1, &curve2);
        match order {
            Shape::C1 => analysis.curv_c1(&mut curve1, &mut curve2)?,
            Shape::C2 => {
                analysis.curv_c1(&mut curve1, &mut curve2)?;
                analysis.curv_c2(&mut curve1, &mut curve2)?;
            }
            Shape::G1 => analysis.curv_g1(&mut curve1, &mut curve2)?,
            Shape::G2 => {
                analysis.curv_g1(&mut curve1, &mut curve2)?;
                analysis.curv_g2(&mut curve1, &mut curve2)?;
            }
            _ => {}
        }
        Ok(analysis)
    }

    fn fail(&mut self, status: LocalAnalysisStatusErrorType) {
        self.my_is_done = false;
        self.my_error_status = Some(status);
    }

    fn curv_c0(&mut self, curv1: &CLProps, curv2: &CLProps) {
        self.my_cont_c0 = curv1.value().distance(&curv2.value());
    }

    fn curv_c1(&mut self, curv1: &mut CLProps, curv2: &mut CLProps) -> Result<(), Failure> {
        let (v1, v2): (Vector3, Vector3) = (curv1.d1()?, curv2.d1()?);
        let (norm1, norm2) = (v1.magnitude(), v2.magnitude());
        if !(norm1 > self.eps.eps_nul && norm2 > self.eps.eps_nul) {
            self.fail(LocalAnalysisStatusErrorType::NullFirstDerivative);
            return Ok(());
        }
        self.my_lambda1 = if norm1 >= norm2 {
            norm2 / norm1
        } else {
            norm1 / norm2
        };
        self.my_cont_c1 = folded_angle(v1.angle(v2)?);
        Ok(())
    }

    fn curv_c2(&mut self, curv1: &mut CLProps, curv2: &mut CLProps) -> Result<(), Failure> {
        let (v1, v2): (Vector3, Vector3) = (curv1.d1()?, curv2.d1()?);
        let (v12, v22): (Vector3, Vector3) = (curv1.d2()?, curv2.d2()?);
        let (norm1, norm2) = (v1.magnitude(), v2.magnitude());
        let (norm12, norm22) = (v12.magnitude(), v22.magnitude());
        if !(norm1 > self.eps.eps_nul && norm2 > self.eps.eps_nul) {
            self.fail(LocalAnalysisStatusErrorType::NullFirstDerivative);
            return Ok(());
        }
        if !(norm12 > self.eps.eps_nul && norm22 > self.eps.eps_nul) {
            self.fail(LocalAnalysisStatusErrorType::NullSecondDerivative);
            return Ok(());
        }
        (self.my_lambda1, self.my_lambda2) = if norm1 >= norm2 {
            (norm2 / norm1, norm22 / norm12)
        } else {
            (norm1 / norm2, norm12 / norm22)
        };
        self.my_cont_c2 = folded_angle(v12.angle(v22)?);
        Ok(())
    }

    fn curv_g1(&mut self, curv1: &mut CLProps, curv2: &mut CLProps) -> Result<(), Failure> {
        if !(curv1.is_tangent_defined()? && curv2.is_tangent_defined()?) {
            self.fail(LocalAnalysisStatusErrorType::TangentNotDefined);
            return Ok(());
        }
        let (tang1, tang2): (Direction3, Direction3) = (curv1.tangent()?, curv2.tangent()?);
        self.my_cont_g1 = folded_angle(tang1.angle(&tang2));
        Ok(())
    }

    fn curv_g2(&mut self, curv1: &mut CLProps, curv2: &mut CLProps) -> Result<(), Failure> {
        let epscrb: f64 = 8.0 * self.eps.eps_c0 / (self.eps.maxlen * self.eps.maxlen);
        if !(curv1.is_tangent_defined()? && curv2.is_tangent_defined()?) {
            self.fail(LocalAnalysisStatusErrorType::TangentNotDefined);
            return Ok(());
        }
        self.my_courb_c1 = curv1.curvature()?;
        self.my_courb_c2 = curv2.curvature()?;
        if !(self.my_courb_c1.abs() > epscrb && self.my_courb_c2.abs() > epscrb) {
            self.fail(LocalAnalysisStatusErrorType::NormalNotDefined);
            return Ok(());
        }
        let (d1, d2): (Direction3, Direction3) = (curv1.normal()?, curv2.normal()?);
        self.my_cont_g2 = folded_angle(d1.angle(&d2));
        self.my_courb_c1 = curv1.curvature()?;
        self.my_courb_c2 = curv2.curvature()?;
        self.my_g2_variation = (self.my_courb_c1 - self.my_courb_c2).abs()
            / (self.my_courb_c1 * self.my_courb_c2).sqrt();
        Ok(())
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.my_is_done {
            return Err(not_done("LocalAnalysis_CurveContinuity", member));
        }
        Ok(())
    }

    fn done_value(&self, member: &str, value: f64) -> Result<f64, Failure> {
        self.check_done(member)?;
        Ok(value)
    }

    /// Whether the points are within `EpsC0` (`IsC0`); StdFail_NotDone for a failed
    /// analysis. `c.is_c0()?`.
    pub fn is_c0(&self) -> Result<bool, Failure> {
        self.check_done("IsC0")?;
        Ok(self.my_cont_c0 <= self.eps.eps_c0)
    }

    /// C0 and first derivatives parallel or opposite within `EpsC1` (`IsC1`).
    /// `c.is_c1()?`.
    pub fn is_c1(&self) -> Result<bool, Failure> {
        self.check_done("IsC1")?;
        let angle_ok: bool =
            self.my_cont_c1 <= self.eps.eps_c1 || (self.my_cont_c1 - PI).abs() <= self.eps.eps_c1;
        Ok(self.is_c0()? && angle_ok)
    }

    /// C1, second derivatives aligned within `EpsC2` and matching derivative ratios
    /// (`IsC2`). `c.is_c2()?`.
    pub fn is_c2(&self) -> Result<bool, Failure> {
        self.check_done("IsC2")?;
        if !self.is_c1()? {
            return Ok(false);
        }
        let eps = &self.eps;
        if !(self.my_cont_c2 <= eps.eps_c2 || (self.my_cont_c2 - PI).abs() <= eps.eps_c2) {
            return Ok(false);
        }
        let epsil1: f64 = 0.5 * eps.eps_c1 * eps.eps_c1 * self.my_lambda1;
        let epsil2: f64 = 0.5 * eps.eps_c2 * eps.eps_c2 * self.my_lambda2;
        Ok((self.my_lambda1 * self.my_lambda1 - self.my_lambda2).abs() <= epsil1 * epsil1 + epsil2)
    }

    /// C0 and tangents parallel or opposite within `EpsG1` (`IsG1`); `c.is_g1()?`.
    pub fn is_g1(&self) -> Result<bool, Failure> {
        self.check_done("IsG1")?;
        let angle_ok: bool =
            self.my_cont_g1 <= self.eps.eps_g1 || (self.my_cont_g1 - PI).abs() <= self.eps.eps_g1;
        Ok(self.is_c0()? && angle_ok)
    }

    /// G1 and the same curvature state (null, finite or infinite); finite curvatures
    /// also need aligned normals and a relative variation below `Percent` (`IsG2`).
    /// `c.is_g2()?`.
    pub fn is_g2(&self) -> Result<bool, Failure> {
        self.check_done("IsG2")?;
        if !self.is_g1()? {
            return Ok(false);
        }
        let crbinf: f64 = 1.0 / self.eps.eps_c0;
        let crbnul: f64 = 8.0 * self.eps.eps_c0 / (self.eps.maxlen * self.eps.maxlen);
        // The curvature states: 0 null, 1 finite, 2 infinite.
        let state = |courb: f64| -> i32 {
            if courb > crbinf {
                2
            } else if courb < crbnul {
                0
            } else {
                1
            }
        };
        let (ieta1, ieta2) = (state(self.my_courb_c1), state(self.my_courb_c2));
        if ieta1 != ieta2 {
            return Ok(false);
        }
        if ieta1 != 1 {
            return Ok(true);
        }
        let eps: f64 = real::real_part((self.my_cont_g2 + self.eps.eps_g2) / PI) * PI;
        if (eps - self.eps.eps_g2).abs() < self.eps.eps_g2 {
            return Ok(self.my_g2_variation < self.eps.percent);
        }
        Ok(false)
    }

    /// The distance between the points (`C0Value`); `c.c0_value()?`.
    pub fn c0_value(&self) -> Result<f64, Failure> {
        self.done_value("C0Value", self.my_cont_c0)
    }

    /// The angle between the first derivatives, folded into `[0, PI/2]` (`C1Angle`).
    /// `c.c1_angle()?`.
    pub fn c1_angle(&self) -> Result<f64, Failure> {
        self.done_value("C1Angle", self.my_cont_c1)
    }

    /// The ratio of the first derivative lengths, at most 1 (`C1Ratio`); `c.c1_ratio()?`.
    pub fn c1_ratio(&self) -> Result<f64, Failure> {
        self.done_value("C1Ratio", self.my_lambda1)
    }

    /// The angle between the second derivatives (`C2Angle`); `c.c2_angle()?`.
    pub fn c2_angle(&self) -> Result<f64, Failure> {
        self.done_value("C2Angle", self.my_cont_c2)
    }

    /// The ratio of the second derivative lengths (`C2Ratio`); `c.c2_ratio()?`.
    pub fn c2_ratio(&self) -> Result<f64, Failure> {
        self.done_value("C2Ratio", self.my_lambda2)
    }

    /// The angle between the tangents (`G1Angle`); `c.g1_angle()?`.
    pub fn g1_angle(&self) -> Result<f64, Failure> {
        self.done_value("G1Angle", self.my_cont_g1)
    }

    /// The angle between the normals (`G2Angle`); `c.g2_angle()?`.
    pub fn g2_angle(&self) -> Result<f64, Failure> {
        self.done_value("G2Angle", self.my_cont_g2)
    }

    /// The relative curvature variation `|k1 - k2| / sqrt(k1 k2)`
    /// (`G2CurvatureVariation`); `c.g2_curvature_variation()?`.
    pub fn g2_curvature_variation(&self) -> Result<f64, Failure> {
        self.done_value("G2CurvatureVariation", self.my_g2_variation)
    }

    /// Whether the analysis succeeded (`IsDone`); `c.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// The reason of a failed analysis (`StatusError`); `None` when no error was
    /// recorded, where the native member is uninitialized. `c.status_error()`.
    pub fn status_error(&self) -> Option<LocalAnalysisStatusErrorType> {
        self.my_error_status
    }

    /// The analysed order (`ContinuityStatus`); StdFail_NotDone for a failed analysis.
    /// `c.continuity_status()?`.
    pub fn continuity_status(&self) -> Result<Shape, Failure> {
        self.check_done("ContinuityStatus")?;
        Ok(self.my_type_cont)
    }
}
