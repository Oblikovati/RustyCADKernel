// Rust translation of ComputeSurfInit (the inertia plane, the metric check and the
// approximated first plate), ComputeAnisotropie and IsOrderG1 of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The computation of the initial surface when none is given.
use super::GeomPlateBuildPlateSurface;
use super::courbe_jointive::trier_tab;
use crate::geom_plate::build_average_plane::GeomPlateBuildAveragePlane;
use crate::geom_plate::live;
use crate::geom_plate::make_approx::GeomPlateMakeApprox;
use crate::geom_plate::surface::GeomPlateSurface;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, errors};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::proj_lib::CompProjectedCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The samples per curve of the inertia plane and of the metric check.
const NB_POINT: i32 = 20;
/// The bounds of the ratio of a curve speed to its projection speed (`R1`, `R2`).
const RATIO_MAX: f64 = 2.0;
const RATIO_MIN: f64 = 0.6;

impl GeomPlateBuildPlateSurface {
    /// The anisotropy of the plate: 1 either way, as natively ("Temporary")
    /// (`ComputeAnisotropie`).
    #[allow(clippy::if_same_then_else)] // The native "Temporary" branches.
    pub(super) fn compute_anisotropie(&self) -> f64 {
        if self.anisotropie { 1.0 } else { 1.0 }
    }

    /// Whether every curve constraint is of order G1 or more (`IsOrderG1`).
    pub(super) fn is_order_g1(&self) -> Result<bool, Failure> {
        for i in 1..=self.nt_lin_cont() {
            if self.lin(i)?.order() < 1 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The initial surface: the average plane `bap` trimmed to three times the box of
    /// the projected points.
    pub(super) fn set_surf_init_from_plane(
        &mut self,
        bap: &GeomPlateBuildAveragePlane,
    ) -> Result<(), Failure> {
        let (mut u1, mut u2, mut v1, mut v2) = bap.min_max_box();
        let (du, dv) = (u2 - u1, v2 - v1);
        u1 -= du;
        u2 += du;
        v1 -= dv;
        v2 += dv;
        let plane: Handle<dyn GeomSurface> = bap.plane()?.cast();
        let trimmed = GeomRectangularTrimmedSurface::new(&plane, (u1, u2), (v1, v2), (true, true))?;
        self.surf_init = Handle::new(trimmed).cast();
        Ok(())
    }

    /// The initial surface when none is given (`ComputeSurfInit`): the plane of the
    /// best normal for a closed G1 contour whose normals lie in a half space, else the
    /// inertia or flux plane of the sampled constraints; when the curves do not map
    /// linearly onto it, the approximation of a first degree 2 plate on it.
    pub(super) fn compute_surf_init(
        &mut self,
        the_progress: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        let (nt_lin_cont, mut nopt) = (self.nt_lin_cont(), 2);
        if nt_lin_cont != 0 {
            self.init_order = Some((1..=nt_lin_cont).collect());
        }
        let courbe_joint: bool = nt_lin_cont != 0 && self.courbe_jointive(self.tol3d)?;
        if courbe_joint && self.is_order_g1()? {
            nopt = 3;
            if !self.surf_init_from_normals()? {
                self.is_linear = false;
                nopt = 2;
            }
        }
        if let Some(order) = self.init_order.as_mut().filter(|_| nt_lin_cont != 0) {
            trier_tab(order)?;
        }
        if nopt != 3 && !self.surf_init_from_inertia(courbe_joint)? {
            return Ok(());
        }
        if nt_lin_cont != 0 && self.is_linear {
            self.check_metrics()?;
        }
        if !self.is_linear {
            self.first_plate(the_progress)?;
        }
        Ok(())
    }

    /// The inertia (`nopt` 1) or flux (`nopt` 2) plane of 20 samples per curve and
    /// the point constraints; false when the samples give no plane.
    fn surf_init_from_inertia(&mut self, courbe_joint: bool) -> Result<bool, Failure> {
        let nt_lin_cont: i32 = self.nt_lin_cont();
        let mut nopt: i32 = 2;
        if self.nt_pnt_cont() != 0 || !courbe_joint || nt_lin_cont != self.nb_bounds {
            nopt = 1;
        }
        for i in 1..=nt_lin_cont {
            // The native code sums the lengths for a point count it never uses; the
            // calls stay for their exceptions.
            self.lin(i)?.length()?;
        }
        let pts: HArray1<Point3> =
            HArray1::with_bounds(1, NB_POINT * nt_lin_cont + self.nt_pnt_cont())
                .map_err(errors::array_bounds)?;
        let mut np: i32 = 1;
        for i in 1..=nt_lin_cont {
            let (uinit, uif) = self.sensed_range(i)?;
            for j in 0..NB_POINT {
                let inter: f64 = f64::from(j) * uif / f64::from(NB_POINT);
                let p: Point3 = self.lin(i)?.d0(uinit + inter)?;
                pts.change_array1()
                    .set_value(np, p)
                    .map_err(errors::array_bounds)?;
                np += 1;
            }
        }
        for i in 1..=self.nt_pnt_cont() {
            pts.change_array1()
                .set_value(np, self.pnt(i)?.d0())
                .map_err(errors::array_bounds)?;
            np += 1;
        }
        if !courbe_joint {
            self.nb_bounds = 0;
        }
        let pts = Handle::new(pts);
        let bap = GeomPlateBuildAveragePlane::new(
            &pts,
            NB_POINT * self.nb_bounds,
            self.tol3d / 1000.0,
            2,
            nopt,
        )?;
        if !bap.is_plane() {
            return Ok(false);
        }
        self.set_surf_init_from_plane(&bap)?;
        Ok(true)
    }

    /// Whether the speed of curve `i` stays within `[0.6, 2]` times the speed of its
    /// projection on `hsur` at 19 inner samples.
    fn curve_is_linear(
        &self,
        i: i32,
        hsur: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<bool, Failure> {
        let lin = self.lin(i)?;
        let (first_par, last_par) = (lin.first_parameter()?, lin.last_parameter()?);
        let uif: f64 = (last_par - first_par) / f64::from(NB_POINT);
        let curve: Handle<dyn Adaptor3dCurve> = lin.curve3d();
        let proj = CompProjectedCurve::with_tolerances(hsur, &curve, self.tol3d, self.tol3d)?;
        let proj_curve: Handle<dyn Adaptor2dCurve2d> = Handle::new(proj).cast();
        let a_proj = Adaptor3dCurveOnSurface::with_curve(&proj_curve, hsur)?;
        let curve = live(&curve, "the curve of a constraint")?;
        for j in 1..NB_POINT {
            let inter: f64 = first_par + f64::from(j) * uif;
            let (_, der_c): (Point3, Vector3) = curve.d1(inter)?;
            let (_, der_c_proj): (Point3, Vector3) = a_proj.d1(inter)?;
            let (a1, a2) = (der_c.magnitude(), der_c_proj.magnitude());
            let ratio: f64 = if a2 <= 1.0e-20 { 1.0e20 } else { a1 / a2 };
            if ratio > RATIO_MAX || ratio < RATIO_MIN {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Compares the metrics of the curves and of their projections on the plane of
    /// the initial surface; a curve out of the ratio bounds makes the plate
    /// non-linear.
    fn check_metrics(&mut self) -> Result<(), Failure> {
        let trimmed: Handle<GeomRectangularTrimmedSurface> = self.surf_init.cast();
        let init_plane = live(&trimmed, "the initial trimmed plane")?.basis_surface();
        let hsur: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&init_plane)?).cast();
        for i in 1..=self.nt_lin_cont() {
            if !self.curve_is_linear(i, &hsur)? {
                self.is_linear = false;
                break;
            }
        }
        Ok(())
    }

    /// Projects the curves without 2d curve and the points without parameter on the
    /// initial surface (the start of the non-linear branch).
    fn project_constraints(&mut self) -> Result<(), Failure> {
        for i in 1..=self.nt_lin_cont() {
            if self.lin(i)?.curve2d_on_surf()?.is_null() {
                let curve2d = self.project_curve(&self.lin(i)?.curve3d())?;
                self.lin(i)?.set_curve2d_on_surf(&curve2d);
            }
        }
        for i in 1..=self.nt_pnt_cont() {
            let p: Point3 = self.pnt(i)?.d0();
            if !self.pnt(i)?.has_pnt2d_on_surf() {
                let p2d = self.project_point(p)?;
                self.pnt(i)?.set_pnt2d_on_surf(p2d);
            }
        }
        Ok(())
    }

    /// The non-linear branch: a degree 2 plate of the G0 constraints on the plane,
    /// approximated by a B-spline that becomes the initial surface; the 2d curves are
    /// then cleared for a new projection.
    fn first_plate(&mut self, the_progress: &ProgressRange<'_>) -> Result<(), Failure> {
        self.planar_surf_init = self.surf_init.clone();
        self.init_projector()?;
        self.project_constraints()?;
        let (pnt_inter, pnt_g1g1) = self.prepare_intersections()?;
        self.discretise(&pnt_inter, &pnt_g1g1)?;
        self.load_curve(0, 0)?;
        if !self.pnt_cont.is_empty() {
            self.load_point(0, 0)?;
        }
        let scope = ProgressScope::with_literal(the_progress, "ComputeSurfInit", 1.0, false);
        let anisotropie: f64 = self.compute_anisotropie();
        self.plate.solve_ti(2, anisotropie, &scope.next(1.0))?;
        if the_progress.user_break() || !self.plate.is_done() {
            return Ok(());
        }
        let surface = Handle::new(GeomPlateSurface::new(&self.surf_init, &self.plate));
        self.geom_plate_surface = surface.clone();
        let app = GeomPlateMakeApprox::with_order(
            &surface,
            self.tol3d,
            1,
            3,
            15.0 * self.tol3d,
            -1,
            Shape::C0,
            1.1,
        )?;
        self.surf_init = app.surface().cast();
        self.surf_init_is_give = true;
        self.plate.init();
        let null_curve: Handle<dyn Geom2dCurve> = Handle::default();
        for i in 1..=self.nt_lin_cont() {
            self.lin(i)?.set_curve2d_on_surf(&null_curve);
        }
        Ok(())
    }
}
