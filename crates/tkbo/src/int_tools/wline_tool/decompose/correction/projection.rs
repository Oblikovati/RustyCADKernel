use super::*;
impl Decomposition<'_> {
    pub(super) fn project_candidate(
        &self,
        point: &mut IntSurfPntOn2S,
        group: &WalkingGroup,
        first: bool,
        surface: usize,
        mut candidate: Point2,
        bounds: SurfaceBounds,
    ) -> Result<(), Failure> {
        let mut spatial = self.surfaces[surface].value(candidate.x(), candidate.y())?;
        let Some(projected) = self.project_uv(spatial, 1 - surface)? else {
            return Ok(());
        };
        let neighbour = self.neighbour(group, first, 1 - surface)?;
        let mut adjusted = adjust_by_neighbour(neighbour, projected, self.surfaces[1 - surface])?;
        self.correct_projected(&mut spatial, &mut candidate, &mut adjusted, surface, bounds)?;
        store_projection(point, spatial, candidate, adjusted, surface);
        Ok(())
    }
    fn correct_projected(
        &self,
        spatial: &mut gp::Point3,
        candidate: &mut Point2,
        adjusted: &mut Point2,
        surface: usize,
        bounds: SurfaceBounds,
    ) -> Result<(), Failure> {
        // Native uses AND for all four comparisons, and the original surface's bounds.
        if adjusted.x() < bounds.0
            && adjusted.x() > bounds.1
            && adjusted.y() < bounds.2
            && adjusted.y() > bounds.3
        {
            *adjusted = clamp_projected(*adjusted, bounds);
            *spatial = self.surfaces[1 - surface].value(adjusted.x(), adjusted.y())?;
            if let Some(uv) = self.project_uv(*spatial, surface)? {
                *candidate = uv;
            }
        }
        Ok(())
    }
    fn project_uv(&self, point: gp::Point3, surface: usize) -> Result<Option<Point2>, Failure> {
        let service = self.context.proj_ps(self.faces[surface])?;
        let mut projector = service.write();
        projector.perform(point)?;
        if !projector.is_done() {
            return Ok(None);
        }
        if projector.lower_distance()?.partial_cmp(&self.tolerance)
            != Some(std::cmp::Ordering::Less)
        {
            return Ok(None);
        }
        let (u, v) = projector.lower_distance_parameters()?;
        Ok(Some(Point2::new(u, v)))
    }
}
fn clamp_projected(point: Point2, bounds: SurfaceBounds) -> Point2 {
    let mut u = point.x();
    let mut v = point.y();
    if u < bounds.0 {
        u = bounds.0;
    }
    if u > bounds.1 {
        u = bounds.1;
    }
    if v < bounds.2 {
        v = bounds.2;
    }
    if v > bounds.3 {
        v = bounds.3;
    }
    Point2::new(u, v)
}

fn store_projection(
    point: &mut IntSurfPntOn2S,
    spatial: gp::Point3,
    candidate: Point2,
    adjusted: Point2,
    surface: usize,
) {
    if surface == 0 {
        point.set_value_all(
            spatial,
            candidate.x(),
            candidate.y(),
            adjusted.x(),
            adjusted.y(),
        );
    } else {
        point.set_value_all(
            spatial,
            adjusted.x(),
            adjusted.y(),
            candidate.x(),
            candidate.y(),
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projected_clamp_keeps_native_sequential_overwrite_for_inverted_bounds() {
        assert_eq!(
            clamp_projected(Point2::new(-1., 3.), (0., 2., 0., 2.)),
            Point2::new(0., 2.)
        );
        assert_eq!(
            clamp_projected(Point2::new(0., 0.), (2., 1., 4., 3.)),
            Point2::new(1., 3.)
        );
    }
}
