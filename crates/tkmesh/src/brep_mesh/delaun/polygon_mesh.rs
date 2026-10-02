//! Frontier self-intersection correction before native polygon ear decomposition.
use super::polygon_state::PolygonLoop;
use super::{BRepMeshDelaun, DegreeOfFreedom};
use rustycad_tkernel::{
    ncollection::{Map, Sequence},
    standard::Failure,
};
mod crossing;
mod edits;
mod touching;
#[derive(Clone, Copy)]
struct PolygonEdge {
    index: usize,
    nodes: [i32; 2],
    points: [rustycad_tkmath::gp::Xy; 2],
    edge: i32,
}
struct PolygonEdit {
    remove_to: usize,
    remove_first: bool,
    replace: bool,
    finish: bool,
}
impl BRepMeshDelaun {
    /// Internal native polygon meshing entry for differential probes; `triangulator.mesh_polygon_links(&mut links)?`.
    #[doc(hidden)]
    pub fn mesh_polygon_links(&mut self, links: &mut Sequence<i32>) -> Result<(), Failure> {
        let mut polygon = self.boxed_polygon(links)?;
        let result = self.mesh_polygon(&mut polygon, None);
        *links = polygon.links;
        result
    }
    pub(super) fn mesh_polygon(
        &mut self,
        polygon: &mut PolygonLoop,
        mut skipped: Option<&mut Map<i32>>,
    ) -> Result<(), Failure> {
        if self.mesh_elementary_polygon(polygon)? {
            return Ok(());
        }
        let area = self
            .polygon_area(&polygon.links, 1, polygon.len() as i32)?
            .abs();
        if self.correct_polygon_frontiers(polygon, 0.001 * area, &mut skipped)? {
            return Ok(());
        }
        self.decompose_all_polygons(polygon)
    }
    fn decompose_all_polygons(&mut self, polygon: &mut PolygonLoop) -> Result<(), Failure> {
        let mut pending = std::collections::VecDeque::new();
        while polygon.len() != 0 {
            let cut = self.decompose_polygon(polygon)?;
            if cut.len() != 0 {
                pending.push_back(cut);
            }
        }
        while let Some(mut current) = pending.pop_front() {
            while current.len() != 0 {
                let cut = self.decompose_polygon(&mut current)?;
                if cut.len() != 0 {
                    pending.push_back(cut);
                }
            }
        }
        Ok(())
    }
    fn polygon_edge(&self, polygon: &PolygonLoop, index: usize) -> Result<PolygonEdge, Failure> {
        let info = polygon.links[index];
        let nodes = self.oriented_nodes(info)?;
        let points = [
            *self.get_vertex(nodes[0])?.coord(),
            *self.get_vertex(nodes[1])?.coord(),
        ];
        Ok(PolygonEdge {
            index,
            nodes,
            points,
            edge: info.wrapping_abs(),
        })
    }
    fn correct_polygon_frontiers(
        &mut self,
        polygon: &mut PolygonLoop,
        small_area: f64,
        skipped: &mut Option<&mut Map<i32>>,
    ) -> Result<bool, Failure> {
        let mut index = 1;
        while index < polygon.len() {
            let mut current = self.polygon_edge(polygon, index)?;
            if self.get_edge(current.edge)?.movability() != DegreeOfFreedom::Frontier {
                index += 1;
                continue;
            }
            let (removed, finished) =
                self.correct_frontier_neighbors(polygon, &mut current, small_area, skipped)?;
            if finished {
                return Ok(true);
            }
            if !removed {
                index += 1;
            }
        }
        Ok(false)
    }
    fn correct_frontier_neighbors(
        &mut self,
        polygon: &mut PolygonLoop,
        current: &mut PolygonEdge,
        small_area: f64,
        skipped: &mut Option<&mut Map<i32>>,
    ) -> Result<(bool, bool), Failure> {
        let mut next_index = current.index + 1;
        while next_index <= polygon.len() {
            let next = self.polygon_edge(polygon, next_index)?;
            if self.get_edge(next.edge)?.movability() != DegreeOfFreedom::Frontier {
                next_index += 1;
                continue;
            }
            let Some(edit) =
                self.correct_frontier_pair(polygon, current, next, small_area, skipped)?
            else {
                next_index += 1;
                continue;
            };
            if edit.finish {
                return Ok((false, true));
            }
            self.apply_polygon_edit(polygon, current, &edit)?;
            if edit.remove_first {
                return Ok((true, false));
            }
            next_index = current.index + 1;
        }
        Ok((false, false))
    }
    fn process_polygon_loop(
        &mut self,
        from: usize,
        to: usize,
        polygon: &PolygonLoop,
    ) -> Result<(), Failure> {
        if to <= from + 3 {
            return Ok(());
        }
        let mut closed = PolygonLoop::default();
        for index in from + 1..to {
            closed.append(polygon.links[index], polygon.boxes[index]);
        }
        self.mesh_polygon(&mut closed, None)
    }
}
