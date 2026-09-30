use super::*;
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};

impl ShapeAnalysisSurface {
    /// Finds the nearest eligible singularity, preserving the native last-gap side effect.
    /// `let entry = analysis.degenerated_values(point, 1e-7, true)?;`
    pub fn degenerated_values(
        &self,
        point: Point3,
        tolerance: f64,
        _forward: bool,
    ) -> Result<Option<SurfaceSingularity>, Failure> {
        let (best, minimum) = self.select_singularity(tolerance, tolerance, |entry| {
            let gap = entry.point.distance(&point);
            self.fields.write().gap = gap;
            Ok(gap)
        })?;
        if best.is_some() {
            self.fields.write().gap = minimum;
        }
        Ok(best)
    }

    /// Adjusts the indeterminate coordinate at a singularity; `analysis.project_degenerated(point, tol, neighbour, &mut uv)?`.
    pub fn project_degenerated(
        &self,
        point: Point3,
        tolerance: f64,
        neighbour: Point2,
        result: &mut Point2,
    ) -> Result<bool, Failure> {
        let Some(entry) = self.closest_degenerate(point, *result, tolerance)? else {
            return Ok(false);
        };
        copy_degenerate_coordinate(result, neighbour, entry.u_iso);
        Ok(true)
    }

    /// Detects a degenerate parameter segment; `analysis.is_degenerated_segment(a, b, 1e-7, 10.)?`.
    pub fn is_degenerated_segment(
        &self,
        first: Point2,
        last: Point2,
        tolerance: f64,
        ratio: f64,
    ) -> Result<bool, Failure> {
        let maximum = self.segment_spread(first, last)?;
        if maximum > tolerance {
            return Ok(false);
        }
        let adaptor = self.adaptor3d();
        let (ru, rv) = (adaptor.u_resolution(1.)?, adaptor.v_resolution(1.)?);
        if ru < precision::p_confusion() || rv < precision::p_confusion() {
            return Ok(false);
        }
        let du = (first.x() - last.x()).abs() / ru;
        let dv = (first.y() - last.y()).abs() / rv;
        let scaled = maximum * ratio;
        Ok(du * du + dv * dv > scaled * scaled)
    }

    fn closest_degenerate(
        &self,
        point: Point3,
        uv: Point2,
        tolerance: f64,
    ) -> Result<Option<SurfaceSingularity>, Failure> {
        let (best, minimum) =
            self.select_singularity(tolerance, tolerance * tolerance, |entry| {
                let squared = entry.point.square_distance(&point);
                if squared > tolerance * tolerance {
                    return Ok(std_min(
                        squared,
                        entry.point.square_distance(&self.value_uv(uv)?),
                    ));
                }
                Ok(squared)
            })?;
        if best.is_some() {
            self.fields.write().gap = minimum.sqrt();
        }
        Ok(best)
    }

    pub(super) fn select_singularity(
        &self,
        tolerance: f64,
        maximum: f64,
        measure: impl Fn(SurfaceSingularity) -> Result<f64, Failure>,
    ) -> Result<(Option<SurfaceSingularity>, f64), Failure> {
        self.compute_singularities()?;
        let entries = self.fields.read().singularities.clone().unwrap();
        let mut best = None;
        let mut minimum = f64::MAX;
        for entry in entries
            .into_iter()
            .take_while(|entry| entry.precision <= tolerance)
        {
            let gap = measure(entry)?;
            if gap <= maximum && minimum > gap {
                minimum = gap;
                best = Some(entry);
            }
        }
        Ok((best, minimum))
    }

    fn segment_spread(&self, first: Point2, last: Point2) -> Result<f64, Failure> {
        let a = self.value_uv(first)?;
        let b = self.value_uv(last)?;
        let midpoint = self.value(0.5 * (first.x() + last.x()), 0.5 * (first.y() + last.y()))?;
        Ok(std_max(
            a.distance(&b),
            std_max(midpoint.distance(&a), midpoint.distance(&b)),
        ))
    }
}

pub(super) fn copy_degenerate_coordinate(result: &mut Point2, neighbour: Point2, u_iso: bool) {
    if u_iso {
        result.set_y(neighbour.y());
    } else {
        result.set_x(neighbour.x());
    }
}

impl ShapeAnalysisSurface {
    /// Adjusts singular points at either end, distributing an entirely degenerate curve.
    /// `analysis.project_degenerated_points(&points, &mut uv, 1e-7, true)?;`
    pub fn project_degenerated_points(
        &self,
        points: &[Point3],
        parameters: &mut [Point2],
        tolerance: f64,
        direct: bool,
    ) -> Result<bool, Failure> {
        self.project_degenerated_sequence(
            points.len() as i32,
            points,
            parameters,
            tolerance,
            direct,
        )
    }
}

pub(super) fn degenerate_coordinate(parameter: Point2, u_iso: bool) -> f64 {
    if u_iso { parameter.y() } else { parameter.x() }
}
