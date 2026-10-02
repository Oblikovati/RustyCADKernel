use super::{BRepMeshDelaun, Failure, PolygonEdge, PolygonEdit, PolygonLoop};
use crate::brep_mesh::delaun::polygon_state::link_box;
impl BRepMeshDelaun {
    pub(super) fn replace_polygon_edge(
        &self,
        polygon: &mut PolygonLoop,
        nodes: [i32; 2],
        points: [rustycad_tkmath::gp::Xy; 2],
        index: usize,
        insert: bool,
    ) -> Result<i32, Failure> {
        let edge = self.add_free_link(nodes[0], nodes[1])?;
        let bounds = link_box(points[0], points[1]);
        if insert {
            polygon.links.insert_after(index, edge)?;
            polygon.boxes.insert_after(index, bounds)?;
        } else {
            polygon.replace(index, edge, bounds)?;
        }
        Ok(edge)
    }
    pub(super) fn apply_polygon_edit(
        &self,
        polygon: &mut PolygonLoop,
        current: &mut PolygonEdge,
        edit: &PolygonEdit,
    ) -> Result<(), Failure> {
        if edit.replace {
            current.edge = self
                .replace_polygon_edge(polygon, current.nodes, current.points, current.index, false)?
                .wrapping_abs();
        }
        let first = if edit.remove_first {
            current.index
        } else {
            current.index + 1
        };
        polygon.remove(first, edit.remove_to)
    }
}
