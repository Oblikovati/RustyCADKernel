use super::edge::EdgeSampling;
use super::*;
use rustycad_tkmath::{
    elclib,
    gp::{Direction2, Line2, Vector2},
};
#[derive(Default)]
pub(super) struct WirePolygon {
    pub(super) points: Vec<Point2>,
    pub(super) deflection: [f64; 2],
    pub(super) edges_left: i32,
    pub(super) not_empty: bool,
    previous: Option<Point3>,
    skip_first: bool,
}
impl WirePolygon {
    pub(super) fn sample(
        classifier: &mut FClass2d,
        wire: &Wire,
        bad: bool,
    ) -> Result<Option<Self>, Failure> {
        let mut polygon = Self {
            edges_left: Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)?.count() as i32,
            ..Default::default()
        };
        let face = classifier.face.clone();
        let mut explorer = WireExplorer::with_wire_on_face(wire, &face)?;
        while explorer.more() {
            polygon.edges_left -= 1;
            if !polygon.sample_edge(classifier, explorer.current(), bad)? {
                return Ok(None);
            }
            explorer.next()?;
        }
        Ok(Some(polygon))
    }
    fn sample_edge(
        &mut self,
        classifier: &mut FClass2d,
        edge: &Edge,
        bad: bool,
    ) -> Result<bool, Failure> {
        if !boundary_edge(edge) {
            return Ok(true);
        }
        let Some(sampling) = EdgeSampling::new(edge, &classifier.face)? else {
            return Ok(false);
        };
        self.sample_parameters(classifier, &sampling)?;
        if bad {
            return Ok(true);
        }
        self.finish_edge(&sampling)
    }
    fn finish_edge(&mut self, sampling: &EdgeSampling) -> Result<bool, Failure> {
        self.skip_first = true;
        self.not_empty = true;
        // The native derivative sequences are not subsequently read, but evaluation can fail.
        sampling.curve.d1(sampling.parameters[0])?;
        sampling.curve.d1(*sampling.parameters.last().unwrap())?;
        Ok(true)
    }
    fn sample_parameters(
        &mut self,
        classifier: &mut FClass2d,
        sampling: &EdgeSampling,
    ) -> Result<(), Failure> {
        let before = self.points.len();
        for index in usize::from(self.skip_first)..sampling.parameters.len() {
            self.add_sample(classifier, sampling, index, before)?;
        }
        Ok(())
    }
    fn add_sample(
        &mut self,
        c: &mut FClass2d,
        e: &EdgeSampling,
        index: usize,
        before: usize,
    ) -> Result<(), Failure> {
        let parameter = e.parameters[index];
        let uv = e.curve.value(parameter)?;
        c.grow_bounds(uv);
        self.retain_sample(e, index, uv)?;
        if self.points.len() > before + 4 {
            self.update_deflection()?;
        }
        Ok(())
    }
    fn retain_sample(&mut self, e: &EdgeSampling, index: usize, uv: Point2) -> Result<(), Failure> {
        let spatial = if e.degenerated {
            Point3::default()
        } else {
            e.spatial.value(e.parameters[index])?
        };
        if !e.is_repeated(index, spatial, self.previous)? {
            if !e.degenerated {
                self.previous = Some(spatial);
            }
            self.points.push(uv);
        }
        Ok(())
    }
    pub(super) fn update_deflection(&mut self) -> Result<(), Failure> {
        let count = self.points.len();
        let (a, b, c) = (
            self.points[count - 3],
            self.points[count - 2],
            self.points[count - 1],
        );
        let line = Line2::new(a, Direction2::from_vector(Vector2::between(a, c))?);
        let projected = elclib::value(elclib::parameter(&line, b)?, &line);
        self.deflection[0] = std_max(self.deflection[0], (projected.x() - b.x()).abs());
        self.deflection[1] = std_max(self.deflection[1], (projected.y() - b.y()).abs());
        Ok(())
    }
}
