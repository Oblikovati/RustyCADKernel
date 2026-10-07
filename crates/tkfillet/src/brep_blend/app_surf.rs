// Rust translation of BRepBlend_AppSurf.hxx and BRepBlend_AppSurf_0.cxx (the AppBlend_AppSurf
// instance on BRepBlend_Line and Blend_AppFunction) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_AppSurf`: the `AppBlend_AppSurf` approximation of the sections of a blend
//! function along a blend line. The template is one Rust type; this module gives it the
//! line (`TheLine`) and the function (`TheSectionGenerator`) of BRepBlend.
use super::BRepBlendLine;
use crate::blend::{BlendAppFunction, BlendPoint, SectionPoles};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::app_blend::{
    AppBlendAppSurf, AppBlendSection, TheLine, TheSectionGenerator,
};
use rustycad_tkgeombase::approx::SweepD1;
use std::cell::RefCell;

/// `BRepBlend_AppSurf`: the AppBlend approximation (`perform`, `perform_smoothing`,
/// `perform_by_pieces` take the line and a [`BlendSectionGenerator`]).
pub type BRepBlendAppSurf = AppBlendAppSurf;

impl TheLine for BRepBlendLine {
    type Point = BlendPoint;
    fn nb_points(&self) -> i32 {
        BRepBlendLine::nb_points(self)
    }
    /// `Point(Index)`: the approximation reads only `1..=NbPoints`; another index reads
    /// the default point (natively Standard_OutOfRange).
    fn point(&self, index: i32) -> BlendPoint {
        BRepBlendLine::point(self, index)
            .copied()
            .unwrap_or_default()
    }
}

/// `TheSectionGenerator` of `BRepBlend_AppSurf`: the blend function (`Blend_AppFunction&`)
/// whose `GetShape`, `Knots` and `Mults` are non-const natively, hence the cell.
///
/// ```ignore
/// let mut generator = BlendSectionGenerator::new(&mut function);
/// approx.perform(Some(&line), &mut generator, false)?;
/// ```
pub struct BlendSectionGenerator<'a>(RefCell<&'a mut dyn BlendAppFunction>);

impl<'a> BlendSectionGenerator<'a> {
    /// The generator of `function`.
    pub fn new(function: &'a mut dyn BlendAppFunction) -> Self {
        Self(RefCell::new(function))
    }
}

impl TheSectionGenerator for BlendSectionGenerator<'_> {
    type Point = BlendPoint;
    fn get_shape(&self) -> Result<[i32; 4], Failure> {
        let (nb_poles, nb_knots, degree, nb_poles2d) = self.0.borrow_mut().get_shape()?;
        Ok([nb_poles, nb_knots, degree, nb_poles2d])
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        self.0.borrow_mut().knots(knots)
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        self.0.borrow_mut().mults(mults)
    }
    fn section_d1(&mut self, p: &BlendPoint, s: &mut AppBlendSection) -> Result<bool, Failure> {
        let values = SweepD1 {
            poles: &mut s.poles,
            d_poles: &mut s.d_poles,
            poles2d: &mut s.poles2d,
            d_poles2d: &mut s.d_poles2d,
            weights: &mut s.weights,
            d_weights: &mut s.d_weights,
        };
        self.0.get_mut().section_d1(p, values)
    }
    fn section(&mut self, p: &BlendPoint, s: &mut AppBlendSection) -> Result<(), Failure> {
        let values = SectionPoles {
            poles: &mut s.poles,
            poles2d: &mut s.poles2d,
            weights: &mut s.weights,
        };
        self.0.get_mut().section(p, values)
    }
    fn parameter(&self, p: &BlendPoint) -> Result<f64, Failure> {
        Ok(self.0.borrow().parameter(p))
    }
}
