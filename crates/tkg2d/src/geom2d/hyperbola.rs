// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Hyperbola.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Hyperbola`: the main branch of a hyperbola, parameterized by `cosh`/`sinh`.
use super::conic::{Geom2dConic, dump_conic, impl_conic_position, reverse_conic_position};
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis2d, Axis22d, Hyperbola2, Point2, Transform2, Vector2, Xy, resolution,
};
use std::io;

#[derive(Clone, Copy)]
struct HyperbolaState {
    pos: Axis22d,
    major_radius: f64,
    minor_radius: f64,
}

/// A hyperbola branch; `Handle::new(Geom2dHyperbola::from_axis(axis, 3.0, 2.0, true)?)`.
pub struct Geom2dHyperbola {
    parent: StandardTransient,
    state: SharedState<HyperbolaState>,
}
impl_transient!(
    Geom2dHyperbola,
    "Geom2d_Hyperbola",
    dyn Geom2dConic,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dConic]
);
impl Clone for Geom2dHyperbola {
    fn clone(&self) -> Self {
        Self::stored(self.state.snapshot())
    }
}

fn check_radius(radius: f64, which: &str) -> Result<f64, Failure> {
    if radius < 0.0 {
        return Err(errors::construction(&format!(
            "Geom2d_Hyperbola: {which} = {radius}; expected {which} >= 0"
        )));
    }
    Ok(radius)
}

// The native Standard_DomainError_Raise_if of Eccentricity and Parameter.
fn check_major_for_ratio(major_radius: f64, member: &str) -> Result<(), Failure> {
    if major_radius <= resolution() {
        return Err(errors::domain(&format!(
            "Geom2d_Hyperbola::{member}: MajorRadius = {major_radius}; expected > gp::Resolution()"
        )));
    }
    Ok(())
}

impl Geom2dHyperbola {
    fn stored(state: HyperbolaState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    fn gp(&self) -> Result<Hyperbola2, Failure> {
        let s: HyperbolaState = *self.state.read();
        Hyperbola2::new(s.pos, s.major_radius, s.minor_radius)
    }
    /// Hyperbola from a gp one (`Geom2d_Hyperbola(gp_Hypr2d)`); `Geom2dHyperbola::from_hypr2d(h)`.
    pub fn from_hypr2d(hyperbola: Hyperbola2) -> Self {
        Self::stored(HyperbolaState {
            pos: hyperbola.axis(),
            major_radius: hyperbola.major_radius(),
            minor_radius: hyperbola.minor_radius(),
        })
    }
    /// Hyperbola on its major axis with a sense (`Geom2d_Hyperbola(gp_Ax2d, Major, Minor, Sense)`);
    /// `Geom2dHyperbola::from_axis(axis, 3.0, 2.0, true)?`.
    pub fn from_axis(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        let pos = Axis22d::from_axis(major_axis, sense)?;
        Self::new(pos, major_radius, minor_radius)
    }
    /// Hyperbola in a coordinate system (`Geom2d_Hyperbola(gp_Ax22d, Major, Minor)`);
    /// `Geom2dHyperbola::new(frame, 3.0, 2.0)?`.
    pub fn new(axis: Axis22d, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        check_radius(major_radius, "MajorRadius")?;
        check_radius(minor_radius, "MinorRadius")?;
        Ok(Self::stored(HyperbolaState {
            pos: axis,
            major_radius,
            minor_radius,
        }))
    }
    /// Replace by a gp hyperbola (`SetHypr2d`); `hyperbola.set_hypr2d(h)`.
    pub fn set_hypr2d(&self, hyperbola: Hyperbola2) {
        *self.state.write() = HyperbolaState {
            pos: hyperbola.axis(),
            major_radius: hyperbola.major_radius(),
            minor_radius: hyperbola.minor_radius(),
        };
    }
    /// Set the major radius (`SetMajorRadius`); `hyperbola.set_major_radius(4.0)?`.
    pub fn set_major_radius(&self, major_radius: f64) -> Result<(), Failure> {
        self.state.write().major_radius = check_radius(major_radius, "MajorRadius")?;
        Ok(())
    }
    /// Set the minor radius (`SetMinorRadius`); `hyperbola.set_minor_radius(1.0)?`.
    pub fn set_minor_radius(&self, minor_radius: f64) -> Result<(), Failure> {
        self.state.write().minor_radius = check_radius(minor_radius, "MinorRadius")?;
        Ok(())
    }
    /// The gp hyperbola (`Hypr2d`); `hyperbola.hypr2d()?`.
    pub fn hypr2d(&self) -> Result<Hyperbola2, Failure> {
        self.gp()
    }
    /// First asymptote (`Asymptote1`); `hyperbola.asymptote1()?`.
    pub fn asymptote1(&self) -> Result<Axis2d, Failure> {
        self.gp()?.asymptote1()
    }
    /// Second asymptote (`Asymptote2`); `hyperbola.asymptote2()?`.
    pub fn asymptote2(&self) -> Result<Axis2d, Failure> {
        self.gp()?.asymptote2()
    }
    /// First conjugate branch (`ConjugateBranch1`); `hyperbola.conjugate_branch1()?`.
    pub fn conjugate_branch1(&self) -> Result<Hyperbola2, Failure> {
        Ok(self.gp()?.conjugate_branch1())
    }
    /// Second conjugate branch (`ConjugateBranch2`); `hyperbola.conjugate_branch2()?`.
    pub fn conjugate_branch2(&self) -> Result<Hyperbola2, Failure> {
        Ok(self.gp()?.conjugate_branch2())
    }
    /// First directrix (`Directrix1`); `hyperbola.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis2d, Failure> {
        self.gp()?.directrix1()
    }
    /// Second directrix (`Directrix2`); `hyperbola.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis2d, Failure> {
        self.gp()?.directrix2()
    }
    /// Distance between the foci (`Focal`); `hyperbola.focal()`.
    pub fn focal(&self) -> f64 {
        let s: HyperbolaState = *self.state.read();
        let (a, b) = (s.major_radius, s.minor_radius);
        2.0 * (a * a + b * b).sqrt()
    }
    /// Focus of the main branch (`Focus1`); `hyperbola.focus1()`.
    pub fn focus1(&self) -> Point2 {
        self.focus(false)
    }
    /// Focus of the other branch (`Focus2`); `hyperbola.focus2()`.
    pub fn focus2(&self) -> Point2 {
        self.focus(true)
    }
    // Pxy = XDirection * (+-C) + Location, with the native XY operations.
    fn focus(&self, negative: bool) -> Point2 {
        let s: HyperbolaState = *self.state.read();
        let (a, b) = (s.major_radius, s.minor_radius);
        let c: f64 = (a * a + b * b).sqrt();
        let mut pxy: Xy = s.pos.x_direction().xy();
        pxy *= if negative { -c } else { c };
        pxy += s.pos.location().xy();
        Point2::from(pxy)
    }
    /// The major radius (`MajorRadius`); `hyperbola.major_radius()`.
    pub fn major_radius(&self) -> f64 {
        self.state.read().major_radius
    }
    /// The minor radius (`MinorRadius`); `hyperbola.minor_radius()`.
    pub fn minor_radius(&self) -> f64 {
        self.state.read().minor_radius
    }
    /// The other branch (`OtherBranch`); `hyperbola.other_branch()?`.
    pub fn other_branch(&self) -> Result<Hyperbola2, Failure> {
        Ok(self.gp()?.other_branch())
    }
    /// Semi-latus rectum `b²/a`; Standard_DomainError for a null major radius (`Parameter`);
    /// `hyperbola.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        let s: HyperbolaState = *self.state.read();
        check_major_for_ratio(s.major_radius, "Parameter")?;
        Ok((s.minor_radius * s.minor_radius) / s.major_radius)
    }
}
impl Geom2dGeometry for Geom2dHyperbola {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.major_radius *= transform.scale_factor().abs();
        state.minor_radius *= transform.scale_factor().abs();
        state.pos.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state: HyperbolaState = *self.state.read();
        dump_class_name(stream, "Geom2d_Hyperbola")?;
        dump_base(stream, depth, |s, d| dump_conic(s, d, state.pos))?;
        dump::dump_real(stream, "majorRadius", state.major_radius)?;
        dump::dump_real(stream, "minorRadius", state.minor_radius)
    }
}
impl Geom2dCurve for Geom2dHyperbola {
    fn reverse(&self) -> Result<(), Failure> {
        reverse_conic_position(&mut self.state.write().pos)
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(-u)
    }
    crate::open_curve_range!(-precision::infinite(), precision::infinite());
    crate::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        let s: HyperbolaState = *self.state.read();
        Ok(elclib::hyperbola_value_2d(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
        ))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let s: HyperbolaState = *self.state.read();
        let (point, d1) = elclib::hyperbola_d1_2d(u, s.pos, s.major_radius, s.minor_radius);
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let s: HyperbolaState = *self.state.read();
        let (point, d1, d2) = elclib::hyperbola_d2_2d(u, s.pos, s.major_radius, s.minor_radius);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let s: HyperbolaState = *self.state.read();
        let (point, d1, d2, d3) = elclib::hyperbola_d3_2d(u, s.pos, s.major_radius, s.minor_radius);
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom2d_Hyperbola::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        let s: HyperbolaState = *self.state.read();
        Ok(elclib::hyperbola_dn_2d(
            u,
            s.pos,
            s.major_radius,
            s.minor_radius,
            n,
        ))
    }
}
impl Geom2dConic for Geom2dHyperbola {
    impl_conic_position!();
    fn eccentricity(&self) -> Result<f64, Failure> {
        let s: HyperbolaState = *self.state.read();
        check_major_for_ratio(s.major_radius, "Eccentricity")?;
        let (a, b) = (s.major_radius, s.minor_radius);
        Ok((a * a + b * b).sqrt() / a)
    }
}
