use super::BRepMeshDelaun;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    bnd::BoundingBox2,
    gp::{Point2, Xy},
};
impl BRepMeshDelaun {
    /// Fill circumcircles from all existing elements; `triangulator.init_circles_tool(8,8)?`.
    pub fn init_circles_tool(&mut self, u: i32, v: i32) -> Result<(), Failure> {
        let bounds = self.node_bounds()?;
        self.initialize_circle_bounds(&bounds, u, v)?;
        for index in self.mesh_ref()?.elements_of_domain().iter() {
            let points = self.triangle_points(&self.get_triangle(*index)?)?;
            self.circles.bind_points(*index, &points)?;
        }
        Ok(())
    }
    fn node_bounds(&self) -> Result<BoundingBox2, Failure> {
        let mut bounds = BoundingBox2::default();
        for index in 1..=self.mesh_ref()?.nb_nodes() {
            bounds.add_point(Point2::from(*self.get_vertex(index)?.coord()));
        }
        bounds.enlarge(rustycad_tkernel::precision::p_confusion());
        Ok(bounds)
    }
    pub(super) fn triangle_points(
        &self,
        triangle: &super::BRepMeshTriangle,
    ) -> Result<[Xy; 3], Failure> {
        let nodes = self.mesh_ref()?.element_nodes(triangle)?;
        Ok([
            *self.get_vertex(nodes[0])?.coord(),
            *self.get_vertex(nodes[1])?.coord(),
            *self.get_vertex(nodes[2])?.coord(),
        ])
    }
    pub(super) fn initialize_circle_bounds(
        &mut self,
        bounds: &BoundingBox2,
        u: i32,
        v: i32,
    ) -> Result<(), Failure> {
        let [min_x, min_y, max_x, max_y] = bounds.get()?;
        // Pinned OCCT tests >100 before >1000, making the later scaler=7 branch unreachable.
        let scaler = if self.mesh_ref()?.nb_nodes() > 100 {
            5
        } else {
            2
        };
        self.circles
            .set_min_max_size(Xy::new(min_x, min_y), Xy::new(max_x, max_y));
        self.circles.set_cell_sizes(
            (max_x - min_x) / f64::from(u.max(scaler)),
            (max_y - min_y) / f64::from(v.max(scaler)),
        )?;
        self.initialized_circles = true;
        Ok(())
    }
}
