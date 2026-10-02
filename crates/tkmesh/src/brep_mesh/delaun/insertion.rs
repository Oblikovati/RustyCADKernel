//! Temporary enclosing-triangle construction used before vertex insertion.
use super::triangle_build::loop_edges;
use super::{BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, DegreeOfFreedom};
use rustycad_tkernel::{ncollection::DataMap, standard::Failure};
use rustycad_tkmath::gp::Xy;

impl BRepMeshDelaun {
    /// Build OCCT's enclosing triangle for a finite box; `delaun.build_super_mesh([0.,0.,2.,2.])?`.
    #[doc(hidden)]
    pub fn build_super_mesh(&mut self, bounds: [f64; 4]) -> Result<([i32; 3], [i32; 3]), Failure> {
        let points = super_vertices(bounds);
        let nodes = self.add_super_vertices(points)?;
        let edges = self.add_super_edges(nodes)?;
        Ok((nodes, edges))
    }

    /// Add the first interior point to the enclosing triangle; `delaun.build_super_fan([0.,0.,2.,2.], Xy::new(1.,1.))?`.
    #[doc(hidden)]
    pub fn build_super_fan(
        &mut self,
        bounds: [f64; 4],
        point: Xy,
    ) -> Result<(i32, [i32; 3]), Failure> {
        let vertex = self
            .mesh_ref()?
            .add_node(&BRepMeshVertex::new(point, 0, DegreeOfFreedom::Free), true)?;
        let (_, edges) = self.build_super_mesh(bounds)?;
        let mut boundary = loop_edges();
        bind_super_edges(edges, &mut boundary);
        self.add_fan_triangles(vertex, &boundary)?;
        Ok((vertex, edges))
    }

    fn add_fan_triangles(
        &mut self,
        vertex: i32,
        boundary: &DataMap<i32, bool>,
    ) -> Result<(), Failure> {
        for (edge, forward) in boundary.items() {
            self.add_fan_triangle(vertex, *edge, *forward)?;
        }
        Ok(())
    }

    fn add_fan_triangle(&mut self, vertex: i32, edge: i32, forward: bool) -> Result<(), Failure> {
        let nodes = self.fan_nodes(edge, forward)?;
        let point = self.get_vertex(vertex)?;
        let first = self.get_vertex(nodes[0])?;
        let last = self.get_vertex(nodes[1])?;
        let direction = *last.coord() - *first.coord();
        let length = direction.modulus();
        if length < rustycad_tkernel::precision::p_confusion() {
            return Ok(());
        }
        let unit = direction / length;
        let sides = [
            (*first.coord() - *point.coord()).crossed(unit),
            unit.crossed(*point.coord() - *last.coord()),
        ];
        self.add_visible_fan(vertex, nodes, edge, forward, sides)
    }

    fn fan_nodes(&self, edge: i32, forward: bool) -> Result<[i32; 2], Failure> {
        let link = self.get_edge(edge)?;
        Ok(if forward {
            [link.first_node(), link.last_node()]
        } else {
            [link.last_node(), link.first_node()]
        })
    }

    fn add_visible_fan(
        &mut self,
        vertex: i32,
        nodes: [i32; 2],
        edge: i32,
        forward: bool,
        sides: [f64; 2],
    ) -> Result<(), Failure> {
        if sides
            .iter()
            .any(|side| side.abs() < rustycad_tkernel::precision::p_confusion())
        {
            return Ok(());
        }
        if sides.iter().any(|side| *side < 0.) {
            return Err(Failure::with_type(
                "Standard_DomainError",
                &format!(
                    "Delaun initial fan sides={sides:?}; expected a point inside the supertriangle"
                ),
                "",
            ));
        }
        let first = self.add_free_link(vertex, nodes[0])?;
        let last = self.add_free_link(nodes[1], vertex)?;
        let oriented_edge = if forward { edge } else { -edge };
        self.add_signed_triangle([first, oriented_edge, last], [nodes[0], vertex, nodes[1]])
    }

    fn add_super_vertices(&mut self, points: [Xy; 3]) -> Result<[i32; 3], Failure> {
        let mut nodes = [0; 3];
        for (slot, point) in nodes.iter_mut().zip(points) {
            *slot = self
                .mesh_ref()?
                .add_node(&BRepMeshVertex::new(point, 0, DegreeOfFreedom::Free), true)?;
            self.auxiliary.append(*slot);
        }
        Ok(nodes)
    }

    fn add_super_edges(&mut self, nodes: [i32; 3]) -> Result<[i32; 3], Failure> {
        for index in 0..3 {
            let next = (index + 1) % 3;
            self.super_edges[index] = self.mesh_ref()?.add_link(&BRepMeshEdge::new(
                nodes[index],
                nodes[next],
                DegreeOfFreedom::Free,
            ))?;
        }
        Ok(self.super_edges)
    }
}

fn bind_super_edges(edges: [i32; 3], boundary: &mut DataMap<i32, bool>) {
    for edge in edges {
        boundary.bind(edge.abs(), edge > 0);
    }
}

fn super_vertices([min_x, min_y, max_x, max_y]: [f64; 4]) -> [Xy; 3] {
    let delta_x = max_x - min_x;
    let delta_y = max_y - min_y;
    let delta_min = delta_x.min(delta_y);
    let delta_max = delta_x.max(delta_y);
    let delta = delta_x + delta_y;
    [
        Xy::new((min_x + max_x) / 2., max_y + delta_max),
        Xy::new(min_x - delta, min_y - delta_min),
        Xy::new(max_x + delta, min_y - delta_min),
    ]
}
