use super::{BRepMeshDelaun, Failure, PolygonEdge, PolygonEdit, PolygonLoop};
impl BRepMeshDelaun {
    pub(super) fn correct_touching(
        &mut self,
        polygon: &mut PolygonLoop,
        current: &mut PolygonEdge,
        next: PolygonEdge,
        edit: &mut PolygonEdit,
    ) -> Result<(), Failure> {
        let chopping = (0..2).find(|i| {
            (next.points[0] - current.points[*i])
                .crossed(next.points[1] - current.points[*i])
                .abs()
                < rustycad_tkernel::precision::p_confusion()
        });
        if let Some(index) = chopping {
            edit.replace = false;
            edit.remove_first = index == 0;
            self.replace_polygon_edge(
                polygon,
                [next.nodes[0], current.nodes[index]],
                [next.points[0], current.points[index]],
                next.index,
                false,
            )?;
            self.replace_polygon_edge(
                polygon,
                [current.nodes[index], next.nodes[1]],
                [current.points[index], next.points[1]],
                next.index,
                true,
            )?;
            return self.process_polygon_loop(current.index + index, edit.remove_to, polygon);
        }
        self.replace_polygon_edge(
            polygon,
            [next.nodes[1], current.nodes[1]],
            [next.points[1], current.points[1]],
            current.index,
            true,
        )?;
        current.nodes[1] = next.nodes[1];
        current.points[1] = next.points[1];
        edit.remove_to += 1;
        self.process_polygon_loop(current.index + 1, edit.remove_to, polygon)
    }
}
