//! FaceFace orchestration preserves native face ordering, persistent seeds and result lifetimes.
use super::*;
use super::{analytic, boundaries, planes::PlaneIntersection, surface_policy};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeomalgo::{
    geom_int::GeomIntLineConstructor,
    int_patch::{IntPatchIntersection, IntPatchIntersectionOptions},
    int_start::TheTopolTool,
};
use std::rc::Rc;
#[derive(Default)]
pub(super) struct IntersectionKernel {
    pub(super) intersection: IntPatchIntersection,
    pub(super) constructor: GeomIntLineConstructor,
    pub(super) face_tolerances: [f64; 2],
}
impl FaceFace {
    /// Intersects both complete face domains with native approximation policy; `ff.perform(&first, &second, false)?`.
    pub fn perform(&mut self, first: &Face, second: &Face, parallel: bool) -> Result<(), Failure> {
        self.reset_faces(first, second);
        self.perform_faces(first, second, parallel)
    }
    fn perform_faces(
        &mut self,
        first: &Face,
        second: &Face,
        parallel: bool,
    ) -> Result<(), Failure> {
        let adaptors = [
            self.context.surface_adaptor(first)?,
            self.context.surface_adaptor(second)?,
        ];
        let types = [
            adaptors[0].read().get_type()?,
            adaptors[1].read().get_type()?,
        ];
        let reverse = surface_policy::reverse_surfaces(types[0], types[1]);
        self.load_ordered_surfaces(types, reverse)?;
        if types == [SurfaceType::Plane; 2] {
            return self.intersect_planes();
        }
        let domains = self.intersection_domains()?;
        let analytic = analytic::treat_analytically(
            &*adaptors[0].read(),
            &*adaptors[1].read(),
            self.tolerance(),
        )?;
        self.perform_domains(&domains, first.is_same(second), analytic, reverse, parallel)
    }
    fn load_ordered_surfaces(
        &mut self,
        types: [SurfaceType; 2],
        reverse: bool,
    ) -> Result<(), Failure> {
        if reverse {
            self.reverse_inputs();
        }
        self.load_surfaces(if reverse { [types[1], types[0]] } else { types })
    }
    fn perform_domains(
        &mut self,
        domains: &[Rc<dyn TheTopolTool>; 2],
        same: bool,
        analytic: bool,
        reverse: bool,
        parallel: bool,
    ) -> Result<(), Failure> {
        self.intersect_pair(domains, same, analytic)?;
        self.done = self.kernel.intersection.is_done();
        self.complete_intersection(domains, reverse, parallel)
    }
    fn reset_faces(&mut self, first: &Face, second: &Face) {
        if self.context.is_null() {
            self.context = Handle::new(Context::default());
        }
        self.curves.clear();
        self.done = false;
        self.faces = [first.clone(), second.clone()];
        // Points and tangent state intentionally survive until the native completion path.
    }
    fn reverse_inputs(&mut self) {
        self.faces.swap(0, 1);
        self.approximation.swap(1, 2);
        for point in &mut self.seeds {
            let (u1, v1, u2, v2) = point.parameters();
            point.set_parameters(u2, v2, u1, v1);
        }
    }
    pub(super) fn tolerance(&self) -> f64 {
        self.kernel.face_tolerances[0] + self.kernel.face_tolerances[1]
    }
    fn load_surfaces(&mut self, kinds: [SurfaceType; 2]) -> Result<(), Failure> {
        let geometry = [
            BRepTool::surface(&self.faces[0])?,
            BRepTool::surface(&self.faces[1])?,
        ];
        let fuzz = self.fuzzy / 2.;
        self.kernel.face_tolerances = [
            BRepTool::face_tolerance(&self.faces[0])? + fuzz,
            BRepTool::face_tolerance(&self.faces[1])? + fuzz,
        ];
        for index in [0, 1] {
            let bounds = self.intersection_bounds(index, kinds)?;
            self.surfaces[index].load_bounds(
                &geometry[index],
                (bounds[0], bounds[1], bounds[2], bounds[3]),
            )?;
        }
        Ok(())
    }
    fn intersection_bounds(
        &self,
        index: usize,
        kinds: [SurfaceType; 2],
    ) -> Result<[f64; 4], Failure> {
        let (u1, u2, v1, v2) = self.context.uv_bounds(&self.faces[index])?;
        let mut bounds = [u1, u2, v1, v2];
        if kinds == [SurfaceType::Plane; 2] {
            return Ok(bounds);
        }
        let other_quadric = matches!(
            kinds[1 - index],
            SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Torus
        );
        if kinds[index] == SurfaceType::Plane && other_quadric {
            boundaries::correct_plane(&mut bounds);
        } else {
            boundaries::correct_surface(&self.faces[index], 2. * self.tolerance(), &mut bounds)?;
        }
        Ok(bounds)
    }
    fn intersect_planes(&mut self) -> Result<(), Failure> {
        let planes = PlaneIntersection {
            surfaces: &self.surfaces,
            face_tolerances: self.kernel.face_tolerances,
            angular: 1e-8,
            tangential: self.tolerance(),
            approximation: [self.approximation[1], self.approximation[2]],
        };
        planes.perform(&mut self.curves, &mut self.tangent)?;
        self.done = true;
        Ok(())
    }
    fn intersection_domains(&mut self) -> Result<[Rc<dyn TheTopolTool>; 2], Failure> {
        let domains: [Rc<dyn TheTopolTool>; 2] = [
            Rc::new(super::super::TopolTool::with_surface(
                &self.surfaces[0].clone().cast(),
            )?),
            Rc::new(super::super::TopolTool::with_surface(
                &self.surfaces[1].clone().cast(),
            )?),
        ];
        self.kernel.constructor.load(
            domains[0].clone(),
            domains[1].clone(),
            &self.surfaces[0],
            &self.surfaces[1],
        );
        self.intersection_tolerances(&domains)?;
        self.discard_torus_seeds()?;
        Ok(domains)
    }
    fn intersection_tolerances(
        &mut self,
        domains: &[Rc<dyn TheTopolTool>; 2],
    ) -> Result<(), Failure> {
        let tangential = surface_policy::tangential_tolerance(
            &self.surfaces[0],
            &self.surfaces[1],
            self.tolerance(),
        )?;
        let first = self.surfaces[0].clone().cast();
        let second = self.surfaces[1].clone().cast();
        let step =
            IntPatchIntersection::define_uv_max_step(&first, &*domains[0], &second, &*domains[1])?;
        let both_splines = self.surfaces[0].get_type()? == SurfaceType::BSplineSurface
            && self.surfaces[1].get_type()? == SurfaceType::BSplineSurface;
        let deflection = if both_splines { 0.1 / 10. } else { 0.1 };
        self.kernel
            .intersection
            .set_tolerances(self.tolerance(), tangential, step, deflection);
        Ok(())
    }
    fn discard_torus_seeds(&mut self) -> Result<(), Failure> {
        let kinds = [self.surfaces[0].get_type()?, self.surfaces[1].get_type()?];
        let freeform = |kind| {
            matches!(
                kind,
                SurfaceType::BSplineSurface
                    | SurfaceType::BezierSurface
                    | SurfaceType::OtherSurface
            )
        };
        if !freeform(kinds[0]) && !freeform(kinds[1]) && kinds.contains(&SurfaceType::Torus) {
            self.seeds.clear();
        }
        Ok(())
    }
    fn intersect_pair(
        &mut self,
        domains: &[Rc<dyn TheTopolTool>; 2],
        same: bool,
        analytic: bool,
    ) -> Result<(), Failure> {
        let tangential = surface_policy::tangential_tolerance(
            &self.surfaces[0],
            &self.surfaces[1],
            self.tolerance(),
        )?;
        let first = self.surfaces[0].clone().cast();
        let tolerance = self.tolerance();
        if same {
            return self.kernel.intersection.perform_self(
                &first,
                &*domains[0],
                tolerance,
                tangential,
            );
        }
        self.intersect_distinct(domains, analytic, tangential)
    }
    fn intersect_distinct(
        &mut self,
        domains: &[Rc<dyn TheTopolTool>; 2],
        analytic: bool,
        tangential: f64,
    ) -> Result<(), Failure> {
        let first = self.surfaces[0].clone().cast();
        let second = self.surfaces[1].clone().cast();
        let tolerance = self.tolerance();
        let options = IntPatchIntersectionOptions {
            is_geom_int: analytic,
            ..Default::default()
        };
        self.kernel.intersection.perform_with_points(
            &first,
            &*domains[0],
            &second,
            &*domains[1],
            (tolerance, tangential),
            &mut self.seeds,
            options,
        )
    }
    fn complete_intersection(
        &mut self,
        domains: &[Rc<dyn TheTopolTool>; 2],
        reverse: bool,
        parallel: bool,
    ) -> Result<(), Failure> {
        if !self.done {
            return Ok(());
        }
        self.tangent = self.kernel.intersection.tangent_faces()?;
        if self.tangent {
            return Ok(());
        }
        for index in 1..=self.kernel.intersection.nb_lines()? {
            self.make_curve(index, domains, self.tolerance())?;
        }
        self.compute_tolerance_reached(parallel)?;
        if reverse {
            self.reverse_pcurves();
        }
        self.append_intersection_points(reverse)
    }
    fn reverse_pcurves(&mut self) {
        for curve in &mut self.curves {
            let first = curve.first_curve2d().clone();
            let second = curve.second_curve2d().clone();
            curve.set_first_curve2d(&second);
            curve.set_second_curve2d(&first);
        }
    }
}
