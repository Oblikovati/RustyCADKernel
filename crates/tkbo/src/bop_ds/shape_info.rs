//! Cached shape metadata used by the Boolean operation data structure.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use std::fmt::Write;

/// `BOPDS_ShapeInfo`; stores a shape, its type, bounds, subshape indices and flags.
#[derive(Clone)]
pub struct ShapeInfo {
    shape: Shape,
    shape_type: ShapeEnum,
    bounds: BoundingBox3,
    sub_shapes: Vec<i32>,
    reference: i32,
    flag: i32,
}

impl Default for ShapeInfo {
    fn default() -> Self {
        Self {
            shape: Shape::default(),
            shape_type: ShapeEnum::Shape,
            bounds: BoundingBox3::default(),
            sub_shapes: Vec::new(),
            reference: -1,
            flag: -1,
        }
    }
}

impl ShapeInfo {
    /// Stores a shape copy; `info.set_shape(&shape)`.
    pub fn set_shape(&mut self, shape: &Shape) {
        self.shape = shape.clone();
    }

    /// Borrows the stored shape; `info.shape().is_null()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Sets the stored type independently of the shape handle.
    pub fn set_shape_type(&mut self, shape_type: ShapeEnum) {
        self.shape_type = shape_type;
    }

    /// Reads the stored type; `ShapeInfo::default().shape_type() == ShapeEnum::Shape`.
    pub fn shape_type(&self) -> ShapeEnum {
        self.shape_type
    }

    /// Copies the supplied bounding box; `info.set_box(&bounds)`.
    pub fn set_box(&mut self, bounds: &BoundingBox3) {
        self.bounds = bounds.clone();
    }

    /// Borrows the stored bounding box; `info.box_bounds().is_void()`.
    pub fn box_bounds(&self) -> &BoundingBox3 {
        &self.bounds
    }

    /// Mutably borrows the stored bounding box; `info.change_box().set_gap(1e-6)`.
    pub fn change_box(&mut self) -> &mut BoundingBox3 {
        &mut self.bounds
    }

    /// Borrows subshape indices in insertion order.
    pub fn sub_shapes(&self) -> &[i32] {
        &self.sub_shapes
    }

    /// Mutably borrows subshape indices; duplicates are retained.
    pub fn change_sub_shapes(&mut self) -> &mut Vec<i32> {
        &mut self.sub_shapes
    }

    /// Tests whether `index` occurs in the subshape list.
    pub fn has_sub_shape(&self, index: i32) -> bool {
        self.sub_shapes.contains(&index)
    }

    /// Reports whether the reference index is nonnegative.
    pub fn has_reference(&self) -> bool {
        self.reference >= 0
    }

    /// Sets the reference index without validation.
    pub fn set_reference(&mut self, reference: i32) {
        self.reference = reference;
    }

    /// Reads the reference index; the default sentinel is -1.
    pub fn reference(&self) -> i32 {
        self.reference
    }

    /// Reports whether this type has boundary representation.
    pub fn has_brep(&self) -> bool {
        matches!(
            self.shape_type,
            ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face
        )
    }

    /// Reports whether the type can participate in an interference.
    pub fn is_interfering(&self) -> bool {
        self.has_brep() || self.shape_type == ShapeEnum::Solid
    }

    /// Reports whether the flag is nonnegative.
    pub fn has_flag(&self) -> bool {
        self.flag >= 0
    }

    /// Copies the flag to `output` and reports whether it is nonnegative.
    pub fn has_flag_into(&self, output: &mut i32) -> bool {
        *output = self.flag;
        self.has_flag()
    }

    /// Sets the flag without validation.
    pub fn set_flag(&mut self, flag: i32) {
        self.flag = flag;
    }

    /// Reads the flag; the default sentinel is -1.
    pub fn flag(&self) -> i32 {
        self.flag
    }

    /// Writes native `Dump` text; `info.dump(&mut String::new())?`.
    pub fn dump(&self, output: &mut dyn Write) -> std::fmt::Result {
        write!(
            output,
            " {} {{",
            rustycad_tkg3d::top_abs::shape_type_to_string(self.shape_type)
        )?;
        for index in &self.sub_shapes {
            write!(output, " {index}")?;
        }
        output.write_str(" }")
    }
}
