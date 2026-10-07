//! The edge (index) and bound members of `Graphic3d_ArrayOfPrimitives`.
use super::{
    Graphic3dArrayOfPrimitives, Graphic3dBufferMembers, Graphic3dIndexBufferMembers,
    Graphic3dTypeOfPrimitiveArray,
};
use rustycad_tkernel::ncollection::{Vec3, Vec4};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::{out_of_range, type_mismatch};

/// `Standard_TypeMismatch` unless the array has the expected type.
fn require_type(
    actual: Graphic3dTypeOfPrimitiveArray,
    expected: Graphic3dTypeOfPrimitiveArray,
    message: &str,
) -> Result<(), Failure> {
    if actual != expected {
        return Err(type_mismatch(&format!(
            "{message}; got {actual:?}, expected {expected:?}"
        )));
    }
    Ok(())
}

/// The edges (1-based vertex indices stored 0-based).
impl Graphic3dArrayOfPrimitives {
    /// The index count, -1 without index buffer; `array.edge_number()`.
    pub fn edge_number(&self) -> i32 {
        self.indices
            .get()
            .map_or(-1, |indices| indices.nb_elements())
    }
    /// The allocated index count; `array.edge_number_allocated()`.
    pub fn edge_number_allocated(&self) -> i32 {
        self.indices
            .get()
            .map_or(0, |indices| indices.nb_max_elements())
    }
    /// The vertex of edge `rank` (1-based); `Standard_OutOfRange` outside. `array.edge(1)?`.
    pub fn edge(&self, rank: i32) -> Result<i32, Failure> {
        let indices = self
            .indices
            .get()
            .filter(|indices| rank >= 1 && rank <= indices.nb_elements());
        let index = indices.and_then(|indices| indices.index(rank - 1));
        index.map(|value| value + 1).ok_or_else(|| {
            out_of_range(&format!(
                "BAD EDGE index; got {rank}, expected [1, {}]",
                self.edge_number()
            ))
        })
    }
    /// `AddEdge`: appends a vertex index; too many edges or a vertex outside `1..=VertexNumber`
    /// raise `Standard_OutOfRange`. Example: `array.add_edge(1)?`.
    pub fn add_edge(&self, vertex: i32) -> Result<i32, Failure> {
        let indices = self
            .indices
            .get()
            .filter(|indices| indices.nb_elements() < indices.nb_max_elements());
        let Some(indices) = indices else {
            return Err(out_of_range(&format!(
                "TOO many EDGE; got {} of {}",
                self.edge_number(),
                self.edge_number_allocated()
            )));
        };
        if vertex < 1 || vertex > self.vertex_number() {
            return Err(out_of_range(&format!(
                "BAD VERTEX index; got {vertex}, expected [1, {}]",
                self.vertex_number()
            )));
        }
        let count = indices.nb_elements();
        indices.set_index(count, vertex - 1);
        indices.set_nb_elements(count + 1);
        Ok(count + 1)
    }
    /// Two edges; `array.add_edges2(1, 2)?`.
    pub fn add_edges2(&self, v1: i32, v2: i32) -> Result<i32, Failure> {
        self.add_edge(v1)?;
        self.add_edge(v2)
    }
    /// Three edges; `array.add_edges3(1, 2, 3)?`.
    pub fn add_edges3(&self, v1: i32, v2: i32, v3: i32) -> Result<i32, Failure> {
        self.add_edges2(v1, v2)?;
        self.add_edge(v3)
    }
    /// Four edges; `array.add_edges4(1, 2, 3, 4)?`.
    pub fn add_edges4(&self, v1: i32, v2: i32, v3: i32, v4: i32) -> Result<i32, Failure> {
        self.add_edges3(v1, v2, v3)?;
        self.add_edge(v4)
    }
    /// A segment of a segments array (`Standard_TypeMismatch` otherwise); `array.add_segment_edges(1, 2)?`.
    pub fn add_segment_edges(&self, v1: i32, v2: i32) -> Result<i32, Failure> {
        require_type(
            self.kind,
            Graphic3dTypeOfPrimitiveArray::Segments,
            "Not array of segments",
        )?;
        self.add_edges2(v1, v2)
    }
    /// A triangle of a triangles array; `array.add_triangle_edges(1, 2, 3)?`.
    pub fn add_triangle_edges(&self, v1: i32, v2: i32, v3: i32) -> Result<i32, Failure> {
        require_type(
            self.kind,
            Graphic3dTypeOfPrimitiveArray::Triangles,
            "Not array of triangles",
        )?;
        self.add_edges3(v1, v2, v3)
    }
    /// `AddTriangleEdges(NCollection_Vec3<int>)`; `array.add_triangle_edges_vec3(Vec3::new(1, 2, 3))?`.
    pub fn add_triangle_edges_vec3(&self, indexes: Vec3<i32>) -> Result<i32, Failure> {
        self.add_triangle_edges(indexes.x, indexes.y, indexes.z)
    }
    /// `AddTriangleEdges(NCollection_Vec4<int>)`: the first three; `array.add_triangle_edges_vec4(v)?`.
    pub fn add_triangle_edges_vec4(&self, indexes: Vec4<i32>) -> Result<i32, Failure> {
        self.add_triangle_edges(indexes.x, indexes.y, indexes.z)
    }
    /// A quad of a quadrangles array; `array.add_quad_edges(1, 2, 3, 4)?`.
    pub fn add_quad_edges(&self, v1: i32, v2: i32, v3: i32, v4: i32) -> Result<i32, Failure> {
        require_type(
            self.kind,
            Graphic3dTypeOfPrimitiveArray::Quadrangles,
            "Not array of quads",
        )?;
        self.add_edges4(v1, v2, v3, v4)
    }
    /// A quad as the triangles (3, 1, 2) and (1, 3, 4); `array.add_quad_triangle_edges(1, 2, 3, 4)?`.
    pub fn add_quad_triangle_edges(
        &self,
        v1: i32,
        v2: i32,
        v3: i32,
        v4: i32,
    ) -> Result<i32, Failure> {
        self.add_triangle_edges(v3, v1, v2)?;
        self.add_triangle_edges(v1, v3, v4)
    }
    /// `AddQuadTriangleEdges(NCollection_Vec4<int>)`; `array.add_quad_triangle_edges_vec4(v)?`.
    pub fn add_quad_triangle_edges_vec4(&self, indexes: Vec4<i32>) -> Result<i32, Failure> {
        self.add_quad_triangle_edges(indexes.x, indexes.y, indexes.z, indexes.w)
    }
}

/// The edge generators of strips, fans and polylines.
impl Graphic3dArrayOfPrimitives {
    /// The triangles of a strip `lower..=upper`, alternating winding; `array.add_triangle_strip_edges(1, 5)?`.
    pub fn add_triangle_strip_edges(&self, lower: i32, upper: i32) -> Result<(), Failure> {
        require_type(
            self.kind,
            Graphic3dTypeOfPrimitiveArray::Triangles,
            "Not array of triangles",
        )?;
        let mut is_odd = true;
        for node in lower + 2..=upper {
            if is_odd {
                self.add_triangle_edges(node - 2, node - 1, node)?;
            } else {
                self.add_triangle_edges(node - 1, node - 2, node)?;
            }
            is_odd = !is_odd;
        }
        Ok(())
    }
    /// The triangles of a fan around `lower`, closed on request; `array.add_triangle_fan_edges(1, 5, true)?`.
    pub fn add_triangle_fan_edges(
        &self,
        lower: i32,
        upper: i32,
        to_close: bool,
    ) -> Result<(), Failure> {
        require_type(
            self.kind,
            Graphic3dTypeOfPrimitiveArray::Triangles,
            "Not array of triangles",
        )?;
        for node in lower + 1..=upper {
            self.add_triangle_edges(lower, node - 1, node)?;
        }
        if to_close {
            self.add_triangle_edges(lower, upper, lower + 1)?;
        }
        Ok(())
    }
    /// The segments of a polyline, closed on request; `array.add_polyline_edges(1, 4, false)?`.
    pub fn add_polyline_edges(
        &self,
        lower: i32,
        upper: i32,
        to_close: bool,
    ) -> Result<(), Failure> {
        require_type(
            self.kind,
            Graphic3dTypeOfPrimitiveArray::Segments,
            "Not array of segments",
        )?;
        for node in lower..upper {
            self.add_segment_edges(node, node + 1)?;
        }
        if to_close {
            self.add_segment_edges(upper, lower)?;
        }
        Ok(())
    }
}

/// The bounds.
impl Graphic3dArrayOfPrimitives {
    /// Whether the bounds have colors; `array.has_bound_colors()`.
    pub fn has_bound_colors(&self) -> bool {
        self.bounds
            .get()
            .is_some_and(|bounds| bounds.state().has_colors)
    }
    /// The bound count, -1 without bound buffer; `array.bound_number()`.
    pub fn bound_number(&self) -> i32 {
        self.bounds
            .get()
            .map_or(-1, |bounds| bounds.state().nb_bounds)
    }
    /// The allocated bound count; `array.bound_number_allocated()`.
    pub fn bound_number_allocated(&self) -> i32 {
        self.bounds
            .get()
            .map_or(0, |bounds| bounds.state().nb_max_bounds)
    }
    /// The count of bound `rank`; `Standard_OutOfRange` outside. `array.bound(1)?`.
    pub fn bound(&self, rank: i32) -> Result<i32, Failure> {
        let bounds = self
            .bounds
            .get()
            .filter(|_| rank >= 1 && rank <= self.bound_number());
        bounds
            .and_then(|bounds| bounds.bound(rank - 1))
            .ok_or_else(|| {
                out_of_range(&format!(
                    "BAD BOUND index; got {rank}, expected [1, {}]",
                    self.bound_number()
                ))
            })
    }
    /// `BoundColor(rank, r, g, b)`; `Standard_OutOfRange` without colors or outside. `array.bound_color_rgb(1)?`.
    pub fn bound_color_rgb(&self, rank: i32) -> Result<[f64; 3], Failure> {
        let bounds = self
            .bounds
            .get()
            .filter(|_| rank >= 1 && rank <= self.bound_number());
        let color = bounds.and_then(|bounds| bounds.color(rank - 1));
        let color = color.ok_or_else(|| {
            out_of_range(&format!(
                "BAD BOUND index; got {rank}, expected a colored bound in [1, {}]",
                self.bound_number()
            ))
        })?;
        Ok([color.x, color.y, color.z].map(f64::from))
    }
    /// `BoundColor(rank)`; `array.bound_color(1)?`.
    pub fn bound_color(&self, rank: i32) -> Result<Color, Failure> {
        let [r, g, b] = self.bound_color_rgb(rank)?;
        Color::new(r, g, b, ColorSpace::Rgb)
    }
    /// `AddBound`: appends a count; too many raise `Standard_OutOfRange`. `array.add_bound(3)?`.
    pub fn add_bound(&self, edge_number: i32) -> Result<i32, Failure> {
        let bounds = self
            .bounds
            .get()
            .filter(|_| self.bound_number() < self.bound_number_allocated());
        let Some(bounds) = bounds else {
            return Err(out_of_range(&format!(
                "TOO many BOUND; got {} of {}",
                self.bound_number(),
                self.bound_number_allocated()
            )));
        };
        let count = self.bound_number();
        bounds.set_bound(count, edge_number);
        bounds.set_nb_bounds(count + 1);
        Ok(count + 1)
    }
    /// `AddBound(count, r, g, b)`: a colored bound; `array.add_bound_rgb(3, 1.0, 0.0, 0.0)?`.
    pub fn add_bound_rgb(&self, edge_number: i32, r: f64, g: f64, b: f64) -> Result<i32, Failure> {
        let count = self.add_bound(edge_number)?;
        self.set_bound_color_rgb(count, r, g, b)?;
        Ok(count)
    }
    /// `AddBound(count, Quantity_Color)`; `array.add_bound_color(3, red)?`.
    pub fn add_bound_color(&self, edge_number: i32, color: Color) -> Result<i32, Failure> {
        self.add_bound_rgb(edge_number, color.red(), color.green(), color.blue())
    }
    /// `SetBoundColor(index, r, g, b)`: alpha 1, raising the bound count; nothing without
    /// bounds; `Standard_OutOfRange` without colors or outside. `array.set_bound_color_rgb(1, 1.0, 0.0, 0.0)?`.
    pub fn set_bound_color_rgb(&self, index: i32, r: f64, g: f64, b: f64) -> Result<(), Failure> {
        let Some(bounds) = self.bounds.get() else {
            return Ok(());
        };
        if index < 1
            || index > self.bound_number_allocated()
            || !bounds.set_color(index - 1, Vec4::new(r as f32, g as f32, b as f32, 1.0))
        {
            return Err(out_of_range(&format!(
                "BAD BOUND index; got {index}, expected a colored bound in [1, {}]",
                self.bound_number_allocated()
            )));
        }
        bounds.set_nb_bounds(index.max(self.bound_number()));
        Ok(())
    }
    /// `SetBoundColor(index, Quantity_Color)`; `array.set_bound_color(1, red)?`.
    pub fn set_bound_color(&self, index: i32, color: Color) -> Result<(), Failure> {
        self.set_bound_color_rgb(index, color.red(), color.green(), color.blue())
    }
}
