//! Approximation controls preserve surface rejection across the native retry loop.
use super::super::WLineTool;
use super::*;
use super::{curves::ApproximationRetry, surface_policy, walking::WalkingPart};
use rustycad_tkgeomalgo::{
    approx_int::{ApproxIntKnotTools, ApproxIntRange},
    geom_int::GeomIntWLApprox,
};
use rustycad_tkgeombase::approx::ParametrizationType;
impl FaceFace {
    pub(super) fn approximate_part(
        &self,
        part: &WalkingPart,
        kinds: [SurfaceType; 2],
        flags: &mut [bool; 2],
        approximation: &mut GeomIntWLApprox,
        retry: &mut ApproximationRetry,
    ) -> Result<(), Failure> {
        let spatial = planar_approximation_flags(kinds, flags, self.approximation[0]);
        let parametrization = ApproxIntKnotTools::define_par_type(
            &part.line,
            part.first,
            part.last,
            (spatial, flags[0], flags[1]),
        )?;
        self.set_approximation(approximation, parametrization, retry)?;
        let wanted =
            self.approximation_range(approximation, part, kinds, *flags, parametrization, retry)?;
        self.fit_range(approximation, part, wanted)
    }
    fn fit_range(
        &self,
        approximation: &mut GeomIntWLApprox,
        part: &WalkingPart,
        wanted: [bool; 3],
    ) -> Result<(), Failure> {
        let range = ApproxIntRange {
            approx_xyz: wanted[0],
            approx_u1v1: wanted[1],
            approx_u2v2: wanted[2],
            indicemin: part.first,
            indicemax: part.last,
        };
        approximation.perform_on_surfaces(
            &self.surfaces[0].clone().cast(),
            &self.surfaces[1].clone().cast(),
            &part.line,
            range,
        )
    }
    fn set_approximation(
        &self,
        approximation: &mut GeomIntWLApprox,
        parametrization: ParametrizationType,
        retry: &mut ApproximationRetry,
    ) -> Result<(), Failure> {
        let same = self.surfaces[0] == self.surfaces[1];
        let reject = same || (retry.reapproximate && !retry.reject_surface);
        if same {
            retry.reject_surface = true;
        }
        let [low, high, iterations] = if reject {
            [4, 8, 0]
        } else {
            surface_policy::approximation_parameters(&self.surfaces[0], &self.surfaces[1])?
        };
        self.apply_approximation_parameters(
            approximation,
            [low, high, iterations],
            !reject,
            parametrization,
        );
        Ok(())
    }
    fn apply_approximation_parameters(
        &self,
        approximation: &mut GeomIntWLApprox,
        [low, high, iterations]: [i32; 3],
        use_surfaces: bool,
        parametrization: ParametrizationType,
    ) {
        approximation.set_parameters(
            self.approximation_tolerance,
            self.approximation_tolerance,
            low,
            high,
            iterations,
            30,
            use_surfaces,
            parametrization,
        );
    }
    fn approximation_range(
        &self,
        approximation: &mut GeomIntWLApprox,
        part: &WalkingPart,
        kinds: [SurfaceType; 2],
        flags: [bool; 2],
        parametrization: ParametrizationType,
        retry: &mut ApproximationRetry,
    ) -> Result<[bool; 3], Failure> {
        if kinds[0] == SurfaceType::Plane {
            return Ok([false, true, self.approximation[2]]);
        }
        if kinds[1] == SurfaceType::Plane {
            return Ok([false, self.approximation[1], true]);
        }
        self.spline_approximation(approximation, part, kinds, parametrization, retry)?;
        Ok([true, flags[0], flags[1]])
    }
    fn reject_spline_surfaces(&self, part: &WalkingPart) -> Result<bool, Failure> {
        WLineTool::not_use_surfaces_for_approx(
            &self.faces[0],
            &self.faces[1],
            &part.line,
            part.first,
            part.last,
        )
    }
    fn spline_approximation(
        &self,
        approximation: &mut GeomIntWLApprox,
        part: &WalkingPart,
        kinds: [SurfaceType; 2],
        parametrization: ParametrizationType,
        retry: &mut ApproximationRetry,
    ) -> Result<(), Failure> {
        let spline = |kind| {
            matches!(
                kind,
                SurfaceType::BezierSurface | SurfaceType::BSplineSurface
            )
        };
        if self.surfaces[0] == self.surfaces[1] || !spline(kinds[0]) || !spline(kinds[1]) {
            return Ok(());
        }
        self.apply_approximation_parameters(approximation, [4, 8, 0], true, parametrization);
        if self.reject_spline_surfaces(part)? {
            retry.reject_surface = true;
            self.apply_approximation_parameters(approximation, [4, 8, 0], false, parametrization);
        }
        Ok(())
    }
}

fn planar_approximation_flags(
    kinds: [SurfaceType; 2],
    flags: &mut [bool; 2],
    spatial: bool,
) -> bool {
    if kinds[0] == SurfaceType::Plane {
        flags[0] = true;
        return false;
    }
    if kinds[1] == SurfaceType::Plane {
        flags[1] = true;
        return false;
    }
    spatial
}
