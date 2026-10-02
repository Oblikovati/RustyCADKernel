//! Boundary-walk neighbor selection used while recovering constraint loops.
use super::{BRepMeshDelaun, BRepMeshVertex, DegreeOfFreedom};
use rustycad_tkernel::{
    ncollection::{Map, Sequence},
    standard::Failure,
};
use rustycad_tkmath::{bnd::CompactBox2, gp::Vector2};

impl BRepMeshDelaun {
    /// Select the greatest-angle visible neighbor; `delaun.find_next_polygon_link(...)?`.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)] // Mirrors the pinned private native member.
    pub fn find_next_polygon_link(
        &self,
        first_node: i32,
        pivot_node: i32,
        pivot_vertex: &BRepMeshVertex,
        reference_direction: Vector2,
        boxes: &Sequence<CompactBox2>,
        polygon: &Sequence<i32>,
        skipped: Option<&Map<i32>>,
        skip_leprous: bool,
        leprous_links: &mut Map<i32>,
        dead_links: &mut Map<i32>,
        next_pivot_node: &mut i32,
        next_link_direction: &mut Vector2,
        next_link_bounds: &mut CompactBox2,
    ) -> Result<i32, Failure> {
        let mut selection = CandidateSelection {
            max_angle: f64::MIN,
            ..CandidateSelection::default()
        };
        for neighbor in self.mesh_ref()?.links_connected_to(pivot_node)?.iter() {
            self.consider_neighbor(
                *neighbor,
                first_node,
                pivot_node,
                pivot_vertex,
                reference_direction,
                boxes,
                polygon,
                skipped,
                skip_leprous,
                leprous_links,
                dead_links,
                &mut selection,
            )?;
        }
        selection.write(next_pivot_node, next_link_direction, next_link_bounds);
        Ok(selection.signed_link)
    }

    #[allow(clippy::too_many_arguments)] // Mirrors native availability and selection state.
    fn consider_neighbor(
        &self,
        neighbor: i32,
        first_node: i32,
        pivot_node: i32,
        pivot_vertex: &BRepMeshVertex,
        reference_direction: Vector2,
        boxes: &Sequence<CompactBox2>,
        polygon: &Sequence<i32>,
        skipped: Option<&Map<i32>>,
        skip_leprous: bool,
        leprous_links: &mut Map<i32>,
        dead_links: &mut Map<i32>,
        selection: &mut CandidateSelection,
    ) -> Result<(), Failure> {
        let link_index = neighbor.wrapping_abs();
        if unavailable_link(link_index, skipped, dead_links, leprous_links, skip_leprous) {
            return Ok(());
        }
        self.consider_available_neighbor(
            neighbor,
            link_index,
            first_node,
            pivot_node,
            pivot_vertex,
            reference_direction,
            boxes,
            polygon,
            leprous_links,
            dead_links,
            selection,
        )
    }

    #[allow(clippy::too_many_arguments)] // Keeps the native candidate context explicit.
    fn consider_available_neighbor(
        &self,
        neighbor: i32,
        link_index: i32,
        first_node: i32,
        pivot_node: i32,
        pivot_vertex: &BRepMeshVertex,
        reference_direction: Vector2,
        boxes: &Sequence<CompactBox2>,
        polygon: &Sequence<i32>,
        leprous_links: &mut Map<i32>,
        dead_links: &mut Map<i32>,
        selection: &mut CandidateSelection,
    ) -> Result<(), Failure> {
        let edge = self.get_edge(link_index)?;
        if self.is_dead_free_link(link_index, edge.movability())? {
            dead_links.add(link_index);
            return Ok(());
        }
        let other_node = other_endpoint(&edge, pivot_node);
        let direction =
            Vector2::from(*self.get_vertex(other_node)?.coord() - *pivot_vertex.coord());
        self.consider_direction(
            neighbor,
            first_node,
            other_node,
            direction,
            &edge,
            reference_direction,
            boxes,
            polygon,
            leprous_links,
            dead_links,
            selection,
        )
    }

    fn is_dead_free_link(
        &self,
        link_index: i32,
        movability: DegreeOfFreedom,
    ) -> Result<bool, Failure> {
        Ok(movability == DegreeOfFreedom::Free
            && self
                .mesh_ref()?
                .elements_connected_to(link_index)?
                .is_empty())
    }

    #[allow(clippy::too_many_arguments)] // Mirrors the native neighbor decision inputs.
    fn consider_direction(
        &self,
        neighbor: i32,
        first_node: i32,
        other_node: i32,
        direction: Vector2,
        edge: &super::BRepMeshEdge,
        reference_direction: Vector2,
        boxes: &Sequence<CompactBox2>,
        polygon: &Sequence<i32>,
        leprous_links: &mut Map<i32>,
        dead_links: &mut Map<i32>,
        selection: &mut CandidateSelection,
    ) -> Result<(), Failure> {
        if direction.square_magnitude() < rustycad_tkernel::precision::square_p_confusion() {
            dead_links.add(neighbor.wrapping_abs());
            return Ok(());
        }
        leprous_links.add(neighbor.wrapping_abs());
        let angle = self.candidate_angle(edge.movability(), reference_direction, direction)?;
        if angle <= selection.max_angle {
            return Ok(());
        }
        self.select_if_visible(
            neighbor,
            first_node,
            other_node,
            direction,
            edge,
            angle,
            angle.abs() > std::f64::consts::PI - rustycad_tkernel::precision::angular(),
            boxes,
            polygon,
            selection,
        )
    }

    fn candidate_angle(
        &self,
        movability: DegreeOfFreedom,
        reference: Vector2,
        direction: Vector2,
    ) -> Result<f64, Failure> {
        let angle = reference.angle(direction)?;
        if movability == DegreeOfFreedom::Frontier && glued_angle(angle) {
            return Ok(angle.abs());
        }
        Ok(angle)
    }

    #[allow(clippy::too_many_arguments)] // Carries the native intersection predicate inputs.
    fn select_if_visible(
        &self,
        neighbor: i32,
        first_node: i32,
        other_node: i32,
        direction: Vector2,
        edge: &super::BRepMeshEdge,
        angle: f64,
        glued: bool,
        boxes: &Sequence<CompactBox2>,
        polygon: &Sequence<i32>,
        selection: &mut CandidateSelection,
    ) -> Result<(), Failure> {
        let mut bounds = CompactBox2::default();
        let visible = self.check_intersection(
            edge,
            polygon,
            boxes,
            other_node != first_node,
            !glued,
            true,
            &mut bounds,
        )?;
        if visible {
            selection.replace(neighbor, edge, other_node, direction, angle, bounds);
        }
        Ok(())
    }
}

#[derive(Default)]
struct CandidateSelection {
    max_angle: f64,
    signed_link: i32,
    next_pivot: i32,
    direction: Vector2,
    bounds: CompactBox2,
}

impl CandidateSelection {
    fn replace(
        &mut self,
        neighbor: i32,
        edge: &super::BRepMeshEdge,
        other: i32,
        direction: Vector2,
        angle: f64,
        bounds: CompactBox2,
    ) {
        self.max_angle = angle;
        self.signed_link = if edge.first_node() == other {
            -neighbor.wrapping_abs()
        } else {
            neighbor.wrapping_abs()
        };
        self.next_pivot = other;
        self.direction = direction;
        self.bounds = bounds;
    }

    fn write(&self, next_pivot: &mut i32, direction: &mut Vector2, bounds: &mut CompactBox2) {
        if self.signed_link == 0 {
            return;
        }
        *next_pivot = self.next_pivot;
        *direction = self.direction;
        *bounds = self.bounds;
    }
}

fn unavailable_link(
    index: i32,
    skipped: Option<&Map<i32>>,
    dead: &Map<i32>,
    leprous: &Map<i32>,
    skip: bool,
) -> bool {
    dead.contains(&index)
        || skipped.is_some_and(|map| map.contains(&index))
        || (skip && leprous.contains(&index))
}

fn other_endpoint(edge: &super::BRepMeshEdge, pivot: i32) -> i32 {
    if edge.first_node() == pivot {
        edge.last_node()
    } else {
        edge.first_node()
    }
}

fn glued_angle(angle: f64) -> bool {
    (angle.abs() - std::f64::consts::PI).abs() < rustycad_tkernel::precision::angular()
}
