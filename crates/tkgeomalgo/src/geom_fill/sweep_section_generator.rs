// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SweepSectionGenerator.hxx/.lxx/.cxx
// (the constructors, Init, Perform and the queries) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_SweepSectionGenerator`: the sections of the old pipe algorithm along
//! a B-spline path, for the AppBlend approximation (`GeomFill_AppSweep`)
//! (docs/geomalgo-design.md, section 14).
use super::profiler::GeomFillProfiler;
use super::sweep_failures::{live, sweep_failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, make_curve};
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Transform3};
use std::f64::consts::PI;

/// `GeomFill_SweepSectionGenerator`: the kind of sweep (`myType`: 0 the circles
/// between two rails, 1 a circle, 2 one section, 3 two sections along a curve, 4
/// to 6 the same along a circle) and the sections it generates.
///
/// Natively the constructors with arguments leave `myNbSections` and
/// `myPolynomial` unset until `Perform`; here they start at 0 and false.
/// `let mut g = GeomFillSweepSectionGenerator::with_radius(&path, 1.0)?; g.perform(false)?;`.
#[derive(Default)]
pub struct GeomFillSweepSectionGenerator {
    pub(crate) my_path: Handle<GeomBSplineCurve>,
    pub(crate) my_first_sect: Handle<GeomBSplineCurve>,
    pub(crate) my_last_sect: Handle<GeomBSplineCurve>,
    pub(crate) my_adp_path: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_adp_first_sect: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_adp_last_sect: Handle<dyn Adaptor3dCurve>,
    pub(crate) my_circ_path_axis: Axis1,
    pub(crate) my_radius: f64,
    pub(crate) my_is_done: bool,
    pub(crate) my_nb_sections: i32,
    pub(crate) my_trsfs: Vec<Transform3>,
    pub(crate) my_type: i32,
    pub(crate) my_polynomial: bool,
}

/// A copy of a B-spline curve, or the conversion of another curve.
fn bspline_of(
    curve: &Handle<dyn GeomCurve>,
    conv: ParameterisationType,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let c = live(curve, "GeomFill_SweepSectionGenerator::Init", "Curve")?;
    let spline: Handle<GeomBSplineCurve> = curve.cast();
    if spline.is_null() {
        return curve_to_bspline_curve(curve, conv);
    }
    Ok(c.copy()?.cast())
}

/// A section as a non-periodic B-spline (quasi-angular conversion).
fn open_section(curve: &Handle<dyn GeomCurve>) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let sect = bspline_of(curve, ParameterisationType::QuasiAngular)?;
    let s = live(&sect, "GeomFill_SweepSectionGenerator::Init", "Sect")?;
    if s.is_periodic() {
        s.set_not_periodic()?;
    }
    Ok(sect)
}

impl GeomFillSweepSectionGenerator {
    /// `GeomFill_SweepSectionGenerator()`: no path (`myType = -1`);
    /// `GeomFillSweepSectionGenerator::new()`.
    pub fn new() -> Self {
        Self {
            my_type: -1,
            ..Self::default()
        }
    }

    /// `GeomFill_SweepSectionGenerator(Path, Radius)`: a tube of circles;
    /// `GeomFillSweepSectionGenerator::with_radius(&path, 1.0)?`.
    pub fn with_radius(path: &Handle<dyn GeomCurve>, radius: f64) -> Result<Self, Failure> {
        let mut generator = Self::default();
        generator.init_radius(path, radius)?;
        Ok(generator)
    }

    /// `GeomFill_SweepSectionGenerator(Path, FirstSect)`;
    /// `GeomFillSweepSectionGenerator::with_section(&path, &section)?`.
    pub fn with_section(
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
    ) -> Result<Self, Failure> {
        let mut generator = Self::default();
        generator.init_section(path, first_sect)?;
        Ok(generator)
    }

    /// `GeomFill_SweepSectionGenerator(Path, FirstSect, LastSect)`;
    /// `GeomFillSweepSectionGenerator::with_sections(&path, &first, &last)?`.
    pub fn with_sections(
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        last_sect: &Handle<dyn GeomCurve>,
    ) -> Result<Self, Failure> {
        let mut generator = Self::default();
        generator.init_sections(path, first_sect, last_sect)?;
        Ok(generator)
    }

    /// `GeomFill_SweepSectionGenerator(Path, Curve1, Curve2, Radius)`;
    /// `GeomFillSweepSectionGenerator::with_rails(&path, &c1, &c2, 1.0)?`.
    pub fn with_rails(
        path: &Handle<dyn Adaptor3dCurve>,
        curve1: &Handle<dyn Adaptor3dCurve>,
        curve2: &Handle<dyn Adaptor3dCurve>,
        radius: f64,
    ) -> Result<Self, Failure> {
        let mut generator = Self::default();
        generator.init_rails(path, curve1, curve2, radius)?;
        Ok(generator)
    }

    /// The kind of the path (`circle` on a circle, else `other`) and its B-spline.
    fn set_path(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        circle: i32,
        other: i32,
    ) -> Result<(), Failure> {
        let the_path = GeomAdaptorCurve::with_curve(path)?;
        if the_path.get_type()? == CurveType::Circle {
            self.my_circ_path_axis = the_path.circle()?.axis();
            self.my_type = circle;
        } else {
            self.my_type = other;
        }
        self.my_path = bspline_of(path, ParameterisationType::TgtThetaOver2)?;
        Ok(())
    }

    /// `Init(Path, Radius)`; `generator.init_radius(&path, 1.0)?`.
    pub fn init_radius(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        radius: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        self.my_radius = radius;
        self.set_path(path, 4, 1)
    }

    /// `Init(Path, FirstSect)`; `generator.init_section(&path, &section)?`.
    pub fn init_section(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        self.my_radius = 0.0;
        self.set_path(path, 5, 2)?;
        self.my_first_sect = open_section(first_sect)?;
        Ok(())
    }

    /// `Init(Path, FirstSect, LastSect)`: both sections made compatible by a
    /// `GeomFill_Profiler`; `generator.init_sections(&path, &first, &last)?`.
    pub fn init_sections(
        &mut self,
        path: &Handle<dyn GeomCurve>,
        first_sect: &Handle<dyn GeomCurve>,
        last_sect: &Handle<dyn GeomCurve>,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        self.my_radius = 0.0;
        self.set_path(path, 6, 3)?;
        let first = bspline_of(first_sect, ParameterisationType::QuasiAngular)?;
        let last = bspline_of(last_sect, ParameterisationType::QuasiAngular)?;
        self.my_first_sect = first.clone();
        self.my_last_sect = last.clone();
        for sect in [&first, &last] {
            let s = live(sect, "GeomFill_SweepSectionGenerator::Init", "Sect")?;
            if s.is_periodic() {
                s.set_not_periodic()?;
            }
        }
        let mut profil = GeomFillProfiler::new();
        profil.add_curve(&first.cast())?;
        profil.add_curve(&last.cast())?;
        profil.perform(precision::confusion())?;
        self.my_first_sect = profil.curve(1)?.cast();
        self.my_last_sect = profil.curve(2)?.cast();
        Ok(())
    }

    /// `Init(Path, Curve1, Curve2, Radius)`: the circles between two rails;
    /// `generator.init_rails(&path, &c1, &c2, 1.0)?`.
    pub fn init_rails(
        &mut self,
        path: &Handle<dyn Adaptor3dCurve>,
        curve1: &Handle<dyn Adaptor3dCurve>,
        curve2: &Handle<dyn Adaptor3dCurve>,
        radius: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        self.my_radius = radius;
        self.my_type = 0;
        let cc = make_curve(live(path, "GeomFill_SweepSectionGenerator::Init", "Path")?)?;
        self.my_path = curve_to_bspline_curve(&cc, ParameterisationType::TgtThetaOver2)?;
        self.my_adp_path = path.clone();
        self.my_adp_first_sect = curve1.clone();
        self.my_adp_last_sect = curve2.clone();
        Ok(())
    }

    pub(crate) fn path(&self, member: &str) -> Result<&GeomBSplineCurve, Failure> {
        live(&self.my_path, member, "myPath")
    }

    pub(crate) fn first_sect(&self, member: &str) -> Result<&GeomBSplineCurve, Failure> {
        live(&self.my_first_sect, member, "myFirstSect")
    }

    /// `Perform(Polynomial)`: the section count (21 per path span, or the points of
    /// a deflection sampling) and the transformation of each section. Along a curve
    /// the transformation stays the identity unless the tangent turns back: the
    /// native code composes `TR`, set only in that case (a quirk kept here). The
    /// transformations are appended to those of an earlier `Perform`, as natively;
    /// `generator.perform(false)?`.
    pub fn perform(&mut self, polynomial: bool) -> Result<(), Failure> {
        self.my_polynomial = polynomial;
        let member = "GeomFill_SweepSectionGenerator::Perform";
        let (nb_knots, u1, u2) = {
            let path = self.path(member)?;
            (
                path.nb_knots(),
                path.first_parameter(),
                path.last_parameter(),
            )
        };
        self.my_nb_sections = 21 * (nb_knots - 1);
        let samples = self.sample_count(u1, u2)?;
        if let Some(n) = samples
            && n > self.my_nb_sections
        {
            self.my_nb_sections = n;
        }
        let parameters = self.parameters(u1, u2);
        if (1..=3).contains(&self.my_type) {
            if self.my_type == 1 {
                self.circle_section()?;
            }
            self.along_curve(&parameters, u1, u2)?;
        } else if self.my_type != 0 {
            if self.my_type == 4 {
                self.circle_section()?;
            }
            let mut cumul_tr = Transform3::default();
            for parameter in parameters.iter().skip(1) {
                cumul_tr.set_rotation(self.my_circ_path_axis, *parameter)?;
                self.my_trsfs.push(cumul_tr);
            }
        }
        self.my_is_done = true;
        Ok(())
    }

    /// The point count of a deflection sampling of the path (1e-5 of its three-point
    /// length), or `None` when the sampling fails.
    fn sample_count(&self, u1: f64, u2: f64) -> Result<Option<i32>, Failure> {
        let adp_path = GeomAdaptorCurve::with_curve(&self.my_path.clone().cast())?;
        let p1 = adp_path.value(u1)?;
        let p2 = adp_path.value((u1 + u2) / 2.0)?;
        let p3 = adp_path.value(u2)?;
        let length = p1.distance(&p2) + p2.distance(&p3);
        let mut samp = QuasiUniformDeflection::new();
        samp.initialize(&adp_path, 1.0e-5 * length, Shape::C1)?;
        if !samp.is_done() {
            return Ok(None);
        }
        Ok(Some(samp.nb_points()?))
    }

    /// `Parameters`: the section parameters from 0, by equal steps, to `U2 - U1`.
    fn parameters(&self, u1: f64, u2: f64) -> Vec<f64> {
        let n = self.my_nb_sections;
        let delta_u = (u2 - u1) / f64::from(n - 1);
        let mut parameters = vec![0.0; n.max(1) as usize];
        for i in 2..n {
            parameters[(i - 1) as usize] = f64::from(i - 1) * delta_u;
        }
        parameters[(n.max(1) - 1) as usize] = u2 - u1;
        parameters
    }

    /// The first section of a tube: the circle of `myRadius` normal to the path
    /// start, converted quasi-angularly.
    fn circle_section(&mut self) -> Result<(), Failure> {
        let path = self.path("GeomFill_SweepSectionGenerator::Perform")?;
        let (pref, d1ref) = path.d1(path.first_parameter())?;
        let circle_axis = Axis2::new(pref, Direction3::from_vector(d1ref)?)?;
        let circle: Handle<dyn GeomCurve> =
            Handle::new(GeomCircle::new(circle_axis, self.my_radius)?).cast();
        let circ: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&circle, 0.0, 2.0 * PI, true, true)?).cast();
        self.my_first_sect = curve_to_bspline_curve(&circ, ParameterisationType::QuasiAngular)?;
        Ok(())
    }

    /// The transformations along a curve path (see `perform`).
    fn along_curve(&mut self, parameters: &[f64], u1: f64, u2: f64) -> Result<(), Failure> {
        let path = self.path("GeomFill_SweepSectionGenerator::Perform")?;
        let (mut pref, mut d1ref) = path.d1(u1)?;
        let (mut tr, mut cumul_tr) = (Transform3::default(), Transform3::default());
        let n = parameters.len();
        let mut trsfs = Vec::with_capacity(n);
        for (i, parameter) in parameters.iter().enumerate().skip(1) {
            let u = if i + 1 == n { u2 } else { parameter + u1 };
            let (p, d1) = path.d1(u)?;
            let trans = Transform3::translation_between(pref, p);
            let mut rot = Transform3::default();
            if !d1ref.is_parallel(d1, precision::angular())? {
                let normal = d1ref.crossed(d1);
                let angle = d1ref.angle_with_ref(d1, normal)?;
                rot.set_rotation(Axis1::new(p, Direction3::from_vector(normal)?), angle)?;
            } else if d1ref.is_opposite(d1, precision::angular())? {
                tr = rot.multiplied(trans);
            }
            cumul_tr = tr.multiplied(cumul_tr);
            trsfs.push(cumul_tr);
            (pref, d1ref) = (p, d1);
        }
        self.my_trsfs.extend(trsfs);
        Ok(())
    }

    /// `GetShape(NbPoles, NbKnots, Degree, NbPoles2d)`: the first section, or
    /// `(7, 2, 6)` for the circles between rails; `generator.get_shape()?`.
    pub fn get_shape(&self) -> Result<[i32; 4], Failure> {
        if self.my_type == 0 {
            return Ok([7, 2, 6, 0]);
        }
        let s = self.first_sect("GeomFill_SweepSectionGenerator::GetShape")?;
        Ok([s.nb_poles(), s.nb_knots(), s.degree(), 0])
    }

    /// `Knots(TKnots)`: the knots of the first section (assigned), or `0, 1`;
    /// `generator.knots(&mut knots)?`.
    pub fn knots(&self, t_knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        if self.my_type != 0 {
            t_knots.assign(
                &self
                    .first_sect("GeomFill_SweepSectionGenerator::Knots")?
                    .knots(),
            );
            return Ok(());
        }
        put(t_knots, 1, 0.0)?;
        put(t_knots, 2, 1.0)
    }

    /// `Mults(TMults)`: the multiplicities of the first section, or `7, 7`;
    /// `generator.mults(&mut mults)?`.
    pub fn mults(&self, t_mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        if self.my_type != 0 {
            t_mults.assign(
                &self
                    .first_sect("GeomFill_SweepSectionGenerator::Mults")?
                    .multiplicities(),
            );
            return Ok(());
        }
        put(t_mults, 2, 7)?;
        put(t_mults, 1, 7)
    }

    /// `NbSections()`; `generator.nb_sections()`.
    pub fn nb_sections(&self) -> i32 {
        self.my_nb_sections
    }

    /// `Transformation(Index)`: Standard_RangeError above the count;
    /// `let t = generator.transformation(1)?;`.
    pub fn transformation(&self, index: i32) -> Result<Transform3, Failure> {
        let length = self.my_trsfs.len() as i32;
        if index > length {
            return Err(sweep_failure(
                "Standard_RangeError",
                &format!(
                    "GeomFill_SweepSectionGenerator::Transformation (Index = {index}; expected <= {length})"
                ),
            ));
        }
        if index < 1 {
            return Err(sweep_failure(
                "Standard_OutOfRange",
                &format!("NCollection_Sequence::Value (Index = {index}; expected >= 1)"),
            ));
        }
        Ok(self.my_trsfs[(index - 1) as usize])
    }

    /// `Parameter(P)`: the path parameter of section `p`;
    /// `generator.parameter(2)?`.
    pub fn parameter(&self, p: i32) -> Result<f64, Failure> {
        let path = self.path("GeomFill_SweepSectionGenerator::Parameter")?;
        let (u1, u2) = (path.first_parameter(), path.last_parameter());
        if p == 1 {
            return Ok(u1);
        }
        if p == self.my_nb_sections {
            return Ok(u2);
        }
        let n = self.my_nb_sections;
        Ok((f64::from(n - p) * u1 + f64::from(p - 1) * u2) / f64::from(n - 1))
    }
}
