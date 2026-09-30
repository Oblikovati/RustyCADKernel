use super::*;
use rustycad_tkg3d::geom::GeomBSplineSurface;

pub(super) struct CachedSurfaceProjector {
    analysis: Handle<ShapeAnalysisSurface>,
    corners: Vec<(Point3, Point2)>,
}

impl CachedSurfaceProjector {
    pub fn new(analysis: &Handle<ShapeAnalysisSurface>) -> Result<Self, Failure> {
        let mut projector = Self {
            analysis: analysis.clone(),
            corners: Vec::new(),
        };
        if analysis.is_null() {
            return Ok(projector);
        }
        let spline = analysis.surface().cast::<GeomBSplineSurface>();
        if let Some(spline) = spline.get() {
            projector.cache_corners(spline)?;
        }
        Ok(projector)
    }

    pub fn value_of_uv(
        &self,
        point: Point3,
        tolerance: f64,
        squared_tolerance: f64,
    ) -> Result<Point2, Failure> {
        if let Some(uv) = self.corner(point, squared_tolerance) {
            return self
                .analysis
                .next_value_of_uv(uv, point, tolerance, tolerance);
        }
        self.analysis.value_of_uv(point, tolerance)
    }

    pub fn next_value_of_uv(
        &self,
        hint: Point2,
        point: Point3,
        tolerance: f64,
        squared_tolerance: f64,
        step: f64,
    ) -> Result<Point2, Failure> {
        if let Some(uv) = self.corner(point, squared_tolerance) {
            return self
                .analysis
                .next_value_of_uv(uv, point, tolerance, tolerance);
        }
        self.analysis.next_value_of_uv(hint, point, tolerance, step)
    }

    pub fn gap(&self) -> f64 {
        self.analysis.gap()
    }

    fn corner(&self, point: Point3, squared_tolerance: f64) -> Option<Point2> {
        self.corners
            .iter()
            .find(|(corner, _)| corner.square_distance(&point) < squared_tolerance)
            .map(|(_, uv)| *uv)
    }

    fn cache_corners(&mut self, spline: &GeomBSplineSurface) -> Result<(), Failure> {
        let u = [(1, 1), (spline.nb_u_poles(), spline.nb_u_knots())];
        let v = [(1, 1), (spline.nb_v_poles(), spline.nb_v_knots())];
        for &(vi, vk) in &v {
            for &(ui, uk) in &u {
                self.cache_corner(spline, (ui, vi), (uk, vk))?;
            }
        }
        Ok(())
    }

    fn cache_corner(
        &mut self,
        spline: &GeomBSplineSurface,
        pole: (i32, i32),
        knot: (i32, i32),
    ) -> Result<(), Failure> {
        if spline.u_multiplicity(knot.0)? < spline.u_degree() + 1
            || spline.v_multiplicity(knot.1)? < spline.v_degree() + 1
        {
            return Ok(());
        }
        self.corners.push((
            spline.pole(pole.0, pole.1)?,
            Point2::new(spline.u_knot(knot.0)?, spline.v_knot(knot.1)?),
        ));
        Ok(())
    }
}

pub(super) fn cached_uv(cache: &[(Point3, Point2)], point: Point3, squared: f64) -> Option<Point2> {
    cache
        .iter()
        .find(|(cached, _)| cached.square_distance(&point) < squared)
        .map(|(_, uv)| *uv)
}
