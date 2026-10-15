//! `BRepFont_Builder`: builds glyph and text topology as TopoDS shapes or in a caller-owned
//! BRepGraph. The glyph builders are in `builder_glyph.rs` and `builder_graph.rs`.
use super::planar_region::BRepFontPlanarRegion;
use super::regularizer::is_valid_real;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkmath::gp::{self, Axis3, Transform3, Vector3, Xy};
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;

/// `BRepFont_Builder::GlyphOptions`.
/// Example: `BRepFontBuilderGlyphOptions { size: 10.0, ..Default::default() }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRepFontBuilderGlyphOptions {
    /// `Size`: the requested glyph height.
    pub size: f64,
    /// `Tolerance` assigned to the generated topology.
    pub tolerance: f64,
    /// `ToConcatenateContours`: one C0 B-spline edge per contour (TopoDS output).
    pub to_concatenate_contours: bool,
}

impl Default for BRepFontBuilderGlyphOptions {
    fn default() -> Self {
        Self {
            size: 1.0,
            tolerance: 1.0e-7,
            to_concatenate_contours: false,
        }
    }
}

/// `BRepFont_Builder::TextPlan::Item`: one positioned use of a plan region.
/// Example: `BRepFontBuilderTextPlanItem::new(0, Xy::new(20.0, 0.0))`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRepFontBuilderTextPlanItem {
    region: usize,
    position: Xy,
}

impl BRepFontBuilderTextPlanItem {
    /// `Item(theRegion, thePosition)`; `BRepFontBuilderTextPlanItem::new(0, position)`.
    pub fn new(region: usize, position: Xy) -> Self {
        Self { region, position }
    }
    /// `Region`: the index of the plan region; `item.region()`.
    pub fn region(&self) -> usize {
        self.region
    }
    /// `Position` in model units; `item.position()`.
    pub fn position(&self) -> &Xy {
        &self.position
    }
}

/// `BRepFont_Builder::TextPlan`: a text layout over unique shared glyph regions.
/// Built only by [`BRepFontBuilder::create_text_plan`]; `plan.items().len()`.
#[derive(Clone, Debug)]
pub struct BRepFontBuilderTextPlan {
    size: f64,
    tolerance: f64,
    lower_left: Xy,
    upper_right: Xy,
    regions: Vec<Arc<BRepFontPlanarRegion>>,
    items: Vec<BRepFontBuilderTextPlanItem>,
}

impl BRepFontBuilderTextPlan {
    /// `Size`; `plan.size()`.
    pub fn size(&self) -> f64 {
        self.size
    }
    /// `Tolerance`; `plan.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
    /// `NbRegions`; `plan.nb_regions()`.
    pub fn nb_regions(&self) -> usize {
        self.regions.len()
    }
    /// `Region(theIndex)`; panics out of range (native `Value` raises); `plan.region(0)`.
    pub fn region(&self, index: usize) -> &BRepFontPlanarRegion {
        &self.regions[index]
    }
    /// `RegionHandle(theIndex)`: the shared region; `Arc::ptr_eq` is the native pointer
    /// identity. Example: `plan.region_handle(0)`.
    pub fn region_handle(&self, index: usize) -> &Arc<BRepFontPlanarRegion> {
        &self.regions[index]
    }
    /// `LowerLeft` of the formatted bounds in model units; `plan.lower_left()`.
    pub fn lower_left(&self) -> &Xy {
        &self.lower_left
    }
    /// `UpperRight` of the formatted bounds in model units; `plan.upper_right()`.
    pub fn upper_right(&self) -> &Xy {
        &self.upper_right
    }
    /// `Width`; `plan.width()`.
    pub fn width(&self) -> f64 {
        self.upper_right.x() - self.lower_left.x()
    }
    /// `Height`; `plan.height()`.
    pub fn height(&self) -> f64 {
        self.upper_right.y() - self.lower_left.y()
    }
    /// `Items`; `plan.items().len()`.
    pub fn items(&self) -> &[BRepFontBuilderTextPlanItem] {
        &self.items
    }
}

/// `BRepFont_Builder::TextOptions`.
/// Example: `BRepFontBuilderTextOptions { pen: Axis3::default(), to_concatenate_contours: true }`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepFontBuilderTextOptions {
    /// `Pen`: the position and orientation of the text baseline.
    pub pen: Axis3,
    /// `ToConcatenateContours`: one C0 B-spline edge per contour (TopoDS output).
    pub to_concatenate_contours: bool,
}

/// `BRepFont_Builder`: static glyph and text topology builders (the native class cannot
/// be instantiated). Example: `BRepFontBuilder::build_glyph(&region, &options)`.
#[derive(Clone, Copy, Debug)]
pub enum BRepFontBuilder {}

/// `isValidPosition`: every coordinate of the pen is a valid real.
pub(super) fn is_valid_pen(pen: &Axis3) -> bool {
    let location = pen.location();
    let (direction, x_direction) = (pen.direction(), pen.x_direction());
    [
        location.x(),
        location.y(),
        location.z(),
        direction.x(),
        direction.y(),
        direction.z(),
        x_direction.x(),
        x_direction.y(),
        x_direction.z(),
    ]
    .into_iter()
    .all(is_valid_real)
}

/// The size and tolerance checks shared by the text builders.
pub(super) fn is_valid_plan(plan: &BRepFontBuilderTextPlan, pen: &Axis3) -> bool {
    is_valid_real(plan.size())
        && plan.size() > 0.0
        && is_valid_real(plan.tolerance())
        && plan.tolerance() > 0.0
        && is_valid_pen(pen)
        && !plan.items().is_empty()
}

/// `aPenTransform.SetTransformation(Pen, gp_Ax3(gp::XOY()))`.
pub(super) fn pen_transform(pen: &Axis3) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_transformation_between(*pen, Axis3::from_axis2(gp::xoy()));
    transform
}

/// The translation of a plan item.
pub(super) fn item_translation(item: &BRepFontBuilderTextPlanItem) -> Transform3 {
    let mut position = Transform3::default();
    position.set_translation(Vector3::new(item.position().x(), item.position().y(), 0.0));
    position
}

fn is_valid_plan_inputs(size: f64, tolerance: f64, lower_left: &Xy, upper_right: &Xy) -> bool {
    is_valid_real(size)
        && size > 0.0
        && is_valid_real(tolerance)
        && tolerance > 0.0
        && is_valid_real(lower_left.x())
        && is_valid_real(lower_left.y())
        && is_valid_real(upper_right.x())
        && is_valid_real(upper_right.y())
        && upper_right.x() >= lower_left.x()
        && upper_right.y() >= lower_left.y()
}

impl BRepFontBuilder {
    /// `CreateTextPlan`: a validated plan, or None when an option or a region reference is
    /// invalid. Example:
    /// `BRepFontBuilder::create_text_plan(12.0, 1e-7, Xy::new(0.0, 0.0), Xy::new(0.0, 0.0), vec![], vec![])`.
    pub fn create_text_plan(
        size: f64,
        tolerance: f64,
        lower_left: Xy,
        upper_right: Xy,
        regions: Vec<Arc<BRepFontPlanarRegion>>,
        items: Vec<BRepFontBuilderTextPlanItem>,
    ) -> Option<BRepFontBuilderTextPlan> {
        if !is_valid_plan_inputs(size, tolerance, &lower_left, &upper_right) {
            return None;
        }
        let valid_region = |region: &Arc<BRepFontPlanarRegion>| {
            is_valid_real(region.units_per_em())
                && region.units_per_em() > 0.0
                && is_valid_real(region.tolerance())
                && region.tolerance() > 0.0
        };
        let valid_item = |item: &BRepFontBuilderTextPlanItem| {
            item.region() < regions.len()
                && is_valid_real(item.position().x())
                && is_valid_real(item.position().y())
        };
        if !regions.iter().all(valid_region) || !items.iter().all(valid_item) {
            return None;
        }
        Some(BRepFontBuilderTextPlan {
            size,
            tolerance,
            lower_left,
            upper_right,
            regions,
            items,
        })
    }

    /// `BuildText(thePlan)` with default options; `BRepFontBuilder::build_text(&plan)`.
    pub fn build_text(plan: &BRepFontBuilderTextPlan) -> Shape {
        Self::build_text_with(plan, &BRepFontBuilderTextOptions::default())
    }

    /// `BuildText(thePlan, theOptions)`: each unique region is built once and placed per
    /// item; a null shape on failure or for empty text.
    /// Example: `BRepFontBuilder::build_text_with(&plan, &options)`.
    pub fn build_text_with(
        plan: &BRepFontBuilderTextPlan,
        options: &BRepFontBuilderTextOptions,
    ) -> Shape {
        if !is_valid_plan(plan, &options.pen) {
            return Shape::default();
        }
        let glyph_options = BRepFontBuilderGlyphOptions {
            size: plan.size(),
            tolerance: plan.tolerance(),
            to_concatenate_contours: options.to_concatenate_contours,
        };
        let mut glyphs: Vec<Option<Shape>> = vec![None; plan.nb_regions()];
        for item in plan.items() {
            if glyphs[item.region()].is_some() {
                continue;
            }
            let region = plan.region(item.region());
            let shape = Self::build_glyph(region, &glyph_options);
            if shape.is_null() && !region.is_empty() {
                return Shape::default();
            }
            glyphs[item.region()] = Some(shape);
        }
        let built: Vec<Shape> = glyphs.into_iter().map(Option::unwrap_or_default).collect();
        Self::build_text_from_glyphs(plan, &built, options)
    }

    /// `BuildText(thePlan, theGlyphShapes, theOptions)`: places one prebuilt shape per
    /// region at each item; a null shape when the input is invalid or the text empty.
    /// Example: `BRepFontBuilder::build_text_from_glyphs(&plan, &shapes, &options)`.
    pub fn build_text_from_glyphs(
        plan: &BRepFontBuilderTextPlan,
        glyph_shapes: &[Shape],
        options: &BRepFontBuilderTextOptions,
    ) -> Shape {
        if !is_valid_plan(plan, &options.pen) || glyph_shapes.len() != plan.nb_regions() {
            return Shape::default();
        }
        place_glyphs(plan, glyph_shapes, &options.pen).unwrap_or_default()
    }
}

fn place_glyphs(
    plan: &BRepFontBuilderTextPlan,
    glyph_shapes: &[Shape],
    pen: &Axis3,
) -> Option<Shape> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for item in plan.items() {
        let mut glyph = glyph_shapes.get(item.region())?.clone();
        if glyph.is_null() {
            if !plan.region(item.region()).is_empty() {
                return None;
            }
            continue;
        }
        glyph
            .move_by(&Location::new(item_translation(item)), true)
            .ok()?;
        builder.add(&mut compound, &glyph).ok()?;
    }
    if compound.nb_children() == 0 {
        return None;
    }
    compound
        .move_by(&Location::new(pen_transform(pen)), true)
        .ok()?;
    Some(compound.into_shape())
}
