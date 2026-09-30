use super::polygon::WirePolygon;
use super::*;
impl FClass2d {
    pub(super) fn sample_wires(&mut self) -> Result<(), Failure> {
        let mut explorer = Explorer::new(&self.face, ShapeEnum::Wire, ShapeEnum::Shape)?;
        let mut bad = false;
        while explorer.more() {
            let wire = Wire::unchecked(explorer.current().clone());
            let Some(mut polygon) = WirePolygon::sample(self, &wire, bad)? else {
                return Ok(());
            };
            bad = self.add_polygon(&wire, &mut polygon)? || bad;
            explorer.next()?;
        }
        if bad && !self.classifiers.is_empty() {
            self.orientations[0] = -1;
        }
        Ok(())
    }
    fn add_polygon(&mut self, wire: &Wire, p: &mut WirePolygon) -> Result<bool, Failure> {
        if p.edges_left != 0 {
            self.push_classifier(&[Point2::default(); 2], p.deflection);
            self.orientations.push(-1);
            return Ok(true);
        }
        if !p.not_empty {
            return Ok(false);
        }
        if p.points.len() <= 3 {
            self.push_classifier(&[], p.deflection);
            self.orientations.push(-1);
            return Ok(true);
        }
        self.add_refined_polygon(wire, p)
    }
    fn add_refined_polygon(&mut self, wire: &Wire, p: &mut WirePolygon) -> Result<bool, Failure> {
        let area = p.refine(wire, &self.face)?;
        let deflection = [
            std_max(p.deflection[0], self.tolerance),
            std_max(p.deflection[1], self.tolerance),
        ];
        self.push_classifier(&p.points, deflection);
        if area.abs() < precision::square_confusion() {
            self.orientations.push(-1);
            return Ok(true);
        }
        self.is_hole = area.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater);
        self.orientations.push(if area > 0.0 { 1 } else { 0 });
        Ok(false)
    }
    fn push_classifier(&mut self, points: &[Point2], deflection: [f64; 2]) {
        let array = Array1::from_shared(points, 1);
        let b = self.bounds;
        self.classifiers.push(Class2d::from_array(
            &array,
            deflection[0],
            deflection[1],
            b[0],
            b[2],
            b[1],
            b[3],
        ));
    }
    pub(super) fn grow_bounds(&mut self, p: Point2) {
        for (axis, value) in [p.x(), p.y()].into_iter().enumerate() {
            if value < self.bounds[axis * 2] {
                self.bounds[axis * 2] = value;
            }
            if value > self.bounds[axis * 2 + 1] {
                self.bounds[axis * 2 + 1] = value;
            }
        }
    }
}
