//! Native walking-line decomposition and approximation restart state.
use super::super::WLineTool;
use super::*;
use super::{curves::ApproximationRetry, surface_policy};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::{
    geom_int::{GeomIntIntSS, GeomIntWLApprox},
    int_patch::{IntPatchPointLine, IntPatchWLine},
};
#[derive(Clone)]
pub(super) struct WalkingPart {
    pub(super) line: Handle<IntPatchWLine>,
    pub(super) first: i32,
    pub(super) last: i32,
}
impl FaceFace {
    pub(super) fn walking_curves(
        &mut self,
        line: &Handle<IntPatchWLine>,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        if !self.approximation[0] {
            for index in 1..=self.kernel.constructor.nb_parts()? {
                let part = self.walking_part(line, index)?;
                let curve = self.polygonal_curve(&part, false)?;
                self.curves.push(curve);
            }
            return Ok(false);
        }
        self.approximate_walking(line, retry)
    }
    fn walking_part(
        &self,
        line: &Handle<IntPatchWLine>,
        index: i32,
    ) -> Result<WalkingPart, Failure> {
        let (first, last) = self.kernel.constructor.part(index)?;
        Ok(WalkingPart {
            line: line.clone(),
            first: first as i32,
            last: last as i32,
        })
    }
    pub(super) fn polygonal_curve(
        &self,
        part: &WalkingPart,
        spatial_first: bool,
    ) -> Result<Curve, Failure> {
        let early = if spatial_first {
            Some(GeomIntIntSS::make_bspline(
                &part.line, part.first, part.last,
            )?)
        } else {
            None
        };
        let mut pcurves: [Handle<dyn Geom2dCurve>; 2] = std::array::from_fn(|_| Handle::default());
        for (index, pcurve) in pcurves.iter_mut().enumerate() {
            if self.approximation[index + 1] {
                *pcurve =
                    GeomIntIntSS::make_bspline2d(&part.line, part.first, part.last, index == 0)?
                        .cast();
            }
        }
        let curve = match early {
            Some(curve) => curve,
            None => GeomIntIntSS::make_bspline(&part.line, part.first, part.last)?,
        };
        Ok(Curve::new(&curve, &pcurves[0], &pcurves[1], 0., 0.))
    }
    fn approximate_walking(
        &mut self,
        line: &Handle<IntPatchWLine>,
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        let kinds = [self.surfaces[0].get_type()?, self.surfaces[1].get_type()?];
        let mut flags = [self.approximation[1], self.approximation[2]];
        if !self.approximation_with_pcurves(kinds)? {
            self.approximation_tolerance = 1e-5;
            flags = [false; 2];
        }
        let (parts, decomposed) = self.decomposed_parts(line)?;
        self.approximate_parts(parts, decomposed, kinds, &mut flags, retry)
    }
    fn approximate_parts(
        &mut self,
        parts: Vec<WalkingPart>,
        decomposed: bool,
        kinds: [SurfaceType; 2],
        flags: &mut [bool; 2],
        retry: &mut ApproximationRetry,
    ) -> Result<bool, Failure> {
        let mut tolerance = if decomposed {
            precision::confusion()
        } else {
            0.
        };
        let mut approximation = GeomIntWLApprox::new();
        for part in parts {
            self.approximate_part(&part, kinds, flags, &mut approximation, retry)?;
            if !approximation.is_done() {
                let curve = self.polygonal_curve(&part, true)?;
                self.curves.push(curve);
                continue;
            }
            tolerance = approximation_floor(kinds, tolerance);
            if self.append_approximations(&approximation, &part, kinds, *flags, tolerance, retry)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn approximation_with_pcurves(&self, kinds: [SurfaceType; 2]) -> Result<bool, Failure> {
        match kinds {
            [SurfaceType::Cylinder, SurfaceType::Sphere] => {
                surface_policy::approximate_with_pcurves(
                    self.surfaces[0].cylinder()?,
                    self.surfaces[1].sphere()?,
                )
            }
            [SurfaceType::Sphere, SurfaceType::Cylinder] => {
                surface_policy::approximate_with_pcurves(
                    self.surfaces[1].cylinder()?,
                    self.surfaces[0].sphere()?,
                )
            }
            _ => Ok(true),
        }
    }
    fn decomposed_parts(
        &self,
        line: &Handle<IntPatchWLine>,
    ) -> Result<(Vec<WalkingPart>, bool), Failure> {
        let mut lines = Vec::new();
        let decomposed = WLineTool::decomposition_of_wline(
            line,
            &self.surfaces[0],
            &self.surfaces[1],
            &self.faces[0],
            &self.faces[1],
            &self.kernel.constructor,
            false,
            self.tolerance(),
            &mut lines,
            &self.context,
        )?;
        Ok((self.collect_parts(line, lines, decomposed)?, decomposed))
    }
    fn collect_parts(
        &self,
        line: &Handle<IntPatchWLine>,
        lines: Vec<Handle<dyn rustycad_tkgeomalgo::int_patch::IntPatchLine>>,
        decomposed: bool,
    ) -> Result<Vec<WalkingPart>, Failure> {
        let mut parts = Vec::new();
        if decomposed {
            for line in lines {
                let line = line.cast::<IntPatchWLine>();
                let last = line.nb_pnts()?;
                parts.push(WalkingPart {
                    line,
                    first: 1,
                    last,
                });
            }
        } else {
            for index in 1..=self.kernel.constructor.nb_parts()? {
                parts.push(self.walking_part(line, index)?);
            }
        }
        Ok(parts)
    }
}

fn approximation_floor(kinds: [SurfaceType; 2], tolerance: f64) -> f64 {
    if kinds.contains(&SurfaceType::Plane)
        && kinds.contains(&SurfaceType::Torus)
        && tolerance < 1e-6
    {
        return 1e-6;
    }
    tolerance
}
