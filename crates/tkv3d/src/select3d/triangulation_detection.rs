//! Output adapters retain native assignment order even when a later node access fails.
use super::sensitive_triangulation::Select3DSensitiveTriangulation;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::Triangle;

impl Select3DSensitiveTriangulation {
    /// Writes a detected triangle, leaving the output intact on a miss.
    /// Example: `sens.last_detected_triangle_into(&mut triangle)?`.
    pub fn last_detected_triangle_into(&self, triangle: &mut Triangle) -> Result<bool, Failure> {
        let index = self.checked_detection_index()?;
        if index == -1 {
            return Ok(false);
        }
        *triangle = self.detection_mesh()?.triangle(index)?;
        Ok(true)
    }
    /// Writes the triangle first, then transformed nodes in native order, preserving earlier writes on failure.
    /// Example: `sens.last_detected_triangle_nodes_into(&mut triangle, &mut nodes)?`.
    pub fn last_detected_triangle_nodes_into(
        &self,
        triangle: &mut Triangle,
        nodes: &mut [Point3; 3],
    ) -> Result<bool, Failure> {
        if !self.last_detected_triangle_into(triangle)? {
            return Ok(false);
        }
        let mesh = self.detection_mesh()?;
        let transform = self.init_location.transformation();
        let (first, middle, last) = triangle.get();
        // OCCT 3d097a0 writes each output before accessing the next node; checked failure is observable.
        for (slot, index) in [first, middle, last].into_iter().enumerate() {
            nodes[slot] = mesh.node(index)?.transformed(transform);
        }
        Ok(true)
    }
    fn detection_mesh(&self) -> Result<&rustycad_tkmath::poly::Triangulation, Failure> {
        self.triangul.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "null mesh: expected detected triangulation",
            )
        })
    }
}
