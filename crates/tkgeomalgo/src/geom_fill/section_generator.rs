// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SectionGenerator.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_SectionGenerator`: the profiled sections of a skinning, served to
//! `GeomFill_AppSurf` as its `TheSectionGenerator`.
use super::GeomFillProfiler;
use super::profiler_unify::curve_at;
use crate::app_blend::{AppBlendSection, TheSectionGenerator};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use std::ops::{Deref, DerefMut};

/// The sections of a skinning (`GeomFill_SectionGenerator`): a `GeomFill_Profiler`
/// (reached through `Deref`) with the parameters imposed on its sections.
///
/// `let mut g = GeomFillSectionGenerator::new(); g.add_curve(&c1)?; g.add_curve(&c2)?; g.perform(1e-9)?;`
#[derive(Clone, Default)]
pub struct GeomFillSectionGenerator {
    profiler: GeomFillProfiler,
    my_params: Handle<HArray1<f64>>,
}

impl Deref for GeomFillSectionGenerator {
    type Target = GeomFillProfiler;
    fn deref(&self) -> &GeomFillProfiler {
        &self.profiler
    }
}

impl DerefMut for GeomFillSectionGenerator {
    fn deref_mut(&mut self) -> &mut GeomFillProfiler {
        &mut self.profiler
    }
}

impl GeomFillSectionGenerator {
    /// An empty generator without parameters (`GeomFill_SectionGenerator()`): the
    /// native constructor sets the parameters `0..n-1` only when the profile already
    /// holds sections, which a new profile never does, so `Parameter` raises until
    /// `SetParam`; `GeomFillSectionGenerator::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetParam(Params)`: shares `params` and copies `Params(Lower + i - 1)` into
    /// its index `i` for `i` in `1..=Length`. The array is both source and target,
    /// so a lower bound other than 1 shifts values in place and then raises
    /// Standard_OutOfRange, as natively; a null handle raises Standard_NullObject;
    /// `generator.set_param(&params)?`.
    pub fn set_param(&mut self, params: &Handle<HArray1<f64>>) -> Result<(), Failure> {
        let array: &HArray1<f64> = params
            .get()
            .ok_or_else(|| errors::null_object("GeomFill_SectionGenerator::SetParam: null"))?;
        let (lower, upper) = {
            let values = array.array1();
            (values.lower(), values.upper())
        };
        self.my_params = params.clone();
        for ii in 1..=(upper - lower + 1) {
            let value: f64 = read_param(array, lower + ii - 1)?;
            write_param(array, ii, value)?;
        }
        Ok(())
    }
    /// `GetShape(NbPoles, NbKnots, Degree, NbPoles2d)` of the first section, with no
    /// 2d curve; Standard_OutOfRange without sections; `generator.get_shape()?`.
    pub fn get_shape(&self) -> Result<[i32; 4], Failure> {
        let first: &GeomBSplineCurve = curve_at(&self.my_sequence, 1)?;
        Ok([first.nb_poles(), first.nb_knots(), first.degree(), 0])
    }
    /// `Knots(TKnots)`: the knots of the first section assigned to `knots`;
    /// `generator.knots(&mut knots)?`.
    pub fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        knots.assign(&curve_at(&self.my_sequence, 1)?.knots());
        Ok(())
    }
    /// `Mults(TMults)`: the multiplicities of the first section;
    /// `generator.mults(&mut mults)?`.
    pub fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        mults.assign(&curve_at(&self.my_sequence, 1)?.multiplicities());
        Ok(())
    }
    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: the poles
    /// and weights of section `p`; the derivatives are never computed (false);
    /// `generator.section_d1(1, &mut section)?`.
    pub fn section_d1(&self, p: i32, section: &mut AppBlendSection) -> Result<bool, Failure> {
        self.section(p, section)?;
        Ok(false)
    }
    /// `Section(P, Poles, Poles2d, Weigths)`: the poles and weights of section `p`
    /// assigned to the section arrays (the 2d poles are left unchanged);
    /// Standard_OutOfRange outside `1..=NbCurves`; `generator.section(1, &mut section)?`.
    pub fn section(&self, p: i32, section: &mut AppBlendSection) -> Result<(), Failure> {
        let curve: &GeomBSplineCurve = curve_at(&self.my_sequence, p)?;
        section.poles.assign(&curve.poles());
        curve.weights_array(|weights| section.weights.assign(weights));
        Ok(())
    }
    /// `Parameter(P)`: the imposed parameter of section `p`; a generator without
    /// `SetParam` raises Standard_NullObject (a null dereference natively), an index
    /// outside the parameter array Standard_OutOfRange; `generator.parameter(2)?`.
    pub fn parameter(&self, p: i32) -> Result<f64, Failure> {
        let array: &HArray1<f64> = self.my_params.get().ok_or_else(|| {
            errors::null_object(
                "GeomFill_SectionGenerator::Parameter: no parameters; expected SetParam",
            )
        })?;
        read_param(array, p)
    }
}

/// `myParams->Value(Index)` with the native bounds check.
fn read_param(array: &HArray1<f64>, index: i32) -> Result<f64, Failure> {
    let values = array.array1();
    values.value(index).copied().map_err(|error| {
        errors::out_of_range(&format!(
            "NCollection_Array1::Value: index {index}; {error}"
        ))
    })
}

/// `myParams->SetValue(Index, Value)` with the native bounds check.
fn write_param(array: &HArray1<f64>, index: i32, value: f64) -> Result<(), Failure> {
    array
        .change_array1()
        .set_value(index, value)
        .map_err(|error| {
            errors::out_of_range(&format!(
                "NCollection_Array1::SetValue: index {index}; {error}"
            ))
        })
}

impl TheSectionGenerator for GeomFillSectionGenerator {
    type Point = i32;
    fn get_shape(&self) -> Result<[i32; 4], Failure> {
        GeomFillSectionGenerator::get_shape(self)
    }
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
        GeomFillSectionGenerator::knots(self, knots)
    }
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
        GeomFillSectionGenerator::mults(self, mults)
    }
    fn section_d1(&mut self, p: &i32, section: &mut AppBlendSection) -> Result<bool, Failure> {
        GeomFillSectionGenerator::section_d1(self, *p, section)
    }
    fn section(&mut self, p: &i32, section: &mut AppBlendSection) -> Result<(), Failure> {
        GeomFillSectionGenerator::section(self, *p, section)
    }
    fn parameter(&self, p: &i32) -> Result<f64, Failure> {
        GeomFillSectionGenerator::parameter(self, *p)
    }
}
