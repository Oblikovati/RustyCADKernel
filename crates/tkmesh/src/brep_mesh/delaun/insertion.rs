//! Temporary enclosing-triangle construction used before vertex insertion.
use super::{BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, DegreeOfFreedom};
use rustycad_tkernel::standard::Failure;
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
