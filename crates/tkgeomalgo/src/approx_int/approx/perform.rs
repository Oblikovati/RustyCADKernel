//! The `Perform` overloads: the line alone, two parametric surfaces, and a quadric with
//! a parametric surface.
use super::{ApproxIntApprox, ApproxIntApproxTypes};
use crate::approx_int::imp_prm_sv_surfaces::ApproxIntImpPrmSvSurfaces;
use crate::approx_int::prm_prm_sv_surfaces::ApproxIntPrmPrmSvSurfaces;
use crate::approx_int::surface_tool::ApproxIntSurfaceTool;
use crate::approx_int::sv_surfaces::{ApproxIntSvSurfaces, SvSurfacesRef};
use crate::int_imp::ThePSurfaceTool;
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::SurfaceType;
use std::cell::RefCell;
use std::rc::Rc;

/// If the number of points is less than this, the line is interpolated (one B-spline)
/// instead of approximated by Bezier pieces.
const MIN_NB_POINTS_FOR_APPROX: i32 = 5;

/// The surface of a pair (`ThePSurface`).
type PSurface<B> = <<B as ApproxIntApproxTypes>::SurfaceTool as ThePSurfaceTool>::Surface;

/// The approximated spaces and the range of a `Perform`: `ApproxXYZ, ApproxU1V1,
/// ApproxU2V2, indicemin, indicemax` (native defaults `true, true, true, 0, 0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApproxIntRange {
    /// `ApproxXYZ`.
    pub approx_xyz: bool,
    /// `ApproxU1V1`.
    pub approx_u1v1: bool,
    /// `ApproxU2V2`.
    pub approx_u2v2: bool,
    /// `indicemin`.
    pub indicemin: i32,
    /// `indicemax`.
    pub indicemax: i32,
}

impl Default for ApproxIntRange {
    fn default() -> Self {
        Self {
            approx_xyz: true,
            approx_u1v1: true,
            approx_u2v2: true,
            indicemin: 0,
            indicemax: 0,
        }
    }
}

/// Whether the surface kind is one of the quadrics of `Perform(Surf1, Surf2, ...)`.
fn is_quadric(typ: SurfaceType) -> bool {
    matches!(
        typ,
        SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Sphere | SurfaceType::Cone
    )
}

impl<B: ApproxIntApproxTypes> ApproxIntApprox<B> {
    /// `Perform(theline, ApproxXYZ, ApproxU1V1, ApproxU2V2, indicemin, indicemax)`: the
    /// approximation of the points of the line alone (no point is added); a line of
    /// one knot interval and more than twice `NbPntMax` points is cut in its middle;
    /// `approx.perform(&wline, ApproxIntRange { indicemin: 1, indicemax: 20, ..Default::default() })?`.
    pub fn perform(
        &mut self,
        line: &Handle<B::Line>,
        range: ApproxIntRange,
    ) -> Result<(), Failure> {
        self.prepare_ds(range);
        let nbpntbez: i32 = self.my_data.indicemax - self.my_data.indicemin;
        self.my_data.bezier_approx = nbpntbez >= MIN_NB_POINTS_FOR_APPROX;
        self.fill_data(line)?;
        self.build_knots(line, None)?;
        if self.my_knots.len() == 2
            && range.indicemax - range.indicemin > 2 * self.my_data.nb_pnt_max
        {
            // At least 3 knots for BRepApprox.
            let last: usize = self.my_knots.len() - 1;
            self.my_knots[last] = (range.indicemax - range.indicemin) / 2;
            self.my_knots.push(range.indicemax);
        }
        self.init_compute_lines(true, self.my_data.parametrization);
        self.build_curve(line, None)
    }

    /// `Perform(Surf1, Surf2, theline, ApproxXYZ, ApproxU1V1, ApproxU2V2, indicemin,
    /// indicemax)`: with a plane, cylinder, sphere or cone, the analytic-parametric
    /// approximation on that quadric (the first one when both are); else the
    /// parametric-parametric one; the SvSurfaces add points where needed;
    /// `approx.perform_on_surfaces(&s1, &s2, &wline, range)?`.
    pub fn perform_on_surfaces(
        &mut self,
        surf1: &PSurface<B>,
        surf2: &PSurface<B>,
        line: &Handle<B::Line>,
        range: ApproxIntRange,
    ) -> Result<(), Failure> {
        self.my_tol_reached3d = 0.0;
        self.my_tol_reached2d = 0.0;
        let type_s1: SurfaceType = B::SurfaceTool::get_type(surf1)?;
        let type_s2: SurfaceType = B::SurfaceTool::get_type(surf2)?;
        if is_quadric(type_s1) || is_quadric(type_s2) {
            let second_is_implicit: bool = !is_quadric(type_s1);
            let (quadric_surface, p_surf) = if second_is_implicit {
                (surf2, surf1)
            } else {
                (surf1, surf2)
            };
            let quad: IntSurfQuadric = Self::quadric(quadric_surface)?;
            return self.perform_quadric(&quad, p_surf, line, range, !second_is_implicit);
        }
        // Non-analytical case: Param-Param perform.
        self.prepare_ds(range);
        let sv = ApproxIntPrmPrmSvSurfaces::<B::SurfaceTool>::new(surf1, surf2)?;
        let sv: SvSurfacesRef = Rc::new(RefCell::new(sv));
        self.my_data.bezier_approx = range.indicemax - range.indicemin >= MIN_NB_POINTS_FOR_APPROX;
        self.fill_data(line)?;
        self.run_with(line, sv)
    }

    /// The quadric of a surface of quadric kind (`Quad.SetValue(ThePSurfaceTool::...)`).
    fn quadric(surface: &PSurface<B>) -> Result<IntSurfQuadric, Failure> {
        let mut quad = IntSurfQuadric::new();
        match B::SurfaceTool::get_type(surface)? {
            SurfaceType::Plane => quad.set_plane(&B::SurfaceTool::plane(surface)?),
            SurfaceType::Cylinder => quad.set_cylinder(&B::SurfaceTool::cylinder(surface)?),
            SurfaceType::Sphere => quad.set_sphere(&B::SurfaceTool::sphere(surface)?),
            SurfaceType::Cone => quad.set_cone(&B::SurfaceTool::cone(surface)?),
            _ => {}
        }
        Ok(quad)
    }

    /// `Perform(ISurf, PSurf, theline, ..., isTheQuadFirst)` (private natively): the
    /// analytic-parametric approximation, with the solver of the SvSurfaces off.
    pub fn perform_quadric(
        &mut self,
        i_surf: &IntSurfQuadric,
        p_surf: &PSurface<B>,
        line: &Handle<B::Line>,
        range: ApproxIntRange,
        is_the_quad_first: bool,
    ) -> Result<(), Failure> {
        self.prepare_ds(range);
        let mut sv = if is_the_quad_first {
            ApproxIntImpPrmSvSurfaces::<B::SurfaceTool>::quadric_first(i_surf, p_surf)
        } else {
            ApproxIntImpPrmSvSurfaces::<B::SurfaceTool>::quadric_second(p_surf, i_surf)
        };
        sv.set_use_solver(false);
        let sv: SvSurfacesRef = Rc::new(RefCell::new(sv));
        self.my_data.bezier_approx = range.indicemax - range.indicemin >= MIN_NB_POINTS_FOR_APPROX;
        self.fill_data(line)?;
        self.run_with(line, sv)
    }

    /// The knots, the options and the curve of a `Perform` with SvSurfaces.
    fn run_with(&mut self, line: &Handle<B::Line>, sv: SvSurfacesRef) -> Result<(), Failure> {
        let cut: bool = self.my_data.bezier_approx;
        self.build_knots(line, Some(sv.clone()))?;
        self.init_compute_lines(cut, self.my_data.parametrization);
        self.build_curve(line, Some(sv))
    }
}
