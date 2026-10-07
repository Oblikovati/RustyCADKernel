// Rust translation of BlendFunc_ConstRad.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constant-radius fillet function between two surfaces along a guide.
use super::SectionShape;
use super::choice::signed_radii;
use super::guide::position_on_guide;
use super::linalg::vector;
use super::package::live_curve;
use super::section_solve::{TangentSolve, solve_tangents};
use super::two_surfaces::{CircleSection, Radii, SurfaceScratch, TwoSurfaceCore};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{self, Axis1, Circle3, Direction3, Point3, Vector3};
use rustycad_tkmath::math::Vector;
use std::cell::RefCell;

thread_local! {
    /// The native `static thread_local` scratch of `ComputeValues`, shared by every
    /// `ConstRad` of the thread.
    static SCRATCH: RefCell<SurfaceScratch> = RefCell::new(SurfaceScratch::default());
}

/// `BlendFunc_ConstRad`: the fillet of constant radius between two surfaces. The
/// variables are `(U1, V1, U2, V2)`; the four equations put the middle of the two contact
/// points in the plane normal to the guide and make the two points the ends of a circular
/// section of the radius, centred on both surface offsets.
///
/// `ComputeValues` caches its last point and parameter; as natively, the guide values and
/// the third surface derivatives are thread-wide scratch shared by every `ConstRad`.
///
/// ```ignore
/// let mut function = ConstRad::new(&surface1, &surface2, &guide)?;
/// function.set_radius(2.0, 1);
/// ```
pub struct ConstRad {
    pub(super) core: TwoSurfaceCore,
    curv: Handle<dyn Adaptor3dCurve>,
    tcurv: Handle<dyn Adaptor3dCurve>,
    ray1: f64,
    ray2: f64,
    pub(super) capture: FailureCapture,
}

impl ConstRad {
    /// `BlendFunc_ConstRad(S1, S2, C)`: the function on the surfaces `s1`, `s2` along the
    /// guide `c`, radius 0, rational sections; `ConstRad::new(&s1, &s2, &c)?`.
    pub fn new(
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        c: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Self, Failure> {
        Ok(Self {
            core: TwoSurfaceCore::new(s1, s2)?,
            curv: c.clone(),
            tcurv: c.clone(),
            ray1: 0.0,
            ray2: 0.0,
            capture: FailureCapture::default(),
        })
    }

    /// `Set(Radius, Choix)`: the radius and the choice of the solution, which signs the
    /// offsets of both surfaces; `function.set_radius(2.0, 1)`.
    pub fn set_radius(&mut self, radius: f64, choix: i32) {
        self.core.choix = choix;
        (self.ray1, self.ray2) = signed_radii(radius, choix);
    }

    /// `Set(TypeSection)`: the shape of the sections; `function.set_shape(SectionShape::Linear)`.
    pub fn set_shape(&mut self, shape: SectionShape) {
        self.core.shape = shape;
    }

    pub(super) fn radii(&self) -> Radii {
        Radii::Constant {
            ray1: self.ray1,
            ray2: self.ray2,
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
        if !state.t_ok {
            let t_order = self.core.begin_guide(state.t, order, by_param);
            if !position_on_guide(
                &self.tcurv,
                state.t,
                t_order,
                &mut scratch.guide,
                &mut self.core.plane,
            )? {
                return Ok(false);
            }
        }
        if !state.x_ok
            && !self
                .core
                .position_surfaces(&mut scratch.d3, x, order, true)?
        {
            return Ok(false);
        }
        self.core
            .equations(scratch, order, by_param, self.radii())?;
        Ok(true)
    }

    /// `IsSolution(Sol, Tol)`: whether `sol` zeroes the equations within `tol`; for a
    /// solution, the tangents (Gauss with pivot 1e-9, then SVD) and the angle range.
    pub(super) fn is_solution_impl(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let ok = self.compute_values(sol, 1, true, self.core.param)?;
        if !self.core.residual_within(tol)? {
            self.core.istangent = true;
            return Ok(false);
        }
        let mut tolerances = vector(4)?;
        self.core.get_tolerance_impl(&mut tolerances, tol)?;
        let solve = TangentSolve {
            dedx: &self.core.eq.dedx,
            dedt: &self.core.eq.dedt,
            tolerances: &tolerances,
            max_pivot: 1.0e-9,
        };
        let solution = solve_tangents(&solve)?;
        self.core.apply_tangents(solution);
        let angle = self.core.current_angle(self.radii());
        self.core.record_angle(angle);
        self.core.record_distance();
        Ok(ok)
    }

    /// `Section(Param, U1, V1, U2, V2, Pdeb, Pfin, C)`: the circular section at `param`
    /// through the two points, with its parameter range;
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

    /// `AxeRot(Prm)`: the axis of the osculating circle of the guide at `prm` (along the
    /// plane normal when the guide is straight); `function.axe_rot(0.5)?`.
    pub fn axe_rot(&self, prm: f64) -> Result<Axis1, Failure> {
        let (ptgui, d1gui, d2gui) = live_curve(&self.curv)?.d2(prm)?;
        let normtg = d1gui.magnitude();
        let np = d1gui.normalized()?;
        let mut dnp = Vector3::default();
        dnp.set_linear_form(1. / normtg, d2gui, -1. / normtg * np.dot(d2gui), np);
        let dirax = np.crossed(dnp);
        let mut axrot = Axis1::default();
        if dirax.magnitude() >= gp::resolution() {
            axrot.set_direction(Direction3::from_vector(dirax)?);
        } else {
            axrot.set_direction(Direction3::from_vector(np)?); // To avoid stop
        }
        let oriax: Point3 = if dnp.magnitude() >= gp::resolution() {
            super::linalg::point_from(
                ptgui.xyz() + (normtg / dnp.magnitude()) * *dnp.normalized()?.xyz(),
            )
        } else {
            super::linalg::point_from(ptgui.xyz())
        };
        axrot.set_location(oriax);
        Ok(axrot)
    }

    pub(super) fn guide(&self) -> &Handle<dyn Adaptor3dCurve> {
        &self.curv
    }

    /// `Set(First, Last)`: the segmentation of the useful part of the curve ("precision
    /// is taken at random and small").
    pub(super) fn set_interval_impl(&mut self, first: f64, last: f64) -> Result<(), Failure> {
        self.tcurv = live_curve(&self.curv)?.trim(first, last, 1.0e-12)?;
        Ok(())
    }

    /// The `GeomFill::GetTolerance` angle and radius of `GetTolerance(BoundTol, ...)`.
    pub(super) fn tolerance_arc(&self) -> (f64, f64) {
        (self.core.minang, self.ray1.abs())
    }

    /// `GetSectionSize()`.
    pub(super) fn section_size(&self) -> f64 {
        self.core.maxang * self.ray1.abs()
    }
}

#[path = "const_rad_traits.rs"]
pub(crate) mod traits;

#[cfg(test)]
#[path = "const_rad_tests.rs"]
mod tests;
