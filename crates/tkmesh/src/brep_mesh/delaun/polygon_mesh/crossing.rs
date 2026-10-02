use super::{BRepMeshDelaun, Failure, Map, PolygonEdge, PolygonEdit, PolygonLoop};
use crate::brep_mesh::IntersectionFlag;
use rustycad_tkmath::gp::{Point2, Xy};
impl BRepMeshDelaun {
    pub(super) fn correct_frontier_pair(
        &mut self,
        polygon: &mut PolygonLoop,
        current: &mut PolygonEdge,
        next: PolygonEdge,
        small: f64,
        skipped: &mut Option<&mut Map<i32>>,
    ) -> Result<Option<PolygonEdit>, Failure> {
        let mut point = Point2::default();
        let flag = self.edge_intersection(
            &self.get_edge(current.edge)?,
            &self.get_edge(next.edge)?,
            false,
            true,
            &mut point,
        )?;
        let mut edit = PolygonEdit {
            remove_to: next.index,
            remove_first: false,
            replace: true,
            finish: false,
        };
        match flag {
            IntersectionFlag::Cross => self.correct_crossing(
                polygon,
                current,
                next,
                point.xy(),
                small,
                skipped,
                &mut edit,
            )?,
            IntersectionFlag::PointOnSegment => {
                self.correct_touching(polygon, current, next, &mut edit)?
            }
            IntersectionFlag::Glued => {
                if current.nodes[1] == next.nodes[0] {
                    current.nodes[1] = next.nodes[1];
                    current.points[1] = next.points[1];
                }
            }
            IntersectionFlag::Same => {
                self.process_polygon_loop(current.index, next.index, polygon)?;
                edit.remove_first = true;
                edit.replace = false;
            }
            _ => return Ok(None),
        }
        Ok(Some(edit))
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn correct_crossing(
        &mut self,
        polygon: &mut PolygonLoop,
        current: &mut PolygonEdge,
        mut next: PolygonEdge,
        point: Xy,
        small: f64,
        skipped: &mut Option<&mut Map<i32>>,
        edit: &mut PolygonEdit,
    ) -> Result<(), Failure> {
        let mut area = self.polygon_area(
            &polygon.links,
            (current.index + 1) as i32,
            next.index as i32,
        )?;
        area += (current.points[1] - point).crossed(next.points[0] - point) / 2.;
        if area.abs() > small {
            next.nodes[1] = current.nodes[0];
            next.points[1] = current.points[0];
            self.replace_polygon_edge(polygon, next.nodes, next.points, next.index, false)?;
            self.process_polygon_loop(current.index, next.index, polygon)?;
            edit.finish = true;
            return Ok(());
        }
        let first =
            (point - next.points[0]).square_modulus() < (point - next.points[1]).square_modulus();
        let endpoint = usize::from(!first);
        current.nodes[1] = next.nodes[endpoint];
        current.points[1] = next.points[endpoint];
        if first {
            edit.remove_to -= 1;
        }
        if let Some(skip) = skipped {
            for index in current.index..=edit.remove_to {
                skip.add(polygon.links[index].wrapping_abs());
            }
        }
        Ok(())
    }
}
