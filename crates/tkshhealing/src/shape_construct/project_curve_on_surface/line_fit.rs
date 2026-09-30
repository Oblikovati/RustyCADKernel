use super::periodicity::fix_periodicity;
use super::point_projection::CachedSurfaceProjector;
use super::sampling::CurveSamples;
use super::*;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dLine, native::std_max};
use rustycad_tkmath::gp::{Direction2, Direction3, Line3, Vector2};

#[derive(Default)]
struct LineProjection {
    points: [Point3; 4],
    uv: [Point2; 4],
    working_tolerance: f64,
    squared_tolerance: f64,
    original_squared: f64,
    saved_index: i32,
    saved: Point2,
    recompute: bool,
    from_cache: bool,
}

pub(super) struct LineFitResult {
    pub curve: Handle<dyn Geom2dCurve>,
    pub recompute: bool,
    pub from_cache: bool,
}

impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn get_line(&self, samples: &mut CurveSamples) -> Result<LineFitResult, Failure> {
        let mut projection = LineProjection::new(samples, self.fields.read().precision);
        self.project_line_extremities(&mut projection)?;
        let surface = self.analysis().surface();
        if projection.recompute && surface.is_kind_name("Geom_SphericalSurface") {
            return Ok(projection.result(Handle::default()));
        }
        let count = samples.points.length();
        samples.uv[1] = projection.uv[0];
        samples.uv[count] = projection.uv[3];
        let range = (samples.parameters[1], samples.parameters[count]);
        if (range.1 - range.0).abs() < precision::p_confusion() {
            return Ok(projection.result(Handle::default()));
        }
        let vector = Vector2::between(projection.uv[0], projection.uv[3]) / (range.1 - range.0);
        if !line_matches_surface(&surface, samples, &projection, vector)? {
            return Ok(projection.result(Handle::default()));
        }
        let curve = parameterized_line([projection.uv[0], projection.uv[3]], range)?;
        Ok(projection.result(curve))
    }

    fn project_line_extremities(&self, projection: &mut LineProjection) -> Result<(), Failure> {
        let analysis = self.analysis();
        let surface = analysis.surface();
        let projector = CachedSurfaceProjector::new(&analysis)?;
        let cache = self.fields.read().cache.clone();
        for index in [0, 3] {
            projection.project(index, &projector, &cache)?;
        }
        if surface.is_u_periodic() || surface.is_v_periodic() {
            for index in [1, 2] {
                projection.project(index, &projector, &cache)?;
            }
        }
        projection.resolve_periods(&surface)
    }
}

impl LineProjection {
    fn new(samples: &CurveSamples, tolerance: f64) -> Self {
        let count = samples.points.length();
        let working_tolerance = if tolerance * tolerance > 1. {
            precision::confusion()
        } else {
            tolerance
        };
        let squared_tolerance = std_max(
            working_tolerance * working_tolerance,
            precision::square_confusion(),
        );
        Self {
            points: [1, 2, count - 1, count].map(|index| samples.points[index]),
            working_tolerance,
            squared_tolerance,
            original_squared: squared_tolerance,
            saved_index: -1,
            ..Self::default()
        }
    }

    fn project(
        &mut self,
        index: usize,
        projector: &CachedSurfaceProjector,
        cache: &[(Point3, Point2)],
    ) -> Result<(), Failure> {
        self.uv[index] = self.project_point(index, projector, cache)?;
        let squared = projector.gap() * projector.gap();
        if self.squared_tolerance < squared {
            self.squared_tolerance = squared;
        }
        Ok(())
    }

    fn project_point(
        &mut self,
        index: usize,
        projector: &CachedSurfaceProjector,
        cache: &[(Point3, Point2)],
    ) -> Result<Point2, Failure> {
        let cached =
            super::point_projection::cached_uv(cache, self.points[index], self.squared_tolerance);
        let Some(uv) = cached else {
            return projector.value_of_uv(
                self.points[index],
                self.working_tolerance,
                self.squared_tolerance,
            );
        };
        let projected = projector.next_value_of_uv(
            uv,
            self.points[index],
            self.working_tolerance,
            self.squared_tolerance,
            self.working_tolerance,
        )?;
        self.record_cache(index, uv);
        Ok(projected)
    }

    fn record_cache(&mut self, index: usize, uv: Point2) {
        self.saved_index = index as i32;
        self.saved = uv;
        if index == 0 {
            self.from_cache = true;
        }
    }

    fn resolve_periods(&mut self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        if surface.is_u_periodic() {
            self.recompute = fix_periodicity(
                &mut self.uv,
                1,
                surface.u_period()?,
                self.saved_index,
                self.saved.x(),
            )?;
        }
        if surface.is_v_periodic() {
            self.recompute = fix_periodicity(
                &mut self.uv,
                2,
                surface.v_period()?,
                self.saved_index,
                self.saved.y(),
            )?;
        }
        Ok(())
    }

    fn result(&self, curve: Handle<dyn Geom2dCurve>) -> LineFitResult {
        LineFitResult {
            curve,
            recompute: self.recompute,
            from_cache: self.from_cache,
        }
    }

    fn parameter_point(&self, parameter: f64, first: f64, vector: Vector2) -> Point2 {
        Point2::new(
            self.uv[0].x() + vector.x() * (parameter - first),
            self.uv[0].y() + vector.y() * (parameter - first),
        )
    }
}

fn line_matches_surface(
    surface: &Handle<dyn GeomSurface>,
    samples: &CurveSamples,
    projection: &LineProjection,
    vector: Vector2,
) -> Result<bool, Failure> {
    if surface.is_cn_u(1)?
        && surface.is_cn_v(1)?
        && let Some(matches) = line_normal_check(surface, samples, projection, vector)?
    {
        return Ok(matches);
    }
    line_distance_check(surface, samples, projection, vector)
}

fn line_normal_check(
    surface: &Handle<dyn GeomSurface>,
    samples: &CurveSamples,
    projection: &LineProjection,
    vector: Vector2,
) -> Result<Option<bool>, Failure> {
    for index in 1..=samples.points.length() {
        let uv =
            projection.parameter_point(samples.parameters[index], samples.parameters[1], vector);
        let derivatives = surface.d1(uv.x(), uv.y())?;
        let normal = derivatives.d1u.crossed(derivatives.d1v);
        if normal.square_magnitude() < precision::square_confusion() {
            return Ok(None);
        }
        let line = Line3::new(derivatives.point, Direction3::from_vector(normal)?);
        if line.distance(samples.points[index]) > projection.working_tolerance {
            return Ok(Some(false));
        }
    }
    Ok(Some(true))
}

fn line_distance_check(
    surface: &Handle<dyn GeomSurface>,
    samples: &CurveSamples,
    projection: &LineProjection,
    vector: Vector2,
) -> Result<bool, Failure> {
    let first = surface
        .value(projection.uv[0].x(), projection.uv[0].y())?
        .square_distance(&samples.points[1]);
    let tolerance = std_max(
        projection.original_squared,
        projection.original_squared * 2. * first,
    );
    for index in 2..samples.points.length() {
        let uv =
            projection.parameter_point(samples.parameters[index], samples.parameters[1], vector);
        let distance = surface
            .value(uv.x(), uv.y())?
            .square_distance(&samples.points[index]);
        if (first - distance).abs() > tolerance {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn parameterized_line(
    points: [Point2; 2],
    range: (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let vector = Vector2::between(points[0], points[1]);
    let length = vector.magnitude();
    if (length - (range.1 - range.0)).abs() <= precision::p_confusion() {
        let direction = vector / length;
        let point = Point2::new(
            points[0].x() - range.0 * direction.x(),
            points[0].y() - range.0 * direction.y(),
        );
        return Ok(Handle::new(Geom2dLine::from_point(
            point,
            Direction2::from_vector(direction)?,
        ))
        .cast());
    }
    straight_bspline(points, range)
}

pub(super) fn straight_bspline(
    points: [Point2; 2],
    range: (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let knots = [range.0, range.1];
    let mults = [2, 2];
    let spline = Geom2dBSplineCurve::new(
        &Array1::from_shared(&points, 1),
        &Array1::from_shared(&knots, 1),
        &Array1::from_shared(&mults, 1),
        1,
        false,
    )?;
    Ok(Handle::new(spline).cast())
}
