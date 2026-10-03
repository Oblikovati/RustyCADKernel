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
    /// `FixSelfIntersectingEdge(num)`: remove a detected p-curve loop or raise its
    /// endpoint tolerance; mode one inserts a midpoint vertex and splits the edge.
    /// Example: `let fixed = fixer.fix_self_intersecting_edge(1)?;`.
    pub(super) fn fix_self_intersecting_edge(&self, index: i32) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer()?;
        if !analyzer.is_ready() {
            return Ok(false);
        }
        let (points_2d, points_3d) = self.analyze_self_intersections(&analyzer, index)?;
        self.transfer_self_intersection_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        let mut repair = SelfIntersectionRepair::load(&analyzer, index)?;
        if self.remove_loop_mode() == 1 {
            return self.split_self_intersection(&analyzer, index, &points_2d);
        }
        self.repair_intersections(&analyzer, index, &mut repair, points_2d, points_3d)?;
        Ok(self.last_fix_status(ShapeExtendStatus::Done))
    }

    fn repair_intersections(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        repair: &mut SelfIntersectionRepair,
        mut points_2d: Sequence<IntersectionPoint>,
        mut points_3d: Sequence<Point3>,
    ) -> Result<(), Failure> {
        self.repeat_loop_removal(analyzer, index, repair, &mut points_2d, &mut points_3d)
    }

    fn repeat_loop_removal(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        repair: &mut SelfIntersectionRepair,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
    ) -> Result<(), Failure> {
        if self.remove_loop_mode() >= 1 {
            return self.raise_intersection_tolerances(repair, points_3d);
        }
        for _ in 0..30 {
            if !self.repair_intersection_pass(analyzer, index, repair, points_2d, points_3d)? {
                break;
            }
            if !self.refresh_intersections(analyzer, index, points_2d, points_3d)? {
                break;
            }
        }
        Ok(())
    }

    fn repair_intersection_pass(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        repair: &mut SelfIntersectionRepair,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
    ) -> Result<bool, Failure> {
        let edge_index = SelfIntersectionRepair::edge_index(analyzer, index)?;
        let edge = analyzer.wire_data().edge(edge_index)?;
        let face = analyzer.face();
        let mut removed = false;
        let mut previous = (0.0, 0.0);
        for point_index in 1..=points_2d.len() {
            let point_2d = *points_2d.value(point_index)?;
            let point_3d = *points_3d.value(point_index)?;
            if self.try_remove_intersection(&edge, &face, &point_2d, point_3d, repair, previous)? {
                previous = (point_2d.param_on_first(), point_2d.param_on_second());
                removed = true;
                continue;
            }
            self.raise_nearest_vertex_tolerance(repair, point_3d)?;
        }
        Ok(removed)
    }

    fn try_remove_intersection(
        &self,
        edge: &rustycad_tkbrep::topods::Edge,
        face: &rustycad_tkbrep::topods::Face,
        intersection: &IntersectionPoint,
        point: Point3,
        repair: &SelfIntersectionRepair,
        previous: (f64, f64),
    ) -> Result<bool, Failure> {
        if !self.geometry_mode()
            || SelfIntersectionGap::measure(repair, point).already_covered(repair)
        {
            return Ok(false);
        }
        let current = (
            intersection.param_on_first(),
            intersection.param_on_second(),
        );
        if current.0 > previous.0 && current.1 < previous.1 {
            return Ok(false);
        }
        let tolerance = SelfIntersectionGap::measure(repair, point)
            .required_tolerance()
            .max(self.precision())
            .min(self.max_tolerance());
        self.try_remove_loop(
            edge,
            face,
            intersection,
            0.1,
            tolerance,
            self.remove_loop_mode() == 0,
        )
    }

    fn refresh_intersections(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
    ) -> Result<bool, Failure> {
        points_2d.clear(None);
        points_3d.clear(None);
        analyzer.check_self_intersecting_edge_into(index, points_2d, points_3d)?;
        Ok(analyzer.last_check_status(ShapeExtendStatus::Done))
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
    fn edge_index(analyzer: &ShapeAnalysisWire, index: i32) -> Result<i32, Failure> {
        let wire: Handle<ShapeExtendWireData> = analyzer.wire_data();
        Ok(if index > 0 { index } else { wire.nb_edges() })
    }

    fn load(analyzer: &ShapeAnalysisWire, index: i32) -> Result<Self, Failure> {
        let wire: Handle<ShapeExtendWireData> = analyzer.wire_data();
        let edge_index = Self::edge_index(analyzer, index)?;
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
#[path = "wire_self_intersection_fix_tests.rs"]
mod tests;
