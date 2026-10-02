//! Native cavity removal before triangulating a recovered constraint loop.
use super::polygon_state::PolygonLoop;
use super::triangle_build::{LoopEdges, loop_edges};
use super::{BRepMeshDelaun, DegreeOfFreedom};
use rustycad_tkernel::ncollection::{DynamicArray, Map, Sequence};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::CompactBox2;

impl BRepMeshDelaun {
    /// Probe exact cavity cleanup for a signed polygon boundary.
    #[doc(hidden)]
    pub fn cleanup_polygon_probe(&mut self, links: &Sequence<i32>) -> Result<(), Failure> {
        let polygon = self.boxed_polygon(links)?;
        self.cleanup_polygon(&polygon)
    }

    pub(super) fn cleanup_polygon(&mut self, polygon: &PolygonLoop) -> Result<(), Failure> {
        if polygon.len() < 3 {
            return Ok(());
        }
        let mut polygon_boundary = self.collect_polygon_boundary(polygon)?;
        let mut survivors = copy_integer_set(&polygon_boundary.ignored);
        self.remove_polygon_cavity(
            &polygon_boundary.vertices,
            &polygon_boundary.vertex_set,
            polygon,
            &mut survivors,
            &mut polygon_boundary.links,
        )?;
        self.remove_unattached_loop_edges(&polygon_boundary.ignored, &polygon_boundary.links)
    }

    fn collect_polygon_boundary(
        &mut self,
        polygon: &PolygonLoop,
    ) -> Result<PolygonBoundary, Failure> {
        let mut vertices = DynamicArray::with_increment(256);
        let mut vertex_set = Map::default();
        let mut ignored = Map::default();
        let mut boundary = loop_edges();
        for (offset, link) in polygon.links.iter().enumerate() {
            let link_id = link.wrapping_abs();
            ignored.add(link_id);
            self.remove_boundary_triangle(*link, &mut boundary)?;
            if offset % 2 == 0 {
                self.append_boundary_vertices(*link, &mut vertices, &mut vertex_set)?;
            }
        }
        close_boundary_vertices(&mut vertices);
        Ok(PolygonBoundary {
            vertices,
            vertex_set,
            ignored,
            links: boundary,
        })
    }

    fn remove_boundary_triangle(
        &mut self,
        signed_link: i32,
        boundary: &mut LoopEdges,
    ) -> Result<(), Failure> {
        let link = signed_link.wrapping_abs();
        let forward = signed_link > 0;
        let pair = self.mesh_ref()?.elements_connected_to(link)?;
        for index in 1..=pair.extent() {
            let triangle_id = pair.index(index)?;
            if triangle_id < 0 {
                continue;
            }
            let triangle = self.get_triangle(triangle_id)?;
            if boundary_edge_slot(&triangle, link, forward).is_some() {
                self.delete_triangle(triangle_id, boundary)?;
                break;
            }
        }
        Ok(())
    }

    fn append_boundary_vertices(
        &self,
        signed_link: i32,
        vertices: &mut DynamicArray<i32>,
        vertex_set: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let edge = self.get_edge(signed_link.wrapping_abs())?;
        let endpoints = oriented_endpoints(&edge, signed_link > 0);
        for node in endpoints {
            vertex_set.add(node);
            vertices.append(node);
        }
        Ok(())
    }

    fn remove_polygon_cavity(
        &mut self,
        vertices: &DynamicArray<i32>,
        vertex_set: &Map<i32>,
        polygon: &PolygonLoop,
        survivors: &mut Map<i32>,
        boundary: &mut LoopEdges,
    ) -> Result<(), Failure> {
        let mut pending_victims = Vec::new();
        for index in 0..vertices.length() - 1 {
            self.kill_triangles_around_vertex(
                vertices[index as usize],
                vertices,
                vertex_set,
                polygon,
                survivors,
                boundary,
                &mut pending_victims,
            )?;
            self.process_victim_frames(
                vertices,
                vertex_set,
                polygon,
                survivors,
                boundary,
                &mut pending_victims,
            )?;
        }
        Ok(())
    }

    fn process_victim_frames(
        &mut self,
        vertices: &DynamicArray<i32>,
        vertex_set: &Map<i32>,
        polygon: &PolygonLoop,
        survivors: &mut Map<i32>,
        boundary: &mut LoopEdges,
        victims: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        #[allow(clippy::single_range_in_vec_init)]
        let mut frames = vec![0..victims.len()];
        while let Some(frame) = frames.last_mut() {
            if frame.start == frame.end {
                frames.pop();
                continue;
            }
            let victim = victims[frame.start];
            frame.start += 1;
            let first_new = victims.len();
            self.kill_triangles_around_vertex(
                victim, vertices, vertex_set, polygon, survivors, boundary, victims,
            )?;
            if first_new < victims.len() {
                frames.push(first_new..victims.len());
            }
        }
        Ok(())
    }

    // The native helper receives these same seven state values explicitly.
    #[allow(clippy::too_many_arguments)]
    fn kill_triangles_around_vertex(
        &mut self,
        zombie: i32,
        vertices: &DynamicArray<i32>,
        vertex_set: &Map<i32>,
        polygon: &PolygonLoop,
        survivors: &mut Map<i32>,
        boundary: &mut LoopEdges,
        victims: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let neighbors = self.mesh_ref()?.links_connected_to(zombie)?;
        for link_id in neighbors.iter().copied() {
            if survivors.contains(&link_id) {
                continue;
            }
            self.process_zombie_neighbor(
                zombie, link_id, vertices, vertex_set, polygon, survivors, boundary, victims,
            )?;
        }
        Ok(())
    }

    // Preserve native cleanup ordering and its explicit edge/vertex state.
    #[allow(clippy::too_many_arguments)]
    fn process_zombie_neighbor(
        &mut self,
        zombie: i32,
        link_id: i32,
        vertices: &DynamicArray<i32>,
        vertex_set: &Map<i32>,
        polygon: &PolygonLoop,
        survivors: &mut Map<i32>,
        boundary: &mut LoopEdges,
        victims: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let edge = self.get_edge(link_id)?;
        if self.keep_frontier_neighbor(&edge, polygon)? {
            survivors.add(link_id);
            return Ok(());
        }
        if edge.movability() != DegreeOfFreedom::Frontier {
            let other = other_node(&edge, zombie);
            if !vertex_set.contains(&other) && self.polygon_contains_vertex(other, vertices)? {
                victims.push(other);
            } else if !vertex_set.contains(&other) {
                return self.kill_intersecting_link(link_id, other, polygon, survivors, boundary);
            }
        }
        survivors.add(link_id);
        self.kill_link_triangles(link_id, boundary)
    }

    fn keep_frontier_neighbor(
        &self,
        edge: &super::BRepMeshEdge,
        polygon: &PolygonLoop,
    ) -> Result<bool, Failure> {
        if edge.movability() != DegreeOfFreedom::Frontier {
            return Ok(false);
        }
        self.link_is_outside_polygon(edge, polygon, false, true)
    }

    fn kill_intersecting_link(
        &mut self,
        link: i32,
        endpoint: i32,
        polygon: &PolygonLoop,
        survivors: &mut Map<i32>,
        boundary: &mut LoopEdges,
    ) -> Result<(), Failure> {
        if survivors.contains(&link)
            || self.link_is_outside_polygon(&self.get_edge(link)?, polygon, false, false)?
        {
            survivors.add(link);
            return Ok(());
        }
        survivors.add(link);
        self.kill_link_triangles(link, boundary)?;
        let neighbors = self.mesh_ref()?.links_connected_to(endpoint)?;
        for neighbor in neighbors.iter().copied() {
            let edge = self.get_edge(neighbor)?;
            self.kill_intersecting_link(
                neighbor,
                other_node(&edge, endpoint),
                polygon,
                survivors,
                boundary,
            )?;
        }
        Ok(())
    }

    fn link_is_outside_polygon(
        &self,
        edge: &super::BRepMeshEdge,
        polygon: &PolygonLoop,
        endpoint_touch: bool,
        point_on_edge: bool,
    ) -> Result<bool, Failure> {
        let mut bounds = CompactBox2::default();
        self.check_intersection(
            edge,
            &polygon.links,
            &polygon.boxes,
            endpoint_touch,
            point_on_edge,
            false,
            &mut bounds,
        )
    }

    fn kill_link_triangles(&mut self, link: i32, boundary: &mut LoopEdges) -> Result<(), Failure> {
        let pair = self.mesh_ref()?.elements_connected_to(link)?;
        for index in 1..=pair.extent() {
            let triangle = pair.index(index)?;
            if triangle >= 0 {
                self.delete_triangle(triangle, boundary)?;
            }
        }
        Ok(())
    }

    fn remove_unattached_loop_edges(
        &self,
        ignored: &Map<i32>,
        boundary: &LoopEdges,
    ) -> Result<(), Failure> {
        let links = boundary.items().map(|(link, _)| *link).collect::<Vec<_>>();
        for link in links {
            if !ignored.contains(&link) && self.mesh_ref()?.elements_connected_to(link)?.is_empty()
            {
                self.mesh_ref()?.remove_link(link, false)?;
            }
        }
        Ok(())
    }
}

struct PolygonBoundary {
    vertices: DynamicArray<i32>,
    vertex_set: Map<i32>,
    ignored: Map<i32>,
    links: LoopEdges,
}

fn copy_integer_set(source: &Map<i32>) -> Map<i32> {
    let mut copy = Map::default();
    for key in source.iter() {
        copy.add(*key);
    }
    copy
}

fn boundary_edge_slot(
    triangle: &super::BRepMeshTriangle,
    link: i32,
    forward: bool,
) -> Option<usize> {
    (0..3).find(|index| triangle.edges[*index] == link && triangle.orientations[*index] == forward)
}

fn oriented_endpoints(edge: &super::BRepMeshEdge, forward: bool) -> [i32; 2] {
    if forward {
        [edge.first_node(), edge.last_node()]
    } else {
        [edge.last_node(), edge.first_node()]
    }
}

fn close_boundary_vertices(vertices: &mut DynamicArray<i32>) {
    if vertices.length() > 0 && vertices[0] != vertices[(vertices.length() - 1) as usize] {
        vertices.append(vertices[0]);
    }
}

fn other_node(edge: &super::BRepMeshEdge, endpoint: i32) -> i32 {
    if edge.first_node() == endpoint {
        edge.last_node()
    } else {
        edge.first_node()
    }
}
