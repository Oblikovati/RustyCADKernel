// Rust translation of BlendFunc_EvolRad.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evolutive-radius fillet function between two surfaces along a guide: the radius is
//! a law of the guide parameter.
use super::SectionShape;
use super::choice::{choice_signs, live_law};
use super::guide::position_on_guide;
use super::linalg::vector;
use super::package::{live_curve, next_shape};
use super::section_solve::{TangentSolve, solve_tangents_gauss};
use super::two_surfaces::{CircleSection, Radii, SurfaceScratch, TwoSurfaceCore};
use crate::blend_func::const_rad::traits::impl_two_surface_function;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Circle3;
use rustycad_tkmath::math::Vector;
use std::cell::RefCell;

thread_local! {
    /// The native `static thread_local` scratch of `ComputeValues`, shared by every
    /// `EvolRad` of the thread (and distinct from the `ConstRad` one).
    static SCRATCH: RefCell<SurfaceScratch> = RefCell::new(SurfaceScratch::default());
}

/// `BlendFunc_EvolRad`: the fillet between two surfaces whose radius is the law `Law`
/// of the guide parameter. As natively, the sign fields are 0 until `Set(Choix)`, and the
/// third-order positioning keeps the normal derivatives of the previous computation.
///
/// ```ignore
/// let mut function = EvolRad::new(&surface1, &surface2, &guide, &law)?;
/// function.set_choix(1);
/// ```
pub struct EvolRad {
    pub(super) core: TwoSurfaceCore,
    curv: Handle<dyn Adaptor3dCurve>,
    tcurv: Handle<dyn Adaptor3dCurve>,
    fevol: Handle<dyn LawFunction>,
    tevol: Handle<dyn LawFunction>,
    sg1: f64,
    sg2: f64,
    ray: f64,
    dray: f64,
    d2ray: f64,
    lengthmin: f64,
    lengthmax: f64,
    pub(super) capture: FailureCapture,
}

/// The class name of the null-law failures.
const CLASS: &str = "BlendFunc_EvolRad";

impl EvolRad {
    /// `BlendFunc_EvolRad(S1, S2, C, Law)`; `EvolRad::new(&s1, &s2, &c, &law)?`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
        law: &Handle<dyn LawFunction>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            core: TwoSurfaceCore::new(s1, s2)?,
            curv: c.clone(),
            tcurv: c.clone(),
            fevol: law.clone(),
            tevol: law.clone(),
            sg1: 0.0,
            sg2: 0.0,
            ray: 0.0,
            dray: 0.0,
            d2ray: 0.0,
            lengthmin: f64::MAX,
            lengthmax: f64::MIN,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Choix)`: the choice of the solution, which signs the offsets of both surfaces;
    /// `function.set_choix(1)`.
    pub fn set_choix(&mut self, choix: i32) {
        self.core.choix = choix;
        (self.sg1, self.sg2) = choice_signs(choix);
    }

    /// `Set(TypeSection)`; `function.set_shape(SectionShape::Linear)`.
    pub fn set_shape(&mut self, shape: SectionShape) {
        self.core.shape = shape;
    }

    pub(super) fn radii(&self) -> Radii {
        Radii::Evolutive {
            sg1: self.sg1,
            sg2: self.sg2,
            ray: self.ray,
            dray: self.dray,
            d2ray: self.d2ray,
        }
    }

    /// `ComputeValues(X, Order, ByParam, Param)`: false for an order above 2.
    pub(super) fn compute_values(
        &mut self,
        x: &Vector<'_>,
        order: i32,
        by_param: bool,
        param: f64,
    ) -> Result<bool, Failure> {
        let mut scratch = SCRATCH.with(|cell| *cell.borrow());
        let result = self.compute_with(&mut scratch, x, order, (by_param, param));
        SCRATCH.with(|cell| *cell.borrow_mut() = scratch);
        result
    }

    fn compute_with(
        &mut self,
        scratch: &mut SurfaceScratch,
        x: &Vector<'_>,
        order: i32,
        (by_param, param): (bool, f64),
    ) -> Result<bool, Failure> {
        let state = self.core.cache_state(x, order, by_param, param)?;
        if state.x_ok && state.t_ok {
            return Ok(true);
        }
        if !state.t_ok && !self.position_guide_and_law(scratch, state.t, order, by_param)? {
            return Ok(false);
        }
        if !state.x_ok
            && !self
                .core
                .position_surfaces(&mut scratch.d3, x, order, false)?
        {
            return Ok(false);
        }
        self.core
            .equations(scratch, order, by_param, self.radii())?;
        Ok(true)
    }

    /// The positioning on the curve and the law.
    fn position_guide_and_law(
        &mut self,
        scratch: &mut SurfaceScratch,
        t: f64,
        order: i32,
        by_param: bool,
    ) -> Result<bool, Failure> {
        let t_order = self.core.begin_guide(t, order, by_param);
        if !position_on_guide(
            &self.tcurv,
            t,
            t_order,
            &mut scratch.guide,
            &mut self.core.plane,
        )? {
            return Ok(false);
        }
        let law = live_law(&self.tevol, CLASS)?;
        match t_order {
            0 => self.ray = law.value(t)?,
            1 => (self.ray, self.dray) = law.d1(t)?,
            _ => (self.ray, self.dray, self.d2ray) = law.d2(t)?,
        }
        Ok(true)
    }

    /// `IsSolution(Sol, Tol)`: the tangents by Gauss alone (pivot 1e-14), the angle range
    /// and the range of the section lengths.
    pub(super) fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let ok = self.compute_values(sol, 1, true, self.core.param)?;
        if !self.core.residual_within(tol)? {
            self.core.istangent = true;
            return Ok(false);
        }
        let solution = self.gauss_tangents(tol)?;
        self.core.apply_tangents(solution);
        let angle = self.core.current_angle(self.radii());
        self.core.record_angle(angle);
        let length = (angle * self.ray).abs();
        if length < self.lengthmin {
            self.lengthmin = length;
        }
        if length > self.lengthmax {
            self.lengthmax = length;
        }
        self.core.record_distance();
        Ok(ok)
    }

    /// The Gauss solve of `IsSolution`; the tolerances are read only when Gauss succeeds.
    fn gauss_tangents(&self, tol: f64) -> Result<Option<[f64; 4]>, Failure> {
        let gauss = rustycad_tkmath::math::Gauss::with_min_pivot(&self.core.eq.dedx, 1.0e-14)?;
        if !gauss.is_done() {
            return Ok(None);
        }
        let mut tolerances = vector(4)?;
        self.core.get_tolerance_impl(&mut tolerances, tol)?;
        let solve = TangentSolve {
            dedx: &self.core.eq.dedx,
            dedt: &self.core.eq.dedt,
            tolerances: &tolerances,
            max_pivot: 1.0e-14,
        };
        solve_tangents_gauss(&gauss, &solve)
    }

    /// `Section(Param, U1, V1, U2, V2, Pdeb, Pfin, C)`;
    /// `let (pdeb, pfin, circle) = function.section_circle(t, [u1, v1, u2, v2])?;`.
    pub fn section_circle(
        &mut self,
        param: f64,
        uv: [f64; 4],
    ) -> Result<(f64, f64, Circle3), Failure> {
        self.compute_values(&TwoSurfaceCore::uv_vector(uv)?, 0, true, param)?;
        let CircleSection { pdeb, pfin, circle } = self.core.section_circle_impl(self.radii())?;
        Ok((pdeb, pfin, circle))
    }

    /// `Set(First, Last)`: the useful parts of the guide and of the law.
    pub(super) fn set_interval_impl(&mut self, first: f64, last: f64) -> Result<(), Failure> {
        self.tcurv = live_curve(&self.curv)?.trim(first, last, 1.0e-12)?;
        self.tevol = live_law(&self.fevol, CLASS)?.trim(first, last, 1.0e-12)?;
        Ok(())
    }

    /// The `GeomFill::GetTolerance` angle and radius: `maxang` and `lengthmin / maxang`
    /// ("a radius is subtracted").
    pub(super) fn tolerance_arc(&self) -> (f64, f64) {
        (self.core.maxang, self.lengthmin / self.core.maxang)
    }

    /// `GetSectionSize()`: the largest section length.
    pub(super) fn section_size(&self) -> f64 {
        self.lengthmax
    }

    /// The native (empty) message of a tangent query on a tangency point.
    fn tangent_message(&self, _member: &str) -> String {
        String::new()
    }

    /// The intervals of the guide (continuity `S + 1`) merged with those of the law.
    fn merged_intervals(&self, shape: Shape) -> Result<Option<Vec<f64>>, Failure> {
        let curve = live_curve(&self.curv)?;
        let law = live_law(&self.fevol, CLASS)?;
        let nb_curve = curve.nb_intervals(next_shape(shape))?;
        let nb_law = law.nb_intervals(shape)?;
        if nb_law == 1 {
            return Ok(None);
        }
        let mut int_c = Array1::with_bounds(1, nb_curve + 1).map_err(range)?;
        let mut int_l = Array1::with_bounds(1, nb_law + 1).map_err(range)?;
        curve.intervals(&mut int_c, next_shape(shape))?;
        law.intervals(&mut int_l, shape)?;
        fusionne_intervalles(&int_c, &int_l).map(Some)
    }

    fn nb_intervals_impl(&self, shape: Shape) -> Result<i32, Failure> {
        match self.merged_intervals(shape)? {
            Some(inter) => Ok(inter.len() as i32 - 1),
            None => live_curve(&self.curv)?.nb_intervals(next_shape(shape)),
        }
    }

    fn intervals_impl(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let Some(inter) = self.merged_intervals(shape)? else {
            return live_curve(&self.curv)?.intervals(bounds, next_shape(shape));
        };
        for (ii, value) in (1..).zip(inter) {
            put(bounds, ii, value)?;
        }
        Ok(())
    }
}

fn range(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    Failure::with_message("Standard_RangeError", &format!("{error:?}"))
}

/// `FusionneIntervalles(I1, I2, Seq)`: the sorted union of two sorted parameter tables,
/// values closer than `0.99 * PConfusion` merged into their middle.
fn fusionne_intervalles(i1: &Array1<'_, f64>, i2: &Array1<'_, f64>) -> Result<Vec<f64>, Failure> {
    let (mut ind1, mut ind2) = (1, 1);
    let epspar = precision::p_confusion() * 0.99;
    let mut seq = Vec::new();
    while ind1 <= i1.upper() && ind2 <= i2.upper() {
        let (v1, v2) = (at(i1, ind1)?, at(i2, ind2)?);
        if (v1 - v2).abs() <= epspar {
            seq.push((v1 + v2) / 2.);
            ind1 += 1;
            ind2 += 1;
        } else if v1 < v2 {
            seq.push(v1);
            ind1 += 1;
        } else {
            seq.push(v2);
            ind2 += 1;
        }
    }
    if ind1 > i1.upper() {
        for ind in ind2..=i2.upper() {
            seq.push(at(i2, ind)?);
        }
    }
    if ind2 > i2.upper() {
        for ind in ind1..=i1.upper() {
            seq.push(at(i1, ind)?);
        }
    }
    Ok(seq)
}

impl_two_surface_function!(EvolRad);

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1-based array of `values`, as the native intervals are.
    fn one_based(values: &[f64]) -> Array1<'static, f64> {
        let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
        for (index, value) in (1..).zip(values) {
            put(&mut array, index, *value).unwrap();
        }
        array
    }

    #[test]
    fn merged_intervals_fuse_close_values() {
        let (a, b) = (
            one_based(&[0., 1., 2.]),
            one_based(&[0., 1.0 + 1.0e-10, 1.5, 2.]),
        );
        let merged = fusionne_intervalles(&a, &b).unwrap();
        assert_eq!(merged, vec![0., (1. + 1.0 + 1.0e-10) / 2., 1.5, 2.]);
    }

    #[test]
    fn choice_signs_both_offsets() {
        let null_surface: Handle<dyn Adaptor3dSurface> = Handle::default();
        let null_curve: Handle<dyn Adaptor3dCurve> = Handle::default();
        let null_law: Handle<dyn LawFunction> = Handle::default();
        let mut function =
            EvolRad::new(&null_surface, &null_surface, &null_curve, &null_law).unwrap();
        function.set_choix(3);
        assert_eq!((function.sg1, function.sg2), (1., -1.));
        assert_eq!(function.tangent_message("TangentOnS1"), "");
    }
}
