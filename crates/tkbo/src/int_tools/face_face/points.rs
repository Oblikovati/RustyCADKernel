//! Native isolated points accumulate across calls and refer to the input face order.
use super::super::PntOnFace;
use super::*;
use rustycad_tkmath::gp::Point2;
impl FaceFace {
    pub(super) fn append_intersection_points(&mut self, reverse: bool) -> Result<(), Failure> {
        for index in 1..=self.kernel.intersection.nb_pnts()? {
            let point = self.kernel.intersection.point(index)?.pnt_on_2s();
            self.append_point(&point, reverse)?;
        }
        Ok(())
    }
    fn append_point(&mut self, point: &IntSurfPntOn2S, reverse: bool) -> Result<(), Failure> {
        let (u1, v1, u2, v2) = point.parameters();
        if !self
            .context
            .is_point_in_on_face(&self.faces[0], Point2::new(u1, v1))?
        {
            return Ok(());
        }
        if !self
            .context
            .is_point_in_on_face(&self.faces[1], Point2::new(u2, v2))?
        {
            return Ok(());
        }
        self.store_point(point, reverse);
        Ok(())
    }
    fn store_point(&mut self, point: &IntSurfPntOn2S, reverse: bool) {
        let (u1, v1, u2, v2) = point.parameters();
        let mut first = PntOnFace::default();
        let mut second = PntOnFace::default();
        first.init(&self.faces[0], point.value(), u1, v1);
        second.init(&self.faces[1], point.value(), u2, v2);
        self.points.push(if reverse {
            PntOn2Faces::new(&second, &first)
        } else {
            PntOn2Faces::new(&first, &second)
        });
    }
}
