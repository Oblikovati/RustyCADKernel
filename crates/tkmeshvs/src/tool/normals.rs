//! OCCT 3d097a0 MeshVS_Tool.cxx uses a fixed first edge, not a normalized sum.
use super::MeshVSTool;
use rustycad_tkernel::{ncollection::Array1, precision::confusion, standard::Failure};
use rustycad_tkmath::gp::Vector3;
struct PolygonNodes<'a, 'b> {
    nodes: &'a Array1<'b, f64>,
    first: i32,
    count: i32,
}
impl MeshVSTool {
    /// Keep output unchanged for too few points or non-planarity; `Self::get_normal(&nodes, &mut normal)?`.
    pub fn get_normal(nodes: &Array1<'_, f64>, normal: &mut Vector3) -> Result<bool, Failure> {
        let polygon = PolygonNodes::capture(nodes)?;
        if polygon.count < 3 {
            return Ok(false);
        }
        let first_edge = polygon.first_edge()?;
        let mut selected = [0.; 3];
        for index in 2..polygon.count {
            if !same_normal(&mut selected, polygon.triangle(index, first_edge)?) {
                return Ok(false);
            }
        }
        normal.set_coordinates(selected[0], selected[1], selected[2]);
        Ok(true)
    }
    /// Write the unnormalized average on non-planarity; `Self::get_average_normal(&nodes, &mut normal)?`.
    pub fn get_average_normal(
        nodes: &Array1<'_, f64>,
        normal: &mut Vector3,
    ) -> Result<bool, Failure> {
        let polygon = PolygonNodes::capture(nodes)?;
        if polygon.count < 3 {
            return Ok(false);
        }
        let (selected, planar) = polygon.average()?;
        normal.set_coordinates(selected[0], selected[1], selected[2]);
        Ok(planar)
    }
}
impl<'a, 'b> PolygonNodes<'a, 'b> {
    fn capture(nodes: &'a Array1<'b, f64>) -> Result<Self, Failure> {
        let mut first = nodes.lower();
        let mut count = (nodes.upper() - first + 1) / 3;
        if first == 0 {
            first = 1;
            count = *nodes
                .value(0)
                .map_err(|cause| cause.into_failure("NCollection_Array1::Value"))?
                as i32;
        }
        Ok(Self {
            nodes,
            first,
            count,
        })
    }
    fn value(&self, index: i32) -> Result<f64, Failure> {
        self.nodes
            .value(index)
            .copied()
            .map_err(|cause| cause.into_failure("NCollection_Array1::Value"))
    }
    fn offset(&self, index: i32) -> Result<[f64; 3], Failure> {
        let mut result = [0.; 3];
        for component in 0..3 {
            result[component as usize] = self.value(self.first + 3 * index + component)?
                - self.value(self.first + component)?;
        }
        Ok(result)
    }
    fn first_edge(&self) -> Result<[f64; 3], Failure> {
        self.offset(1)
    }
    fn triangle(&self, index: i32, first: [f64; 3]) -> Result<[f64; 3], Failure> {
        let current = self.offset(index)?;
        let mut cross = [
            first[1] * current[2] - first[2] * current[1],
            first[2] * current[0] - first[0] * current[2],
            first[0] * current[1] - first[1] * current[0],
        ];
        if cross.iter().any(|component| component.abs() > confusion()) {
            let length = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            for component in &mut cross {
                *component /= length;
            }
        }
        Ok(cross)
    }
    fn average(&self) -> Result<([f64; 3], bool), Failure> {
        let first = self.first_edge()?;
        let mut triangles = Vec::with_capacity((self.count - 2) as usize);
        let mut selected = [0.; 3];
        let mut planar = true;
        for index in 2..self.count {
            let current = self.triangle(index, first)?;
            triangles.push(current);
            if !same_normal(&mut selected, current) {
                planar = false;
            }
        }
        if !planar {
            selected = average_triangles(&triangles);
        }
        Ok((selected, planar))
    }
}
fn same_normal(selected: &mut [f64; 3], current: [f64; 3]) -> bool {
    if selected
        .iter()
        .all(|component| component.abs() <= confusion())
    {
        *selected = current;
    }
    !selected
        .iter()
        .zip(current)
        .any(|(left, right)| (*left - right).abs() > confusion())
}
fn average_triangles(triangles: &[[f64; 3]]) -> [f64; 3] {
    let mut average = [0.; 3];
    for component in 0..3 {
        for triangle in triangles {
            average[component] += triangle[component];
        }
        average[component] /= triangles.len() as f64;
    }
    average
}
