//! Private polygon primitives used by the still-unported constraint orchestration.
use super::{BRepMeshDelaun, BRepMeshEdge};
use crate::brep_mesh::{GeomTool, IntersectionFlag};
use rustycad_tkernel::{
    ncollection::{DynamicArray, Sequence},
    standard::Failure,
};
use rustycad_tkmath::gp::{Point2, Vector2, Xy};
impl BRepMeshDelaun {
    /// Internal winding predicate; the input must explicitly repeat its first vertex to close the loop. `triangulator.polygon_contains_vertex(1,&vertices)?`.
    #[doc(hidden)]
    pub fn polygon_contains_vertex(
        &self,
        index: i32,
        polygon: &DynamicArray<i32>,
    ) -> Result<bool, Failure> {
        if polygon.length() < 3 {
            return Ok(false);
        }
        let center = *self.get_vertex(index)?.coord();
        let mut vertices = polygon.iter();
        let first = *vertices.next().expect("polygon length >= 3");
        let previous = Vector2::from(*self.get_vertex(first)?.coord() - center);
        if near_polygon_vertex(previous) {
            return Ok(true);
        }
        self.winding_from_vertices(center, previous, vertices.copied())
    }
    fn winding_from_vertices(
        &self,
        center: Xy,
        mut previous: Vector2,
        vertices: impl Iterator<Item = i32>,
    ) -> Result<bool, Failure> {
        let mut angle = 0.;
        for vertex in vertices {
            let current = Vector2::from(*self.get_vertex(vertex)?.coord() - center);
            if near_polygon_vertex(current) {
                return Ok(true);
            }
            angle += current.angle(previous)?;
            previous = current;
        }
        Ok((angle.abs() - 2. * std::f64::consts::PI).abs()
            <= rustycad_tkernel::precision::angular())
    }
    /// Internal signed loop area with native one-based sequence bounds; `triangulator.polygon_area(&edges,1,4)?`.
    #[doc(hidden)]
    pub fn polygon_area(
        &self,
        polygon: &Sequence<i32>,
        start: i32,
        end: i32,
    ) -> Result<f64, Failure> {
        if start >= end || start > polygon.length() {
            return Ok(0.);
        }
        let [origin, _] = self.polygon_edge_points(*polygon.value(start as usize)?)?;
        let mut area = 0.;
        for index in start + 1..=end {
            let [a, b] = self.polygon_edge_points(*polygon.value(index as usize)?)?;
            area += (a - origin).crossed(b - origin);
        }
        Ok(area / 2.)
    }
    /// Internal segment query through mesh endpoints; `triangulator.edge_intersection(&a,&b,true,true,&mut point)?`.
    #[doc(hidden)]
    pub fn edge_intersection(
        &self,
        a: &BRepMeshEdge,
        b: &BRepMeshEdge,
        end_touch: bool,
        on_edge: bool,
        point: &mut Point2,
    ) -> Result<IntersectionFlag, Failure> {
        let first = [
            *self.get_vertex(a.first_node())?.coord(),
            *self.get_vertex(a.last_node())?.coord(),
        ];
        let second = [
            *self.get_vertex(b.first_node())?.coord(),
            *self.get_vertex(b.last_node())?.coord(),
        ];
        Ok(GeomTool::int_seg_seg(
            first[0], first[1], second[0], second[1], end_touch, on_edge, point,
        ))
    }
    pub(super) fn polygon_edge_points(&self, index: i32) -> Result<[Xy; 2], Failure> {
        let edge = self.get_edge(index.wrapping_abs())?;
        let mut nodes = [edge.first_node(), edge.last_node()];
        if index <= 0 {
            nodes.swap(0, 1);
        }
        Ok([
            *self.get_vertex(nodes[0])?.coord(),
            *self.get_vertex(nodes[1])?.coord(),
        ])
    }
}
fn near_polygon_vertex(direction: Vector2) -> bool {
    let precision = rustycad_tkernel::precision::p_confusion();
    direction.square_magnitude() < precision * precision
}
