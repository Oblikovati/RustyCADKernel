// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_EvolvedSection.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_EvolvedSection`: a section scaled from the origin by a law.
use super::frame_math::point_from_xyz;
use super::section_curve::{SectionCurve, section_tolerance};
use super::section_law::{GeomFillSectionLaw, SectionD1, SectionD2};
use super::sweep_failures::{live, sweep_failure};
use crate::law::LawFunction;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

/// `GeomFill_EvolvedSection`: `Poles(t) = L(t) * Poles`.
///
/// `IsConstant` is always false, so `ConstantSection` always raises
/// StdFail_NotDone, as natively. A B-spline input is shared, not copied.
/// `let law = GeomFillEvolvedSection::new(&curve, &law)?;`.
pub struct GeomFillEvolvedSection {
    parent: StandardTransient,
    first: f64,
    last: f64,
    curve: SectionCurve,
    my_law: Handle<dyn LawFunction>,
    t_law: SharedState<Handle<dyn LawFunction>>,
}
impl_transient!(
    GeomFillEvolvedSection,
    "GeomFill_EvolvedSection",
    dyn GeomFillSectionLaw,
    parent,
    interfaces: [dyn GeomFillSectionLaw]
);

impl GeomFillEvolvedSection {
    /// `GeomFill_EvolvedSection(C, L)`: the law trimmed to its own bounds.
    pub fn new(c: &Handle<dyn GeomCurve>, l: &Handle<dyn LawFunction>) -> Result<Self, Failure> {
        let law = live(l, "GeomFill_EvolvedSection", "L")?;
        let (first, last) = law.bounds()?;
        let my_law = law.trim(first, last, 1.0e-20)?;
        let curve = SectionCurve::new(c)?;
        Ok(Self {
            parent: StandardTransient::default(),
            first,
            last,
            curve,
            t_law: SharedState::new(my_law.clone()),
            my_law,
        })
    }

    fn law(&self) -> &dyn LawFunction {
        self.my_law.get().expect("Trim returns a law")
    }

    fn t_law(&self) -> Result<Handle<dyn LawFunction>, Failure> {
        let law = self.t_law.read().clone();
        live(&law, "GeomFill_EvolvedSection", "TLaw")?;
        Ok(law)
    }
}

/// `Poles(ii).ChangeCoord() *= val` for `ii` in `1..=l`.
fn scale_poles(poles: &mut Array1<'_, Point3>, l: i32, val: f64) -> Result<(), Failure> {
    for ii in 1..=l {
        scale_pole(poles, ii, val)?;
    }
    Ok(())
}

fn scale_pole(poles: &mut Array1<'_, Point3>, ii: i32, val: f64) -> Result<(), Failure> {
    let mut xyz = at(poles, ii)?.xyz();
    xyz *= val;
    put(poles, ii, point_from_xyz(xyz))
}

/// `DPoles(ii).SetXYZ(Poles(ii).XYZ())`, and `D2Poles(ii) = DPoles(ii) * d2val`
/// when asked, then `DPoles(ii) *= dval` and `Poles(ii) *= val`, index by index.
fn derive_poles(
    poles: &mut Array1<'_, Point3>,
    d_poles: &mut Array1<'_, Vector3>,
    mut d2: Option<(&mut Array1<'_, Vector3>, f64)>,
    (l, val, dval): (i32, f64, f64),
) -> Result<(), Failure> {
    for ii in 1..=l {
        let dp = Vector3::from(at(poles, ii)?.xyz());
        put(d_poles, ii, dp)?;
        if let Some((d2_poles, d2val)) = d2.as_mut() {
            let mut d2p = at(d_poles, ii)?;
            d2p *= *d2val;
            put(d2_poles, ii, d2p)?;
        }
        let mut dp = at(d_poles, ii)?;
        dp *= dval;
        put(d_poles, ii, dp)?;
        scale_pole(poles, ii, val)?;
    }
    Ok(())
}

impl GeomFillSectionLaw for GeomFillEvolvedSection {
    fn d0(
        &self,
        u: f64,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let l = poles.length();
        let val = self.t_law()?.get().expect("checked").value(u)?;
        poles.assign(&self.curve.curve().poles());
        scale_poles(poles, l, val)?;
        self.curve.curve().weights_array(|w| {
            weights.assign(w);
        });
        Ok(true)
    }
    fn d1(&self, u: f64, v: SectionD1<'_, '_>) -> Result<bool, Failure> {
        let l = v.poles.length();
        let (val, dval) = self.t_law()?.get().expect("checked").d1(u)?;
        self.curve.assign_poles(v.poles, v.weights);
        derive_poles(v.poles, v.d_poles, None, (l, val, dval))?;
        v.d_weights.init(0.0);
        Ok(true)
    }
    fn d2(&self, u: f64, v: SectionD2<'_, '_>) -> Result<bool, Failure> {
        let l = v.d1.poles.length();
        let (val, dval, d2val) = self.t_law()?.get().expect("checked").d2(u)?;
        self.curve.assign_poles(v.d1.poles, v.d1.weights);
        derive_poles(
            v.d1.poles,
            v.d1.d_poles,
            Some((v.d2_poles, d2val)),
            (l, val, dval),
        )?;
        v.d1.d_weights.init(0.0);
        v.d2_weights.init(0.0);
        Ok(true)
    }
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure> {
        Ok(self.curve.section_shape())
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.curve.knots(knots);
        Ok(())
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        self.curve.mults(mults);
        Ok(())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        Ok(self.curve.curve().is_rational())
    }
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        Ok(self.curve.curve().is_periodic())
    }
    /// Whether the law takes the same value at both ends (within Confusion).
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        Ok(
            (self.law().value(self.first)? - self.law().value(self.last)?).abs()
                < precision::confusion(),
        )
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.law().nb_intervals(shape)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.law().intervals(bounds, shape)
    }
    /// `TLaw = myLaw->Trim(F, L, PConfusion)`.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        *self.t_law.write() = self.law().trim(first, last, precision::p_confusion())?;
        Ok(())
    }
    fn get_interval(&self) -> Result<(f64, f64), Failure> {
        self.t_law()?.get().expect("checked").bounds()
    }
    fn get_domain(&self) -> Result<(f64, f64), Failure> {
        Ok((self.first, self.last))
    }
    fn get_tolerance(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        _angle_tol: f64,
        tol3d: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        section_tolerance(bound_tol, surf_tol, tol3d)
    }
    /// The sum of 21 section points scaled by the sum of 21 law values over 441.
    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        let mut bary = self.curve.point_sum()?;
        let (mut u, delta) = (self.first, (self.last - self.first) / 20.0);
        let mut b = 0.0;
        for _ in 0..=20 {
            b += self.law().value(u)?;
            u += delta;
        }
        bary *= b / f64::from(21 * 21);
        Ok(point_from_xyz(bary))
    }
    /// The section length times the largest of 21 law values (at least 0).
    fn maximal_section(&self) -> Result<f64, Failure> {
        let l = self.curve.length()?;
        let (mut u, delta) = (self.first, (self.last - self.first) / 20.0);
        let mut max = 0.0;
        for _ in 0..=20 {
            let val = self.law().value(u)?;
            if val > max {
                max = val;
            }
            u += delta;
        }
        Ok(l * max)
    }
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.curve.curve().weights_array(|w| {
            weights.assign(w);
        });
        Ok(())
    }
    /// Always false with `Error = 0`.
    fn is_constant(&self, error: &mut f64) -> Result<bool, Failure> {
        *error = 0.0;
        Ok(false)
    }
    /// StdFail_NotDone ("The Law is not Constant!"), since `IsConstant` is false;
    /// the native scaling code after the test is unreachable.
    fn constant_section(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let mut err = 0.0;
        if !self.is_constant(&mut err)? {
            return Err(sweep_failure("StdFail_NotDone", "The Law is not Constant!"));
        }
        let scale = self.law().value(self.first)?
            + self.law().value((self.first + self.last) / 2.0)?
            + self.law().value(self.last)?;
        let mut t = Transform3::default();
        t.set_scale(Point3::new(0.0, 0.0, 0.0), scale / 3.0)?;
        let c: Handle<dyn GeomCurve> =
            live(&self.curve.my_section, "ConstantSection", "mySection")?
                .copy()?
                .cast();
        live(&c, "ConstantSection", "C")?.transform(t)?;
        Ok(c)
    }
}
