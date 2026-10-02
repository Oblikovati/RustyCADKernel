//! Constraint-loop recovery and decomposition of the polygon on one side of an edge.
use super::polygon_state::{PolygonLoop, link_box};
use super::{BRepMeshDelaun, BRepMeshVertex};
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{bnd::CompactBox2, gp::Vector2};

impl BRepMeshDelaun {
    /// Recover and triangulate the loop to one side of a constraint; `delaun.mesh_left_polygon(1,true,None)?`.
    #[doc(hidden)]
    pub fn mesh_left_polygon(
        &mut self,
        start_link: i32,
        forward: bool,
        skipped: Option<&mut Map<i32>>,
    ) -> Result<bool, Failure> {
        if skipped
            .as_deref()
            .is_some_and(|map| map.contains(&start_link))
        {
            return Ok(true);
        }
        let Some(mut polygon) =
            self.collect_left_polygon(start_link, forward, skipped.as_deref())?
        else {
            return Ok(false);
        };
        self.cleanup_polygon(&polygon)?;
        self.mesh_polygon(&mut polygon, skipped)?;
        Ok(true)
    }

    fn collect_left_polygon(
        &self,
        start_link: i32,
        forward: bool,
        skipped: Option<&Map<i32>>,
    ) -> Result<Option<PolygonLoop>, Failure> {
        let edge = self.get_edge(start_link.wrapping_abs())?;
        let (start, pivot) = oriented_nodes(&edge, forward);
        let mut polygon = PolygonLoop::default();
        let signed_start = if forward { start_link } else { -start_link };
        let (start_vertex, mut pivot_vertex) = (self.get_vertex(start)?, self.get_vertex(pivot)?);
        let mut reference = vertex_direction(&start_vertex, &pivot_vertex);
        if too_short(reference) {
            return Ok(Some(polygon));
        }
        polygon.append(
            signed_start,
            link_box(*start_vertex.coord(), *pivot_vertex.coord()),
        );
        let mut state = PolygonWalkState::new(start, pivot, signed_start);
        while state.pivot != state.first {
            let found = self.find_polygon_successor(
                &mut state,
                &pivot_vertex,
                reference,
                &polygon,
                skipped,
            )?;
            if let Some(successor) = found {
                state.advance(successor.pivot);
                pivot_vertex = successor.vertex;
                reference = successor.direction;
                polygon.append(successor.link, successor.bounds);
                continue;
            }
            if !state.backtrack(&mut polygon)? {
                return Ok(None);
            }
            let (new_start, new_pivot) = state.last_oriented_edge(&polygon, self)?;
            pivot_vertex = self.get_vertex(new_pivot)?;
            reference = vertex_direction(&self.get_vertex(new_start)?, &pivot_vertex);
        }
        if polygon.len() < 3 {
            return Ok(None);
        }
        Ok(Some(polygon))
    }

    fn find_polygon_successor(
        &self,
        state: &mut PolygonWalkState,
        pivot_vertex: &BRepMeshVertex,
        reference: Vector2,
        polygon: &PolygonLoop,
        skipped: Option<&Map<i32>>,
    ) -> Result<Option<PolygonSuccessor>, Failure> {
        let mut next_pivot = 0;
        let mut direction = Vector2::default();
        let mut bounds = CompactBox2::default();
        let next_link = self.find_next_polygon_link(
            state.first,
            state.pivot,
            pivot_vertex,
            reference,
            &polygon.boxes,
            &polygon.links,
            skipped,
            state.skip_leprous,
            &mut state.leprous,
            &mut state.dead,
            &mut next_pivot,
            &mut direction,
            &mut bounds,
        )?;
        if next_link == 0 {
            return Ok(None);
        }
        Ok(Some(PolygonSuccessor {
            link: next_link,
            pivot: next_pivot,
            vertex: self.get_vertex(next_pivot)?,
            direction,
            bounds,
        }))
    }
}

struct PolygonWalkState {
    first: i32,
    pivot: i32,
    skip_leprous: bool,
    dead: Map<i32>,
    leprous: Map<i32>,
}

struct PolygonSuccessor {
    link: i32,
    pivot: i32,
    vertex: BRepMeshVertex,
    direction: Vector2,
    bounds: CompactBox2,
}

impl PolygonWalkState {
    fn new(first: i32, pivot: i32, start_link: i32) -> Self {
        let mut leprous = Map::default();
        leprous.add(start_link.wrapping_abs());
        Self {
            first,
            pivot,
            skip_leprous: true,
            dead: Map::default(),
            leprous,
        }
    }

    fn advance(&mut self, pivot: i32) {
        self.pivot = pivot;
        self.skip_leprous = true;
    }

    fn backtrack(&mut self, polygon: &mut PolygonLoop) -> Result<bool, Failure> {
        if polygon.len() == 1 {
            return Ok(false);
        }
        let previous_length = polygon.len();
        let dead = polygon.links.last().copied().unwrap().wrapping_abs();
        self.dead.add(dead);
        self.leprous.remove(&dead);
        polygon.links.remove(previous_length)?;
        polygon.boxes.remove(previous_length)?;
        self.skip_leprous = false;
        Ok(true)
    }

    fn last_oriented_edge(
        &self,
        polygon: &PolygonLoop,
        delaun: &BRepMeshDelaun,
    ) -> Result<(i32, i32), Failure> {
        let info = *polygon.links.last().ok_or_else(|| {
            Failure::out_of_range("polygon backtrack has no previous edge; expected nonempty loop")
        })?;
        let edge = delaun.get_edge(info.wrapping_abs())?;
        Ok(oriented_nodes(&edge, info > 0))
    }
}

fn oriented_nodes(edge: &super::BRepMeshEdge, forward: bool) -> (i32, i32) {
    if forward {
        (edge.first_node(), edge.last_node())
    } else {
        (edge.last_node(), edge.first_node())
    }
}

fn vertex_direction(start: &BRepMeshVertex, end: &BRepMeshVertex) -> Vector2 {
    Vector2::from(*end.coord() - *start.coord())
}

fn too_short(direction: Vector2) -> bool {
    direction.square_magnitude() < rustycad_tkernel::precision::square_p_confusion()
}
