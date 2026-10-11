//! Whole-buffer overloads keep range derivation identical to pinned OCCT.
use super::{Select3DPrimArrayInit, Select3DSensitivePrimitiveArray};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::{Graphic3dBufferSource, Graphic3dIndexBufferSource};

impl Select3DSensitivePrimitiveArray {
    /// Initialize every indexed or sequential triangle; indices select the iteration range.
    /// Example: `sensitive.init_triangulation_all(&vertices, &indices, &Location::default(), true, 1)?`.
    pub fn init_triangulation_all(
        &self,
        vertices: &Handle<dyn Graphic3dBufferSource>,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        location: &Location,
        evaluate_bounds: bool,
        group_count: i32,
    ) -> Result<bool, Failure> {
        self.init_triangulation(&Select3DPrimArrayInit::whole(
            vertices,
            indices,
            location,
            evaluate_bounds,
            group_count,
        ))
    }

    /// Initialize every indexed or sequential point, preserving the supplied buffer aliases.
    /// Example: `sensitive.init_points_all(&vertices, &indices, &Location::default(), true, 1)?`.
    pub fn init_points_all(
        &self,
        vertices: &Handle<dyn Graphic3dBufferSource>,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        location: &Location,
        evaluate_bounds: bool,
        group_count: i32,
    ) -> Result<bool, Failure> {
        self.init_points(&Select3DPrimArrayInit::whole(
            vertices,
            indices,
            location,
            evaluate_bounds,
            group_count,
        ))
    }

    /// Initialize sequential points without an index buffer.
    /// Example: `sensitive.init_points_vertices(&vertices, &Location::default(), true, 1)?`.
    pub fn init_points_vertices(
        &self,
        vertices: &Handle<dyn Graphic3dBufferSource>,
        location: &Location,
        evaluate_bounds: bool,
        group_count: i32,
    ) -> Result<bool, Failure> {
        self.init_points_all(
            vertices,
            &Handle::default(),
            location,
            evaluate_bounds,
            group_count,
        )
    }
}
