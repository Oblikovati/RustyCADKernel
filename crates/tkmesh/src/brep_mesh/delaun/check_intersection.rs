//! Segment intersection and compact bounds gate used while walking constraint loops.
use super::BRepMeshDelaun;
use super::polygon_state::link_box;
use crate::brep_mesh::IntersectionFlag;
use rustycad_tkernel::{ncollection::Sequence, standard::Failure};
use rustycad_tkmath::{bnd::CompactBox2, gp::Point2};

impl BRepMeshDelaun {
    /// Reject candidate edges crossing polygon boundaries; `delaun.check_intersection(&edge,&loop,&boxes,true,true,true,&mut bounds)?`.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)] // Mirrors the pinned native member signature.
    pub fn check_intersection(
        &self,
        link: &super::BRepMeshEdge,
        polygon: &Sequence<i32>,
        boxes: &Sequence<CompactBox2>,
        endpoint_touch: bool,
        point_on_edge: bool,
        skip_last: bool,
        link_bounds: &mut CompactBox2,
    ) -> Result<bool, Failure> {
        *link_bounds = self.edge_bounds(link)?;
        let count = polygon.length() - i32::from(skip_last);
        self.check_polygon_edges(link, polygon, boxes, count, endpoint_touch, point_on_edge)
    }

    fn edge_bounds(&self, link: &super::BRepMeshEdge) -> Result<CompactBox2, Failure> {
        let first = *self.get_vertex(link.first_node())?.coord();
        let last = *self.get_vertex(link.last_node())?.coord();
        Ok(link_box(first, last))
    }

    fn check_polygon_edges(
        &self,
        link: &super::BRepMeshEdge,
        polygon: &Sequence<i32>,
        boxes: &Sequence<CompactBox2>,
        count: i32,
        endpoint_touch: bool,
        point_on_edge: bool,
    ) -> Result<bool, Failure> {
        for index in 1..=count {
            if !self.edge_bounds_overlap(link, boxes.value(index as usize)?)? {
                continue;
            }
            if self.intersects_polygon_edge(
                link,
                *polygon.value(index as usize)?,
                endpoint_touch,
                point_on_edge,
            )? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn edge_bounds_overlap(
        &self,
        link: &super::BRepMeshEdge,
        other: &CompactBox2,
    ) -> Result<bool, Failure> {
        let current = self.edge_bounds(link)?;
        let centers = [current.center(), other.center()];
        let halves = [current.half_size(), other.half_size()];
        Ok((0..2).all(|axis| {
            (centers[0][axis] - centers[1][axis]).abs() <= halves[0][axis] + halves[1][axis]
        }))
    }

    fn intersects_polygon_edge(
        &self,
        link: &super::BRepMeshEdge,
        polygon_index: i32,
        endpoint_touch: bool,
        point_on_edge: bool,
    ) -> Result<bool, Failure> {
        let polygon_edge = self.get_edge(polygon_index.wrapping_abs())?;
        if link.movability() == super::DegreeOfFreedom::Frontier
            && polygon_edge.movability() == super::DegreeOfFreedom::Frontier
        {
            return Ok(false);
        }
        let mut point = Point2::default();
        Ok(self.edge_intersection(
            link,
            &polygon_edge,
            endpoint_touch,
            point_on_edge,
            &mut point,
        )? != IntersectionFlag::NoIntersection)
    }
}
