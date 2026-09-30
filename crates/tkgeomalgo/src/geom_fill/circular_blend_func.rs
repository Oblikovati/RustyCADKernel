// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CircularBlendFunc.hxx/.cxx
// (the constructor, Discret and the static helpers) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_CircularBlendFunc`: the circular arcs of radius `Radius` from a point
//! of `Curve1` to a point of `Curve2` around the point of `Path` at the same
//! parameter, as an `Approx_SweepFunction` (docs/geomalgo-design.md, section 14).
use super::geomfill::get_shape;
use super::sweep_failures::live;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::approx::SweepFunction;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
use std::f64::consts::PI;

/// The trimmed curves the evaluators read (`myTPath`, `myTCurve1`, `myTCurve2`).
#[derive(Clone)]
pub(crate) struct TrimmedCurves {
    pub path: Handle<dyn Adaptor3dCurve>,
    pub curve1: Handle<dyn Adaptor3dCurve>,
    pub curve2: Handle<dyn Adaptor3dCurve>,
}

/// `GeomFill_CircularBlendFunc`: the rolling-ball blend section.
/// `let f = GeomFillCircularBlendFunc::new(&path, &c1, &c2, 2.0, false)?;`.
pub struct GeomFillCircularBlendFunc {
    parent: StandardTransient,
    pub(crate) my_path: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_curve1: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_curve2: Handle<dyn Adaptor3dCurve>,
    pub(crate) trimmed: SharedState<TrimmedCurves>,
    pub(crate) my_bary: Point3,
    pub(crate) my_radius: f64,
    pub(crate) maxang: f64,
    pub(crate) minang: f64,
    pub(crate) my_degree: i32,
    pub(crate) my_nb_knots: i32,
    pub(crate) my_nb_poles: i32,
    pub(crate) my_t_conv: ParameterisationType,
    pub(crate) myreverse: bool,
}

impl_transient!(
    GeomFillCircularBlendFunc,
    "GeomFill_CircularBlendFunc",
    dyn SweepFunction,
    parent,
    interfaces: [dyn SweepFunction]
);

/// The static `GeomFillNextShape`: the continuity one order higher (CN stays).
pub(crate) fn next_shape(s: Shape) -> Shape {
    match s {
        Shape::C0 => Shape::C1,
        Shape::C1 => Shape::C2,
        Shape::C2 => Shape::C3,
        _ => Shape::CN,
    }
}

/// The static `GeomFillFusInt`: the merge of two sorted interval arrays, a pair
/// within 0.99 PConfusion() becoming its mean.
pub(crate) fn fus_int(i1: &[f64], i2: &[f64]) -> Vec<f64> {
    let epspar = precision::p_confusion() * 0.99;
    let (mut ind1, mut ind2, mut seq) = (0, 0, Vec::new());
    while ind1 < i1.len() && ind2 < i2.len() {
        let (v1, v2) = (i1[ind1], i2[ind2]);
        if (v1 - v2).abs() <= epspar {
            seq.push((v1 + v2) / 2.0);
            (ind1, ind2) = (ind1 + 1, ind2 + 1);
        } else if v1 < v2 {
            seq.push(v1);
            ind1 += 1;
        } else {
            seq.push(v2);
            ind2 += 1;
        }
    }
    seq.extend_from_slice(&i2[ind2..]);
    seq.extend_from_slice(&i1[ind1..]);
    seq
}

/// The sampled extent of the blend: the barycentre of the sampled pairs and the
/// extreme opening angles. The native member `distmin` (the smallest distance
/// between the rails) is written and never read, and is left out.
struct Discretization {
    bary: Xyz,
    maxang: f64,
    minang: f64,
}

impl Discretization {
    /// The samples of the deflection points (or of 21 equal steps when the sampling
    /// failed); returns the barycentre weight `1 / (2 n)`.
    fn sample_all(
        &mut self,
        curves: &TrimmedCurves,
        samp: &QuasiUniformDeflection,
        (t_first, t_last): (f64, f64),
    ) -> Result<f64, Failure> {
        if samp.is_done() {
            for ii in 1..=samp.nb_points()? {
                self.sample(curves, samp.parameter(ii)?)?;
            }
            return Ok(1.0 / f64::from(2 * samp.nb_points()?));
        }
        let delta = (t_last - t_first) / 20.0;
        let mut t = t_first;
        for _ in 0..=20 {
            self.sample(curves, t)?;
            t += delta;
        }
        Ok(1.0 / 42.0)
    }

    /// One sample: the opening angle at `t` and the pair added to the barycentre.
    fn sample(&mut self, curves: &TrimmedCurves, t: f64) -> Result<(), Failure> {
        let p1 = live(&curves.curve1, "GeomFill_CircularBlendFunc", "myCurve1")?.d0(t)?;
        let p2 = live(&curves.curve2, "GeomFill_CircularBlendFunc", "myCurve2")?.d0(t)?;
        let center = live(&curves.path, "GeomFill_CircularBlendFunc", "myPath")?.d0(t)?;
        let mut ns1 = Vector3::from(center.xyz() - p1.xyz());
        let mut ns2 = Vector3::from(center.xyz() - p2.xyz());
        ns1.normalize()?;
        ns2.normalize()?;
        let mut cosa = ns1.dot(ns2);
        if cosa > 1.0 {
            cosa = 1.0;
        }
        let angle = cosa.acos().abs();
        if angle > self.maxang {
            self.maxang = angle;
        }
        if angle < self.minang {
            self.minang = angle;
        }
        self.bary += p1.xyz() + p2.xyz();
        Ok(())
    }
}

/// The length of the polyline through the start, middle and end of `c`.
fn three_point_length(c: &dyn Adaptor3dCurve, t_first: f64, t_last: f64) -> Result<f64, Failure> {
    let p1 = c.value(t_first)?;
    let p2 = c.value((t_first + t_last) / 2.0)?;
    let p3 = c.value(t_last)?;
    Ok(p1.distance(&p2) + p2.distance(&p3))
}

/// `Discret()`: samples the longer rail by a deflection of 1 % of its length (or
/// 21 equal steps when the sampling fails).
fn discret(curves: &TrimmedCurves) -> Result<Discretization, Failure> {
    let path = live(&curves.path, "GeomFill_CircularBlendFunc", "myPath")?;
    let (t_first, t_last) = (path.first_parameter()?, path.last_parameter()?);
    let c1 = live(&curves.curve1, "GeomFill_CircularBlendFunc", "myCurve1")?;
    let c2 = live(&curves.curve2, "GeomFill_CircularBlendFunc", "myCurve2")?;
    let (l1, l2) = (
        three_point_length(c1, t_first, t_last)?,
        three_point_length(c2, t_first, t_last)?,
    );
    let (l, c) = if l1 > l2 { (l1, c1) } else { (l2, c2) };
    let mut samp = QuasiUniformDeflection::new();
    samp.initialize(c, 1.0e-2 * l, Shape::C1)?;
    let mut d = Discretization {
        bary: Xyz::new(0.0, 0.0, 0.0),
        maxang: f64::MIN,
        minang: f64::MAX,
    };
    let percent = d.sample_all(curves, &samp, (t_first, t_last))?;
    d.bary *= percent;
    let t = (t_first + t_last) / 2.0;
    c1.d0(t)?;
    c2.d0(t)?;
    path.d1(t)?;
    Ok(d)
}

/// The arc parameterisation: polynomial on request, quasi-angular above 0.65 PI
/// (continuous), else rational by half-angle tangent.
fn arc_parameterisation(polynomial: bool, maxang: f64) -> ParameterisationType {
    if polynomial {
        ParameterisationType::Polynomial
    } else if maxang > 0.65 * PI {
        ParameterisationType::QuasiAngular
    } else {
        ParameterisationType::TgtThetaOver2
    }
}

impl GeomFillCircularBlendFunc {
    /// `GeomFill_CircularBlendFunc(Path, Curve1, Curve2, Radius, Polynomial)`: the
    /// sampled opening angles choose the arc parameterisation (polynomial on
    /// request, quasi-angular above 0.65 PI, else rational by half-angle tangent);
    /// `GeomFillCircularBlendFunc::new(&path, &c1, &c2, 2.0, false)?`.
    pub fn new(
        path: &Handle<dyn Adaptor3dCurve>,
        curve1: &Handle<dyn Adaptor3dCurve>,
        curve2: &Handle<dyn Adaptor3dCurve>,
        radius: f64,
        polynomial: bool,
    ) -> Result<Self, Failure> {
        let curves = TrimmedCurves {
            path: path.clone(),
            curve1: curve1.clone(),
            curve2: curve2.clone(),
        };
        let d = discret(&curves)?;
        let mut conv = arc_parameterisation(polynomial, d.maxang);
        let (nb_poles, nb_knots, degree) = get_shape(d.maxang, &mut conv);
        Ok(Self {
            parent: StandardTransient::default(),
            my_path: path.clone(),
            my_curve1: curve1.clone(),
            my_curve2: curve2.clone(),
            trimmed: SharedState::new(curves),
            my_bary: Point3::from(d.bary),
            my_radius: radius,
            maxang: d.maxang,
            minang: d.minang,
            my_degree: degree,
            my_nb_knots: nb_knots,
            my_nb_poles: nb_poles,
            my_t_conv: conv,
            myreverse: false,
        })
    }
}
