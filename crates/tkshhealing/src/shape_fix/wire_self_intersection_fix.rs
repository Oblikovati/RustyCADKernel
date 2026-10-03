//! Tolerance repair after native `CheckSelfIntersectingEdge` detects crossings.
use super::{FixRoot, ShapeFixWire};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::Point3;

struct SelfIntersectionRepair {
    first_vertex: rustycad_tkbrep::topods::Vertex,
    last_vertex: rustycad_tkbrep::topods::Vertex,
    first_point: Point3,
    last_point: Point3,
    first_tolerance: f64,
    last_tolerance: f64,
}

impl ShapeFixWire {
    /// `FixSelfIntersectingEdge(num)`: raise a vertex tolerance to cover crossings.
    /// Example: `let fixed = fixer.fix_self_intersecting_edge(1)?;`.
    /// Geometric loop removal and the mode-1 split branch remain unsupported.
    pub(super) fn fix_self_intersecting_edge(&self, index: i32) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer()?;
        if !analyzer.is_ready() {
            return Ok(false);
        }
        let (_points_2d, points_3d) = self.analyze_self_intersections(&analyzer, index)?;
        self.transfer_self_intersection_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        let mut repair = SelfIntersectionRepair::load(&analyzer, index)?;
        self.raise_intersection_tolerances(&mut repair, &points_3d)?;
        Ok(self.last_fix_status(ShapeExtendStatus::Done))
    }

    fn analyze_self_intersections(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
    ) -> Result<(Sequence<IntersectionPoint>, Sequence<Point3>), Failure> {
        let mut points_2d = Sequence::default();
        let mut points_3d = Sequence::default();
        analyzer.check_self_intersecting_edge_into(index, &mut points_2d, &mut points_3d)?;
        Ok((points_2d, points_3d))
    }

    fn raise_intersection_tolerances(
        &self,
        repair: &mut SelfIntersectionRepair,
        points: &Sequence<Point3>,
    ) -> Result<(), Failure> {
        for index in 1..=points.len() {
            let point = *points.value(index)?;
            self.raise_nearest_vertex_tolerance(repair, point)?;
        }
        Ok(())
    }

    fn raise_nearest_vertex_tolerance(
        &self,
        repair: &mut SelfIntersectionRepair,
        point: Point3,
    ) -> Result<(), Failure> {
        let gap = SelfIntersectionGap::measure(repair, point);
        if gap.already_covered(repair) {
            return Ok(());
        }
        if gap.required_tolerance() >= self.max_tolerance() {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
            return Ok(());
        }
        gap.raise_nearest_vertex(repair)?;
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        Ok(())
    }

    fn transfer_self_intersection_failure(&self, analyzer: &ShapeAnalysisWire) {
        if analyzer.last_check_status(ShapeExtendStatus::Fail) {
            self.or_last_fix_status(ShapeExtendStatus::Fail1);
        }
    }
}

impl SelfIntersectionRepair {
    fn load(analyzer: &ShapeAnalysisWire, index: i32) -> Result<Self, Failure> {
        let wire: Handle<ShapeExtendWireData> = analyzer.wire_data();
        let edge_index = if index > 0 { index } else { wire.nb_edges() };
        let edge = wire.edge(edge_index)?;
        let analysis = ShapeAnalysisEdge::new();
        let (first_vertex, last_vertex) =
            (analysis.first_vertex(&edge), analysis.last_vertex(&edge));
        let (first_point, last_point) =
            (BRepTool::pnt(&first_vertex)?, BRepTool::pnt(&last_vertex)?);
        Ok(Self {
            first_tolerance: BRepTool::vertex_tolerance(&first_vertex)?,
            last_tolerance: BRepTool::vertex_tolerance(&last_vertex)?,
            first_vertex,
            last_vertex,
            first_point,
            last_point,
        })
    }
}

struct SelfIntersectionGap {
    first_distance: f64,
    last_distance: f64,
}

impl SelfIntersectionGap {
    fn measure(repair: &SelfIntersectionRepair, point: Point3) -> Self {
        Self {
            first_distance: repair.first_point.distance(&point),
            last_distance: repair.last_point.distance(&point),
        }
    }

    fn already_covered(&self, repair: &SelfIntersectionRepair) -> bool {
        self.first_distance < repair.first_tolerance || self.last_distance < repair.last_tolerance
    }

    fn required_tolerance(&self) -> f64 {
        1.001 * self.first_distance.min(self.last_distance)
    }

    fn raise_nearest_vertex(&self, repair: &mut SelfIntersectionRepair) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        if self.first_distance < self.last_distance {
            let tolerance = self.required_tolerance();
            builder.update_vertex_tolerance(&repair.first_vertex, tolerance)?;
            repair.first_tolerance = tolerance;
            return Ok(());
        }
        let tolerance = self.required_tolerance();
        builder.update_vertex_tolerance(&repair.last_vertex, tolerance)?;
        repair.last_tolerance = tolerance;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape_analysis::ShapeAnalysisWire;
    use crate::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
    use rustycad_tkbrep::topods::{Builder, Edge, Face, Vertex};
    use rustycad_tkernel::ncollection::Array1;
    use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
    use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::gp::{Axis3, Point2, Point3};

    const EXPECTED: &str =
        include_str!("../../../../tools/oracle/tkshhealing_wire_self_intersection.expected");

    #[test]
    fn tolerance_repair_matches_native_after_geometry_removal_is_disabled() {
        let (analyzer, first, last) = loop_fixture();
        let fixer = ShapeFixWire::new();
        fixer.init(&analyzer);
        fixer.set_geometry_mode(false);
        fixer.set_max_tolerance(2.0);
        assert!(fixer.fix_self_intersecting_edge(1).unwrap());
        assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
        assert!(!fixer.last_fix_status(ShapeExtendStatus::Fail2));
        assert_close(
            BRepTool::vertex_tolerance(&first).unwrap(),
            expected("tol1"),
        );
        assert_close(BRepTool::vertex_tolerance(&last).unwrap(), expected("tol2"));
    }

    fn loop_fixture() -> (Handle<ShapeAnalysisWire>, Vertex, Vertex) {
        let face = planar_face();
        let (first, last) = loop_vertices();
        let edge = loop_edge(&face, &first, &last);
        let wire = Handle::new(ShapeExtendWireData::new());
        wire.add_edge(&edge, 0).unwrap();
        let analyzer = ShapeAnalysisWire::from_wire_data(&wire, &face, 1e-7).unwrap();
        (Handle::new(analyzer), first, last)
    }

    fn loop_vertices() -> (Vertex, Vertex) {
        let builder = BRepBuilder::default();
        let mut first = Vertex::default();
        let mut last = Vertex::default();
        builder
            .make_vertex_at(&mut first, Point3::default(), 1e-7)
            .unwrap();
        builder
            .make_vertex_at(&mut last, Point3::new(1.0, 0.0, 0.0), 1e-7)
            .unwrap();
        (first, last)
    }

    fn planar_face() -> Face {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
        let mut face = Face::default();
        BRepBuilder::default()
            .make_face_on_surface(&mut face, &surface, 1e-7)
            .unwrap();
        face
    }

    fn loop_edge(face: &Face, first: &Vertex, last: &Vertex) -> Edge {
        let builder = BRepBuilder::default();
        let mut edge = Edge::default();
        builder.make_edge(&mut edge).unwrap();
        Builder
            .add(&mut edge, &first.oriented(Orientation::Forward))
            .unwrap();
        Builder
            .add(&mut edge, &last.oriented(Orientation::Reversed))
            .unwrap();
        let poles = [
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 3.0),
            Point2::new(-3.0, 3.0),
            Point2::new(1.0, 0.0),
        ];
        let curve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dBezierCurve::new(&Array1::from_shared(&poles, 1)).unwrap()).cast();
        builder
            .update_edge_pcurve_on_face(&edge, &curve, face, 1e-7)
            .unwrap();
        builder.range_on_face(&edge, face, 0.0, 1.0).unwrap();
        edge
    }

    fn expected(label: &str) -> f64 {
        let row = EXPECTED
            .lines()
            .find(|line| line.starts_with("fix "))
            .unwrap();
        let mut values = row.split_whitespace();
        while values.next() != Some(label) {}
        values.next().unwrap().parse().unwrap()
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-14, "{actual} != {expected}");
    }
}
