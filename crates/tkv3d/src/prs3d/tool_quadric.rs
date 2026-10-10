//! `Prs3d_ToolQuadric`: the triangulation of a parametric quadric over a grid of slices (U)
//! and stacks (V). The abstract class is a trait; the concrete tools (cylinder, disk,
//! sphere, torus, sector) implement `vertex` and `normal`.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point3, Transform3};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL, Graphic3dArrayOfTriangles,
};

/// `Prs3d_ToolQuadric::TrianglesNb(slices, stacks)`; `prs3d_quadric_triangles_nb(12, 1) == 24`.
pub fn prs3d_quadric_triangles_nb(slices_nb: i32, stacks_nb: i32) -> i32 {
    slices_nb * stacks_nb * 2
}

/// `Prs3d_ToolQuadric::VerticesNb(slices, stacks, indexed)`: the grid nodes when indexed, else
/// three per triangle; `prs3d_quadric_vertices_nb(12, 1, true) == 26`.
pub fn prs3d_quadric_vertices_nb(slices_nb: i32, stacks_nb: i32, is_indexed: bool) -> i32 {
    if is_indexed {
        (slices_nb + 1) * (stacks_nb + 1)
    } else {
        prs3d_quadric_triangles_nb(slices_nb, stacks_nb) * 3
    }
}

/// A parametric quadric tool; `vertex` and `normal` are the native pure virtuals, the provided
/// members are the native non-virtual ones.
pub trait Prs3dToolQuadric {
    /// `mySlicesNb`: the slices along U.
    fn slices_nb(&self) -> i32;
    /// `myStacksNb`: the stacks along V.
    fn stacks_nb(&self) -> i32;
    /// `Vertex(U, V)` in the local frame; `tool.vertex(0.5, 0.5)`.
    fn vertex(&self, u: f64, v: f64) -> Point3;
    /// `Normal(U, V)`; a null normal raises `Standard_ConstructionError` (the `gp_Dir` constructor).
    /// Example: `tool.normal(0.5, 0.5)?`.
    fn normal(&self, u: f64, v: f64) -> Result<Direction3, Failure>;
    /// `TrianglesNb()`; `tool.triangles_nb()`.
    fn triangles_nb(&self) -> i32 {
        prs3d_quadric_triangles_nb(self.slices_nb(), self.stacks_nb())
    }
    /// `VerticesNb(indexed)`; `tool.vertices_nb(true)`.
    fn vertices_nb(&self, is_indexed: bool) -> i32 {
        prs3d_quadric_vertices_nb(self.slices_nb(), self.stacks_nb(), is_indexed)
    }
    /// `FillArray(array, trsf)`: creates an indexed array with normals when null, then adds the
    /// grid vertices (with triangle edges when the array has edges, else six vertices per
    /// cell). Example: `tool.fill_array(&mut array, &trsf)?`.
    fn fill_array(
        &self,
        array: &mut Handle<Graphic3dArrayOfTriangles>,
        trsf: &Transform3,
    ) -> Result<(), Failure> {
        if array.is_null() {
            let created = Graphic3dArrayOfTriangles::new(
                self.vertices_nb(true),
                self.triangles_nb() * 3,
                GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
            )?;
            *array = Handle::new(created);
        }
        let Some(triangles) = array.get() else {
            return Ok(());
        };
        if triangles.edge_number_allocated() > 0 {
            fill_indexed(self, triangles, trsf)
        } else {
            fill_unindexed(self, triangles, trsf)
        }
    }
    /// `CreateTriangulation(trsf)`: a new array filled by `FillArray`; `tool.create_triangulation(&trsf)?`.
    fn create_triangulation(
        &self,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        let mut array = Handle::default();
        self.fill_array(&mut array, trsf)?;
        Ok(array)
    }
    /// `CreatePolyTriangulation(trsf)`: the grid nodes and triangles as a `Poly_Triangulation`.
    /// Example: `tool.create_poly_triangulation(&trsf)?`.
    fn create_poly_triangulation(
        &self,
        trsf: &Transform3,
    ) -> Result<Handle<Triangulation>, Failure> {
        let triangulation =
            Triangulation::with_sizes(self.vertices_nb(true), self.triangles_nb(), false, false)?;
        let (slices, stacks) = (self.slices_nb(), self.stacks_nb());
        let step_u = 1.0 / f64::from(slices);
        // The native V step is a float; its products are float too.
        let step_v = 1.0f32 / stacks as f32;
        let mut index = 0;
        for u in 0..=slices {
            let param_u = f64::from(u) * step_u;
            for v in 0..=stacks {
                let param_v = v as f32 * step_v;
                let vert_id = u * (stacks + 1) + (v + 1);
                triangulation.set_node(
                    vert_id,
                    self.vertex(param_u, f64::from(param_v)).transformed(*trsf),
                )?;
                if u != 0 && v != 0 {
                    index += 1;
                    triangulation.set_triangle(
                        index,
                        Triangle::new(vert_id, vert_id - stacks - 2, vert_id - 1),
                    )?;
                    index += 1;
                    triangulation.set_triangle(
                        index,
                        Triangle::new(vert_id - stacks - 2, vert_id, vert_id - stacks - 1),
                    )?;
                }
            }
        }
        Ok(Handle::new(triangulation))
    }
    /// The deprecated `FillArray(array, triangulation, trsf)`: both representations.
    /// Example: `tool.fill_array_and_triangulation(&mut array, &mut triangulation, &trsf)?`.
    fn fill_array_and_triangulation(
        &self,
        array: &mut Handle<Graphic3dArrayOfTriangles>,
        triangulation: &mut Handle<Triangulation>,
        trsf: &Transform3,
    ) -> Result<(), Failure> {
        *array = self.create_triangulation(trsf)?;
        *triangulation = self.create_poly_triangulation(trsf)?;
        Ok(())
    }
}

/// The native float parameter steps of `FillArray` (`1.0f / n`, widened to double).
fn float_steps(slices: i32, stacks: i32) -> (f64, f64) {
    (
        f64::from(1.0f32 / slices as f32),
        f64::from(1.0f32 / stacks as f32),
    )
}

/// Adds the transformed vertex and normal at `(u, v)`.
fn add_node<T: Prs3dToolQuadric + ?Sized>(
    tool: &T,
    array: &Graphic3dArrayOfTriangles,
    trsf: &Transform3,
    u: f64,
    v: f64,
) -> Result<i32, Failure> {
    let vertex = tool.vertex(u, v).transformed(*trsf);
    let normal = tool.normal(u, v)?.transformed(*trsf);
    array.add_vertex_pnt_normal(&vertex, &normal)
}

/// The indexed branch of `FillArray`: one vertex per grid node, two triangles per cell.
fn fill_indexed<T: Prs3dToolQuadric + ?Sized>(
    tool: &T,
    array: &Graphic3dArrayOfTriangles,
    trsf: &Transform3,
) -> Result<(), Failure> {
    let (slices, stacks) = (tool.slices_nb(), tool.stacks_nb());
    let (step_u, step_v) = float_steps(slices, stacks);
    for u in 0..=slices {
        let param_u = f64::from(u) * step_u;
        for v in 0..=stacks {
            add_node(tool, array, trsf, param_u, f64::from(v) * step_v)?;
            if u != 0 && v != 0 {
                let id = array.vertex_number();
                array.add_triangle_edges(id, id - stacks - 2, id - 1)?;
                array.add_triangle_edges(id - stacks - 2, id, id - stacks - 1)?;
            }
        }
    }
    Ok(())
}

/// The non-indexed branch of `FillArray`: six vertices per cell.
fn fill_unindexed<T: Prs3dToolQuadric + ?Sized>(
    tool: &T,
    array: &Graphic3dArrayOfTriangles,
    trsf: &Transform3,
) -> Result<(), Failure> {
    let (slices, stacks) = (tool.slices_nb(), tool.stacks_nb());
    let (step_u, step_v) = float_steps(slices, stacks);
    for u in 0..slices {
        let param_u = f64::from(u) * step_u;
        for v in 0..stacks {
            let param_v = f64::from(v) * step_v;
            let corners = [
                (param_u, param_v),
                (param_u + step_u, param_v),
                (param_u + step_u, param_v + step_v),
                (param_u + step_u, param_v + step_v),
                (param_u, param_v + step_v),
                (param_u, param_v),
            ];
            for (cu, cv) in corners {
                add_node(tool, array, trsf, cu, cv)?;
            }
        }
    }
    Ok(())
}
