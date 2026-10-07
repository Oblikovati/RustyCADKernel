//! The vertex members of `Graphic3d_ArrayOfPrimitives`: positions, normals, texels and colors
//! written into and read from the interleaved vertex buffer.
use super::{Graphic3dArrayOfPrimitives, Graphic3dAttributeSlot, Graphic3dBufferMembers};
use crate::cast::int_from_f64;
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::out_of_range;
use rustycad_tkernel::standard::real::real_to_short_real;
use rustycad_tkmath::gp::{Direction3, Point2, Point3};

/// `Standard_OutOfRange` "BAD VERTEX index" with the index and the limit.
fn bad_vertex(index: i32, limit: i32) -> Failure {
    out_of_range(&format!(
        "BAD VERTEX index; got {index}, expected [1, {limit}]"
    ))
}

/// The byte offset of vertex `index` (1-based) in a slot.
fn slot_offset(slot: &Graphic3dAttributeSlot, index: i32) -> Option<usize> {
    slot.offset
        .map(|offset| offset + slot.stride.wrapping_mul((index as usize).wrapping_sub(1)))
}

/// Checks and accessors shared by the vertex members.
impl Graphic3dArrayOfPrimitives {
    /// `Standard_OutOfRange` unless `1 <= index <= NbMaxElements` (the setters' check).
    fn check_allocated(&self, index: i32) -> Result<(), Failure> {
        let limit = self.vertex_number_allocated();
        if index < 1 || index > limit {
            return Err(bad_vertex(index, limit));
        }
        Ok(())
    }
    /// `Standard_OutOfRange` unless `1 <= rank <= NbElements` (the getters' check).
    fn check_defined(&self, rank: i32) -> Result<(), Failure> {
        let limit = self.vertex_number();
        if rank < 1 || rank > limit {
            return Err(bad_vertex(rank, limit));
        }
        Ok(())
    }
    /// Raises `NbElements` to `index` when it is below.
    fn extend_to(&self, index: i32) {
        if let Some(buffer) = self.attribs.get() {
            buffer.set_nb_elements(index.max(buffer.nb_elements()));
        }
    }
    fn write_slot<T: super::Graphic3dBufferValue>(
        &self,
        slot: &Graphic3dAttributeSlot,
        index: i32,
        value: T,
    ) {
        if let (Some(buffer), Some(offset)) = (self.attribs.get(), slot_offset(slot, index)) {
            buffer.write_at(offset, value);
        }
    }
    fn read_slot<T: super::Graphic3dBufferValue>(
        &self,
        slot: &Graphic3dAttributeSlot,
        index: i32,
    ) -> Option<T> {
        self.attribs
            .get()
            .zip(slot_offset(slot, index))
            .and_then(|(buffer, offset)| buffer.read_at(offset))
    }
    fn position_slot(&self) -> Graphic3dAttributeSlot {
        Graphic3dAttributeSlot {
            offset: Some(0),
            stride: self.position_stride,
        }
    }
}

/// The `AddVertex` overloads, each returning the new 1-based index.
impl Graphic3dArrayOfPrimitives {
    /// `AddVertex(float x, y, z)`; `array.add_vertex(0.0, 0.0, 0.0)?`.
    pub fn add_vertex(&self, x: f32, y: f32, z: f32) -> Result<i32, Failure> {
        let index = self.vertex_number() + 1;
        self.set_vertice(index, x, y, z)?;
        Ok(index)
    }
    /// `AddVertex(double x, y, z)`: clamped to the float range; `array.add_vertex_f64(0.0, 0.0, 0.0)?`.
    pub fn add_vertex_f64(&self, x: f64, y: f64, z: f64) -> Result<i32, Failure> {
        self.add_vertex(
            real_to_short_real(x),
            real_to_short_real(y),
            real_to_short_real(z),
        )
    }
    /// `AddVertex(const gp_Pnt&)`; `array.add_vertex_pnt(&point)?`.
    pub fn add_vertex_pnt(&self, point: &Point3) -> Result<i32, Failure> {
        self.add_vertex_f64(point.x(), point.y(), point.z())
    }
    /// `AddVertex(const NCollection_Vec3<float>&)`; `array.add_vertex_vec3(Vec3::new(0.0, 0.0, 0.0))?`.
    pub fn add_vertex_vec3(&self, point: Vec3<f32>) -> Result<i32, Failure> {
        self.add_vertex(point.x, point.y, point.z)
    }
    /// `AddVertex(gp_Pnt, Quantity_Color)`; `array.add_vertex_pnt_color(&point, red)?`.
    pub fn add_vertex_pnt_color(&self, point: &Point3, color: Color) -> Result<i32, Failure> {
        let index = self.add_vertex_pnt(point)?;
        self.set_vertex_color_rgb(index, color.red(), color.green(), color.blue())?;
        Ok(index)
    }
    /// `AddVertex(gp_Pnt, int theColor32)`; `array.add_vertex_pnt_color32(&point, 0x00ff00ff)?`.
    pub fn add_vertex_pnt_color32(&self, point: &Point3, color: i32) -> Result<i32, Failure> {
        let index = self.add_vertex_pnt(point)?;
        self.set_vertex_color32(index, color)?;
        Ok(index)
    }
    /// `AddVertex(gp_Pnt, NCollection_Vec4<uint8_t>)`; `array.add_vertex_pnt_color4(&point, rgba)?`.
    pub fn add_vertex_pnt_color4(&self, point: &Point3, color: Vec4<u8>) -> Result<i32, Failure> {
        let index = self.add_vertex_pnt(point)?;
        self.set_vertex_color4(index, color)?;
        Ok(index)
    }
    /// `AddVertex(float x, y, z, nx, ny, nz)`; `array.add_vertex_normal([0.0; 3], [0.0, 0.0, 1.0])?`.
    pub fn add_vertex_normal(&self, point: [f32; 3], normal: [f32; 3]) -> Result<i32, Failure> {
        let index = self.vertex_number() + 1;
        self.set_vertice(index, point[0], point[1], point[2])?;
        self.set_vertex_normal(
            index,
            f64::from(normal[0]),
            f64::from(normal[1]),
            f64::from(normal[2]),
        )?;
        Ok(index)
    }
    /// `AddVertex(double x, y, z, nx, ny, nz)`; `array.add_vertex_normal_f64([0.0; 3], [0.0, 0.0, 1.0])?`.
    pub fn add_vertex_normal_f64(&self, point: [f64; 3], normal: [f64; 3]) -> Result<i32, Failure> {
        self.add_vertex_normal(point.map(real_to_short_real), normal.map(|c| c as f32))
    }
    /// `AddVertex(gp_Pnt, gp_Dir)`; `array.add_vertex_pnt_normal(&point, &normal)?`.
    pub fn add_vertex_pnt_normal(
        &self,
        point: &Point3,
        normal: &Direction3,
    ) -> Result<i32, Failure> {
        self.add_vertex_normal_f64(
            [point.x(), point.y(), point.z()],
            [normal.x(), normal.y(), normal.z()],
        )
    }
    /// `AddVertex(gp_Pnt, gp_Dir, Quantity_Color)`; `array.add_vertex_pnt_normal_color(&p, &n, red)?`.
    pub fn add_vertex_pnt_normal_color(
        &self,
        point: &Point3,
        normal: &Direction3,
        color: Color,
    ) -> Result<i32, Failure> {
        let index = self.add_vertex_pnt_normal(point, normal)?;
        self.set_vertex_color_rgb(index, color.red(), color.green(), color.blue())?;
        Ok(index)
    }
    /// `AddVertex(gp_Pnt, gp_Dir, int theColor32)`; `array.add_vertex_pnt_normal_color32(&p, &n, c)?`.
    pub fn add_vertex_pnt_normal_color32(
        &self,
        point: &Point3,
        normal: &Direction3,
        color: i32,
    ) -> Result<i32, Failure> {
        let index = self.add_vertex_pnt_normal(point, normal)?;
        self.set_vertex_color32(index, color)?;
        Ok(index)
    }
    /// `AddVertex(float x, y, z, tx, ty)`; `array.add_vertex_texel([0.0; 3], [0.5, 0.5])?`.
    pub fn add_vertex_texel(&self, point: [f32; 3], texel: [f32; 2]) -> Result<i32, Failure> {
        let index = self.vertex_number() + 1;
        self.set_vertice(index, point[0], point[1], point[2])?;
        self.set_vertex_texel(index, f64::from(texel[0]), f64::from(texel[1]))?;
        Ok(index)
    }
    /// `AddVertex(double x, y, z, tx, ty)`; `array.add_vertex_texel_f64([0.0; 3], [0.5, 0.5])?`.
    pub fn add_vertex_texel_f64(&self, point: [f64; 3], texel: [f64; 2]) -> Result<i32, Failure> {
        self.add_vertex_texel(point.map(real_to_short_real), texel.map(|c| c as f32))
    }
    /// `AddVertex(gp_Pnt, gp_Pnt2d)`; `array.add_vertex_pnt_texel(&point, &texel)?`.
    pub fn add_vertex_pnt_texel(&self, point: &Point3, texel: &Point2) -> Result<i32, Failure> {
        self.add_vertex_texel_f64([point.x(), point.y(), point.z()], [texel.x(), texel.y()])
    }
    /// `AddVertex(float x, y, z, nx, ny, nz, tx, ty)`; `array.add_vertex_normal_texel(p, n, t)?`.
    pub fn add_vertex_normal_texel(
        &self,
        point: [f32; 3],
        normal: [f32; 3],
        texel: [f32; 2],
    ) -> Result<i32, Failure> {
        let index = self.add_vertex_normal(point, normal)?;
        self.set_vertex_texel(index, f64::from(texel[0]), f64::from(texel[1]))?;
        Ok(index)
    }
    /// `AddVertex(double x, y, z, nx, ny, nz, tx, ty)`; `array.add_vertex_normal_texel_f64(p, n, t)?`.
    pub fn add_vertex_normal_texel_f64(
        &self,
        point: [f64; 3],
        normal: [f64; 3],
        texel: [f64; 2],
    ) -> Result<i32, Failure> {
        self.add_vertex_normal_texel(
            point.map(real_to_short_real),
            normal.map(|c| c as f32),
            texel.map(|c| c as f32),
        )
    }
    /// `AddVertex(gp_Pnt, gp_Dir, gp_Pnt2d)`; `array.add_vertex_pnt_normal_texel(&p, &n, &t)?`.
    pub fn add_vertex_pnt_normal_texel(
        &self,
        point: &Point3,
        normal: &Direction3,
        texel: &Point2,
    ) -> Result<i32, Failure> {
        self.add_vertex_normal_texel_f64(
            [point.x(), point.y(), point.z()],
            [normal.x(), normal.y(), normal.z()],
            [texel.x(), texel.y()],
        )
    }
}

/// The vertex setters (1-based indices up to the allocated count).
impl Graphic3dArrayOfPrimitives {
    /// `SetVertice(index, float x, y, z)`; `array.set_vertice(1, 0.0, 1.0, 0.0)?`.
    pub fn set_vertice(&self, index: i32, x: f32, y: f32, z: f32) -> Result<(), Failure> {
        self.check_allocated(index)?;
        self.write_slot(&self.position_slot(), index, Vec3::new(x, y, z));
        self.extend_to(index);
        Ok(())
    }
    /// `SetVertice(index, gp_Pnt)`: narrowed to float; `array.set_vertice_pnt(1, &point)?`.
    pub fn set_vertice_pnt(&self, index: i32, point: &Point3) -> Result<(), Failure> {
        self.set_vertice(index, point.x() as f32, point.y() as f32, point.z() as f32)
    }
    /// `SetVertexColor(index, r, g, b)`: bytes `(uint8_t)(c * 255)`, alpha 255; `array.set_vertex_color_rgb(1, 1.0, 0.0, 0.0)?`.
    pub fn set_vertex_color_rgb(&self, index: i32, r: f64, g: f64, b: f64) -> Result<(), Failure> {
        let byte = |c: f64| int_from_f64(c * 255.0) as u8;
        self.set_vertex_color4(index, Vec4::new(byte(r), byte(g), byte(b), 255))
    }
    /// `SetVertexColor(index, Quantity_Color)`; `array.set_vertex_color(1, red)?`.
    pub fn set_vertex_color(&self, index: i32, color: Color) -> Result<(), Failure> {
        self.set_vertex_color_rgb(index, color.red(), color.green(), color.blue())
    }
    /// `SetVertexColor(index, NCollection_Vec4<uint8_t>)`; `array.set_vertex_color4(1, rgba)?`.
    pub fn set_vertex_color4(&self, index: i32, color: Vec4<u8>) -> Result<(), Failure> {
        self.check_allocated(index)?;
        self.write_slot(&self.colors, index, color);
        self.extend_to(index);
        Ok(())
    }
    /// `SetVertexColor(index, int theColor32)`. Native quirk: unlike the other setters it does
    /// not raise the vertex count. Example: `array.set_vertex_color32(1, -1)?`.
    pub fn set_vertex_color32(&self, index: i32, color: i32) -> Result<(), Failure> {
        self.check_allocated(index)?;
        self.write_slot(&self.colors, index, color);
        Ok(())
    }
    /// `SetVertexNormal(index, nx, ny, nz)`: narrowed to float; `array.set_vertex_normal(1, 0.0, 0.0, 1.0)?`.
    pub fn set_vertex_normal(&self, index: i32, nx: f64, ny: f64, nz: f64) -> Result<(), Failure> {
        self.check_allocated(index)?;
        self.write_slot(
            &self.normals,
            index,
            Vec3::new(nx as f32, ny as f32, nz as f32),
        );
        self.extend_to(index);
        Ok(())
    }
    /// `SetVertexNormal(index, gp_Dir)`; `array.set_vertex_normal_dir(1, &normal)?`.
    pub fn set_vertex_normal_dir(&self, index: i32, normal: &Direction3) -> Result<(), Failure> {
        self.set_vertex_normal(index, normal.x(), normal.y(), normal.z())
    }
    /// `SetVertexTexel(index, tx, ty)`: narrowed to float; `array.set_vertex_texel(1, 0.5, 0.5)?`.
    pub fn set_vertex_texel(&self, index: i32, tx: f64, ty: f64) -> Result<(), Failure> {
        self.check_allocated(index)?;
        self.write_slot(&self.texels, index, Vec2::new(tx as f32, ty as f32));
        self.extend_to(index);
        Ok(())
    }
    /// `SetVertexTexel(index, gp_Pnt2d)`; `array.set_vertex_texel_pnt2d(1, &texel)?`.
    pub fn set_vertex_texel_pnt2d(&self, index: i32, texel: &Point2) -> Result<(), Failure> {
        self.set_vertex_texel(index, texel.x(), texel.y())
    }
}

/// The vertex getters (1-based ranks up to the vertex count).
impl Graphic3dArrayOfPrimitives {
    /// `Vertice(rank, x, y, z)`; `array.vertice_xyz(1)?`.
    pub fn vertice_xyz(&self, rank: i32) -> Result<[f64; 3], Failure> {
        self.check_defined(rank)?;
        let value: Vec3<f32> = self
            .read_slot(&self.position_slot(), rank)
            .unwrap_or(Vec3::new(0.0, 0.0, 0.0));
        Ok([value.x, value.y, value.z].map(f64::from))
    }
    /// `Vertice(rank)`; `array.vertice(1)?`.
    pub fn vertice(&self, rank: i32) -> Result<Point3, Failure> {
        let [x, y, z] = self.vertice_xyz(rank)?;
        Ok(Point3::new(x, y, z))
    }
    /// `VertexColor(rank, r, g, b)`: bytes / 255, zeros without colors; `array.vertex_color_rgb(1)?`.
    pub fn vertex_color_rgb(&self, rank: i32) -> Result<[f64; 3], Failure> {
        self.check_defined(rank)?;
        let Some(color) = self.read_slot::<Vec4<u8>>(&self.colors, rank) else {
            return Ok([0.0; 3]);
        };
        Ok([color.x, color.y, color.z].map(|c| f64::from(c) / 255.0))
    }
    /// `VertexColor(rank)`; `array.vertex_color(1)?`.
    pub fn vertex_color(&self, rank: i32) -> Result<Color, Failure> {
        let [r, g, b] = self.vertex_color_rgb(rank)?;
        Color::new(r, g, b, ColorSpace::Rgb)
    }
    /// `VertexColor(index, NCollection_Vec4<uint8_t>&)`; raises without colors. `array.vertex_color4(1)?`.
    pub fn vertex_color4(&self, index: i32) -> Result<Vec4<u8>, Failure> {
        if self.colors.offset.is_none() {
            return Err(out_of_range(&format!(
                "BAD VERTEX index; got {index} for an array without vertex colors"
            )));
        }
        self.check_defined(index)?;
        Ok(self
            .read_slot(&self.colors, index)
            .unwrap_or(Vec4::new(0, 0, 0, 0)))
    }
    /// `VertexColor(rank, int&)`: the packed color, `None` (output untouched natively) without
    /// colors. Example: `array.vertex_color32(1)?`.
    pub fn vertex_color32(&self, rank: i32) -> Result<Option<i32>, Failure> {
        self.check_defined(rank)?;
        Ok(self.read_slot(&self.colors, rank))
    }
    /// `VertexNormal(rank, nx, ny, nz)`: zeros without normals; `array.vertex_normal_xyz(1)?`.
    pub fn vertex_normal_xyz(&self, rank: i32) -> Result<[f64; 3], Failure> {
        self.check_defined(rank)?;
        let value: Option<Vec3<f32>> = self.read_slot(&self.normals, rank);
        Ok(value.map_or([0.0; 3], |n| [n.x, n.y, n.z].map(f64::from)))
    }
    /// `VertexNormal(rank)`: a zero normal raises the `gp_Dir` construction error, as natively.
    /// Example: `array.vertex_normal(1)?`.
    pub fn vertex_normal(&self, rank: i32) -> Result<Direction3, Failure> {
        let [x, y, z] = self.vertex_normal_xyz(rank)?;
        Direction3::new(x, y, z)
    }
    /// `VertexTexel(rank, tx, ty)`: zeros without texels; `array.vertex_texel_xy(1)?`.
    pub fn vertex_texel_xy(&self, rank: i32) -> Result<[f64; 2], Failure> {
        self.check_defined(rank)?;
        let value: Option<Vec2<f32>> = self.read_slot(&self.texels, rank);
        Ok(value.map_or([0.0; 2], |t| [t.x, t.y].map(f64::from)))
    }
    /// `VertexTexel(rank)`; `array.vertex_texel(1)?`.
    pub fn vertex_texel(&self, rank: i32) -> Result<Point2, Failure> {
        let [x, y] = self.vertex_texel_xy(rank)?;
        Ok(Point2::new(x, y))
    }
}
