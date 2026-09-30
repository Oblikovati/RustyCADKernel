// Rust translation of Perform of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_BuildPlateSurface::Perform`.
use super::GeomPlateBuildPlateSurface;
use super::courbe_jointive::trier_tab;
use crate::geom_plate::make_approx::GeomPlateMakeApprox;
use crate::geom_plate::surface::GeomPlateSurface;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// Whether `Perform` goes on after a step: false when the user breaks.
type Continue = bool;

impl GeomPlateBuildPlateSurface {
    /// The contour order when the initial surface is given: the curves chained for
    /// two curves or more, the identity for one.
    fn order_given_contour(&mut self) -> Result<(), Failure> {
        let nt_lin_cont: i32 = self.nt_lin_cont();
        if nt_lin_cont >= 2 {
            self.init_order = Some((1..=nt_lin_cont).collect());
            // A contour that is not closed within Tol3d is only a debug warning natively.
            self.courbe_jointive(self.tol3d)?;
            if let Some(order) = self.init_order.as_mut() {
                trier_tab(order)?;
            }
        } else if nt_lin_cont > 0 {
            self.sense = Some(vec![0; nt_lin_cont as usize]);
            self.init_order = Some(vec![1; nt_lin_cont as usize]);
        }
        Ok(())
    }

    /// Projects the curves without 2d curve; false when one has no continuous
    /// projection.
    fn project_missing_curves(&mut self) -> Result<bool, Failure> {
        for i in 1..=self.nt_lin_cont() {
            if self.lin(i)?.curve2d_on_surf()?.is_some() {
                continue;
            }
            let curve2d = self.project_curve(&self.lin(i)?.curve3d())?;
            if curve2d.is_null() {
                return Ok(false);
            }
            self.lin(i)?.set_curve2d_on_surf(&curve2d);
        }
        Ok(true)
    }

    /// Projects every curve again; false when one has no continuous projection.
    fn project_all_curves(&mut self) -> Result<bool, Failure> {
        for i in 1..=self.nt_lin_cont() {
            let curve2d = self.project_curve(&self.lin(i)?.curve3d())?;
            if curve2d.is_null() {
                return Ok(false);
            }
            self.lin(i)?.set_curve2d_on_surf(&curve2d);
        }
        Ok(true)
    }

    /// Projects the points on the initial surface; only those without parameter
    /// unless `all`.
    fn project_points(&mut self, all: bool) -> Result<(), Failure> {
        for i in 1..=self.nt_pnt_cont() {
            if !all && self.pnt(i)?.has_pnt2d_on_surf() {
                continue;
            }
            let p: Point3 = self.pnt(i)?.d0();
            let p2d = self.project_point(p)?;
            self.pnt(i)?.set_pnt2d_on_surf(p2d);
        }
        Ok(())
    }

    /// The fallback when a curve has no continuous projection: the approximation of
    /// the current plate surface becomes the initial surface (Standard_NullObject
    /// when there is none: a null dereference natively), then the planar one when
    /// the curves still do not project.
    fn reproject_on_approximation(&mut self) -> Result<(), Failure> {
        let app = GeomPlateMakeApprox::with_order(
            &self.geom_plate_surface,
            self.tol3d,
            1,
            3,
            15.0 * self.tol3d,
            -1,
            Shape::C0,
            1.3,
        )?;
        self.surf_init = app.surface().cast();
        self.init_projector()?;
        if self.project_all_curves()? {
            return self.project_points(true);
        }
        self.surf_init = self.planar_surf_init.clone();
        self.init_projector()?;
        for i in 1..=self.nt_lin_cont() {
            let curve2d = self.project_curve(&self.lin(i)?.curve3d())?;
            self.lin(i)?.set_curve2d_on_surf(&curve2d);
        }
        Ok(())
    }

    /// Solves the plate of the loaded constraints and builds the plate surface on its
    /// constraint box; `None` when the user breaks or the plate fails.
    fn solve_plate(&mut self, scope: &ProgressScope<'_>) -> Result<Option<Continue>, Failure> {
        let anisotropie: f64 = self.compute_anisotropie();
        self.plate
            .solve_ti(self.degree, anisotropie, &scope.next(90.0))?;
        if scope.user_break() || !self.plate.is_done() {
            return Ok(None);
        }
        let surface = Handle::new(GeomPlateSurface::new(&self.surf_init, &self.plate));
        self.geom_plate_surface = surface.clone();
        let (umin, umax, vmin, vmax) = self.plate.uv_box()?;
        crate::geom_plate::live(&surface, "the plate surface")?
            .set_bounds(umin, umax, vmin, vmax)?;
        Ok(Some(true))
    }

    /// One loop over the curve constraints: discretise, load, solve and check; `None`
    /// to stop, else whether the result is final.
    fn curve_loop(
        &mut self,
        nb_boucle: i32,
        zones: &super::intersect::ZonesPerCurve,
        g1: &super::intersect::ZonesPerCurve,
        scope: &ProgressScope<'_>,
    ) -> Result<Option<bool>, Failure> {
        self.discretise(zones, g1)?;
        self.load_curve(nb_boucle, 2)?;
        if !self.pnt_cont.is_empty() {
            self.load_point(nb_boucle, 2)?;
        }
        if self.solve_plate(scope)?.is_none() {
            return Ok(None);
        }
        let mut fini: bool = self.verif_surface(nb_boucle)?;
        if nb_boucle >= self.nb_iter && !fini {
            // "objective was not reached": a debug warning natively.
            fini = true;
        }
        if !self.pnt_cont.is_empty() && fini {
            self.verif_points()?;
        }
        Ok(Some(fini))
    }

    /// The single loop over the point constraints alone.
    fn point_loop(
        &mut self,
        nb_boucle: i32,
        scope: &ProgressScope<'_>,
    ) -> Result<Option<bool>, Failure> {
        self.load_point(nb_boucle, 2)?;
        if self.solve_plate(scope)?.is_none() {
            return Ok(None);
        }
        self.verif_points()?;
        Ok(Some(true))
    }

    /// The initial surface and the projections before the loops; false to stop.
    fn prepare(&mut self, scope: &ProgressScope<'_>) -> Result<Continue, Failure> {
        if !self.surf_init_is_give {
            self.compute_surf_init(&scope.next(10.0))?;
            if scope.user_break() {
                return Ok(false);
            }
        } else {
            self.order_given_contour()?;
        }
        if self.surf_init.is_null() {
            return Ok(false);
        }
        self.init_projector()?;
        if !self.project_missing_curves()? {
            self.reproject_on_approximation()?;
        }
        self.project_points(false)?;
        Ok(true)
    }

    /// Computes the plate surface of the loaded constraints (`Perform`): the initial
    /// surface (computed when not given), the projections, then plates of degree
    /// `Degree` on more and more points until the curve constraints are met or
    /// `NbIter` loops ran. Without constraint nothing is computed and the builder is
    /// done (the plate is reset). `builder.perform(&ProgressRange::default())?`.
    pub fn perform(&mut self, the_progress: &ProgressRange<'_>) -> Result<(), Failure> {
        self.geom_plate_surface = Handle::default();
        if self.nb_bounds == 0 {
            self.nb_bounds = self.nt_lin_cont();
        }
        self.plate.init();
        if self.nt_lin_cont() + self.nt_pnt_cont() == 0 {
            return Ok(());
        }
        let scope = ProgressScope::with_literal(
            the_progress,
            "Calculating the surface filled",
            100.0,
            true,
        );
        if !self.prepare(&scope)? {
            return Ok(());
        }
        let (zones, g1) = self.prepare_intersections()?;
        self.free = !self.is_linear;
        let mut nb_boucle: i32 = 0;
        loop {
            nb_boucle += 1;
            let step = if self.lin_cont.is_empty() {
                self.point_loop(nb_boucle, &scope)?
            } else {
                self.curve_loop(nb_boucle, &zones, &g1, &scope)?
            };
            match step {
                None => return Ok(()),
                Some(true) => return Ok(()),
                Some(false) => {}
            }
        }
    }
}
