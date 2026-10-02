//! Native minimum-distance/angle polygon decomposition, including FIFO cut processing.
use super::polygon_state::{PolygonLoop, link_box};
use super::{BRepMeshDelaun, BRepMeshEdge, DegreeOfFreedom};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2, Xy};
struct PivotChoice {
    used: usize,
    nodes: [i32; 3],
    points: [Xy; 3],
    angle: f64,
    distance: f64,
}
impl BRepMeshDelaun {
    pub(super) fn mesh_elementary_polygon(
        &mut self,
        polygon: &PolygonLoop,
    ) -> Result<bool, Failure> {
        if polygon.len() < 3 {
            return Ok(true);
        }
        if polygon.len() > 3 {
            return Ok(false);
        }
        let edges = [polygon.links[1], polygon.links[2], polygon.links[3]];
        let first = self.get_edge(edges[0].wrapping_abs())?;
        let second = self.get_edge(edges[1].wrapping_abs())?;
        let mut nodes = [first.first_node(), first.last_node(), second.first_node()];
        if nodes[2] == nodes[0] || nodes[2] == nodes[1] {
            nodes[2] = second.last_node();
        }
        self.add_signed_triangle(edges, nodes)?;
        Ok(true)
    }
    pub(super) fn decompose_polygon(
        &mut self,
        polygon: &mut PolygonLoop,
    ) -> Result<PolygonLoop, Failure> {
        if self.mesh_elementary_polygon(polygon)? {
            polygon.clear();
            return Ok(PolygonLoop::default());
        }
        let first = polygon.links[1];
        let nodes = self.oriented_nodes(first)?;
        let points = [
            *self.get_vertex(nodes[0])?.coord(),
            *self.get_vertex(nodes[1])?.coord(),
            Xy::default(),
        ];
        let direction = points[1] - points[0];
        let length = direction.modulus();
        if length < rustycad_tkernel::precision::p_confusion() {
            polygon.clear();
            return Ok(PolygonLoop::default());
        }
        let mut choice = PivotChoice {
            used: 0,
            nodes: [nodes[0], nodes[1], 0],
            points,
            angle: 0.,
            distance: f64::MAX,
        };
        self.choose_polygon_pivot(polygon, Vector2::from(direction / length), &mut choice)?;
        if choice.used == 0 {
            polygon.clear();
            return Ok(PolygonLoop::default());
        }
        let edges = [
            first,
            self.add_free_link(choice.nodes[1], choice.nodes[2])?,
            self.add_free_link(choice.nodes[2], choice.nodes[0])?,
        ];
        self.add_signed_triangle(edges, choice.nodes)?;
        update_polygon_cut(polygon, &choice, edges)
    }
    fn choose_polygon_pivot(
        &self,
        polygon: &PolygonLoop,
        direction: Vector2,
        choice: &mut PivotChoice,
    ) -> Result<(), Failure> {
        for index in 3..=polygon.len() {
            let node = self.oriented_nodes(polygon.links[index])?[0];
            if node == choice.nodes[1] {
                continue;
            }
            let point = *self.get_vertex(node)?.coord();
            let displacement = Vector2::from(point - choice.points[1]);
            let distance = direction.xy().crossed(displacement.xy());
            let angle = direction.angle(displacement)?.abs();
            if !pivot_improves(choice, distance, angle) {
                continue;
            }
            if self.pivot_intersects(polygon, index, node, point, choice)? {
                continue;
            }
            choice.angle = angle;
            choice.distance = distance.abs();
            choice.nodes[2] = node;
            choice.points[2] = point;
            choice.used = index;
        }
        Ok(())
    }
    fn pivot_intersects(
        &self,
        polygon: &PolygonLoop,
        index: usize,
        node: i32,
        point: Xy,
        choice: &PivotChoice,
    ) -> Result<bool, Failure> {
        for endpoint in 0..2 {
            let edge = BRepMeshEdge::new(choice.nodes[endpoint], node, DegreeOfFreedom::Free);
            let bounds = link_box(choice.points[endpoint], point);
            if self.pivot_edge_intersects(polygon, index, &edge, &bounds)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn pivot_edge_intersects(
        &self,
        polygon: &PolygonLoop,
        skip: usize,
        edge: &BRepMeshEdge,
        bounds: &rustycad_tkmath::bnd::CompactBox2,
    ) -> Result<bool, Failure> {
        for index in 2..=polygon.len() {
            if index == skip || bounds.is_out_box(&polygon.boxes[index]) {
                continue;
            }
            let other = self.get_edge(polygon.links[index].wrapping_abs())?;
            if edge.is_equal(&other) {
                continue;
            }
            if self.edge_intersection(edge, &other, false, false, &mut Point2::default())?
                != crate::brep_mesh::IntersectionFlag::NoIntersection
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
fn pivot_improves(choice: &PivotChoice, distance: f64, angle: f64) -> bool {
    if distance.abs() < rustycad_tkernel::precision::p_confusion() || distance < 0. {
        return false;
    }
    !(distance.abs() >= choice.distance
        && (angle <= choice.angle || angle > 90. * (std::f64::consts::PI / 180.)))
}
fn update_polygon_cut(
    polygon: &mut PolygonLoop,
    choice: &PivotChoice,
    edges: [i32; 3],
) -> Result<PolygonLoop, Failure> {
    let mut cut = PolygonLoop::default();
    let box02 = link_box(choice.points[0], choice.points[2]);
    if choice.used == 3 {
        polygon.remove(1, 1)?;
        polygon.replace(1, -edges[2], box02)?;
        return Ok(cut);
    }
    if choice.used < polygon.len() {
        cut = polygon.split(choice.used)?;
        cut.prepend(-edges[2], box02);
    } else {
        polygon.remove(polygon.len(), polygon.len())?;
    }
    polygon.replace(1, -edges[1], link_box(choice.points[1], choice.points[2]))?;
    Ok(cut)
}
