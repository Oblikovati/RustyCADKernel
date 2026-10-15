//! The concrete `AIS_Shape`: wireframe, shaded and bounding box presentations of a topological
//! shape, its hidden line presentations, its selection and the shape parameters (texture
//! mapping, deviation, HLR type), over the inherited `AisShapeBase` state.
use super::shape_bounds::shape_presentation_bounds_snapshot;
use super::shape_compute_bounds::ais_shape_compute_bounds_mode;
use super::*;
use crate::prs_mgr::*;
use crate::prs3d::{Prs3dDrawer, Prs3dTypeOfHlr};
use crate::select_mgr::*;
use crate::std_prs::{
    StdPrsHlrPolyShape, StdPrsHlrShape, StdPrsHlrShapeI, StdPrsShadedShape,
    StdPrsToolTriangulatedShape, StdPrsVolume, StdPrsWfShape,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::Message;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, StandardTransient, catch_native_failure,
};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point2, TransformForm};
use rustycad_tkmath::toploc::{Datum3, Location};
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer, field_real};
use std::io;

/// `AIS_Shape`: the presentation of a topological shape.
/// Example: `let object = AisShape::new(&shape); context.display(&object.cast(), true)?;`.
pub struct AisShape {
    transient: StandardTransient,
    base: AisShapeBase,
}
rustycad_tkernel::impl_transient!(AisShape, "AIS_Shape", dyn AisInteractiveObject, transient,
    interfaces: [dyn AisShapeSource, dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl AisShape {
    /// `AIS_Shape(shape)`: projector dependent, UV origin (0, 0), repeat and scale (1, 1).
    /// Example: `AisShape::new(&shape)`.
    pub fn new(shape: &Shape) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisShapeBase::new(shape),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}

impl AisShapeSource for AisShape {
    fn shape_base(&self) -> &AisShapeBase {
        &self.base
    }
}

impl AisInteractiveObject for AisShape {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
    /// `Type`: `AIS_KindOfInteractive_Shape`.
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Shape
    }
    /// `Signature`: 0.
    fn signature(&self) -> i32 {
        0
    }
}

impl PrsMgrPresentableObject for AisShape {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.interactive.presentable
    }
    /// `AcceptDisplayMode`: the modes 0 (wireframe), 1 (shaded) and 2 (bounding box).
    fn accept_display_mode(&self, mode: i32) -> bool {
        (0..=2).contains(&mode)
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_shape_compute(self, &prs.observe(), mode)
    }
    fn compute_hlr(
        &self,
        projector: &Handle<Graphic3dCamera>,
        trsf: &Handle<Datum3>,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        ais_shape_compute_hlr(self, projector, trsf, prs)
    }
    fn color(&self) -> Color {
        super::shape_appearance_queries::shape_color_value(self)
    }
    fn material(&self) -> Graphic3dNameOfMaterial {
        ais_shape_material(self)
    }
    fn transparency(&self) -> f64 {
        ais_shape_transparency(self)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        ais_shape_set_color(self, color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        ais_shape_unset_color(self)
    }
    fn set_material(&self, material: &Graphic3dMaterialAspect) -> Result<(), Failure> {
        ais_shape_set_material(self, material)
    }
    fn unset_material(&self) -> Result<(), Failure> {
        ais_shape_unset_material(self)
    }
    fn set_width(&self, value: f64) -> Result<(), Failure> {
        ais_shape_set_width(self, value)
    }
    fn unset_width(&self) -> Result<(), Failure> {
        ais_shape_unset_width(self)
    }
    fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        ais_shape_set_transparency(self, value)
    }
    fn unset_transparency(&self) -> Result<(), Failure> {
        ais_shape_unset_transparency(self)
    }
    crate::select_mgr_presentable_overrides!(dump = ais_shape_dump_json);
}

impl SelectMgrSelectableObject for AisShape {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_shape_compute_selection(self, selection, mode)
    }
    /// `AcceptShapeDecomposition`: true.
    fn accept_shape_decomposition(&self) -> bool {
        true
    }
}

/// Sends a native "Error: AIS_Shape::Compute() ... has failed (...)" message.
fn report_compute_failure(what: &str, failure: &Failure) {
    let text = format!(
        "Error: AIS_Shape::Compute() {what} has failed ({})",
        failure.message()
    );
    Message::default_messenger().send_fail_text(&AsciiString::from_bytes(text.as_bytes()));
}

/// The native `AIS_Shape::Compute` body: a wire, edge or vertex gets a raised display
/// priority and every structure type; mode 0 is the wireframe, mode 1 the shading (the
/// wireframe for a shape simpler than a shell, an infinite one, or when the shading builder
/// fails), mode 2 the bounding box. Builder failures are reported, not returned.
/// Example: `ais_shape_compute(&*object, &prs, 1)?`.
pub fn ais_shape_compute<S: AisShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    if mode == 2 {
        return ais_shape_compute_bounds_mode(object, prs);
    }
    let shape = object.shape_base().fields.read().shape.clone();
    if shape.is_null() {
        return Ok(());
    }
    let kind = shape.shape_type()?;
    if kind == ShapeEnum::Compound && shape.nb_children() == 0 {
        return Ok(());
    }
    if kind >= ShapeEnum::Wire && kind <= ShapeEnum::Vertex {
        let priority = match kind {
            ShapeEnum::Wire => Graphic3dDisplayPriority::Above1,
            ShapeEnum::Edge => Graphic3dDisplayPriority::Above2,
            _ => Graphic3dDisplayPriority::Highlight,
        };
        prs.set_visual(Graphic3dTypeOfStructure::All);
        prs.set_display_priority(priority)?;
    }
    if object.is_infinite() {
        prs.set_infinite_state(true);
    }
    let drawer = object.attributes();
    match mode {
        0 => {
            StdPrsToolTriangulatedShape::clear_on_own_deflection_change(&shape, &drawer, true)?;
            if let Err(failure) = catch_native_failure(|| StdPrsWfShape::add(prs, &shape, &drawer)) {
                report_compute_failure("wireframe presentation builder", &failure);
            }
        }
        1 => {
            StdPrsToolTriangulatedShape::clear_on_own_deflection_change(&shape, &drawer, true)?;
            if kind as i32 > 4 || object.is_infinite() {
                StdPrsWfShape::add(prs, &shape, &drawer)?;
            } else {
                let fields = object.shape_base().fields.read().clone();
                let aspect = drawer.shading_aspect().aspect();
                let textured = aspect.to_map_texture() && !aspect.texture_map().is_null();
                let shaded = catch_native_failure(|| {
                    StdPrsShadedShape::add_with_texels(
                        prs,
                        &shape,
                        &drawer,
                        textured,
                        fields.uv_origin,
                        fields.uv_repeat,
                        fields.uv_scale,
                        StdPrsVolume::Autodetection,
                        &Handle::default(),
                    )
                });
                if let Err(failure) = shaded {
                    report_compute_failure("shaded presentation builder", &failure);
                    StdPrsWfShape::add(prs, &shape, &drawer)?;
                }
            }
            let transparency = object.transparency();
            if transparency > 0.0 {
                object.set_transparency(transparency)?;
            }
        }
        _ => {}
    }
    prs.re_compute();
    Ok(())
}

/// The native `AIS_Shape::computeHLR` body: the shape moved by a non-identity
/// transformation, then [`ais_shape_compute_hlr_presentation`] with the object drawer.
/// Example: `ais_shape_compute_hlr(&*object, &camera, &trsf, &prs)?`.
pub fn ais_shape_compute_hlr<S: AisShapeSource + ?Sized>(
    object: &S,
    projector: &Handle<Graphic3dCamera>,
    trsf: &Handle<Datum3>,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let shape = object.shape_base().fields.read().shape.clone();
    let drawer = object.attributes();
    match trsf.get() {
        Some(datum) if datum.trsf().form() != TransformForm::Identity => {
            let location = Location::new(*datum.trsf()).multiplied(shape.location())?;
            let moved = shape.located(&location, true)?;
            ais_shape_compute_hlr_presentation(projector, prs, &moved, &drawer)
        }
        _ => ais_shape_compute_hlr_presentation(projector, prs, &shape, &drawer),
    }
}

/// `AIS_Shape::computeHlrPresentation`: a vertex, edge or wire is drawn as wireframe below
/// the others; otherwise the hidden line flag follows the link drawer, the link deflection is
/// relative during the computation, and the exact (Algo) or polygonal (PolyAlgo, NotSet)
/// HLR builder runs, falling back to the wireframe when it fails.
/// Example: `ais_shape_compute_hlr_presentation(&camera, &prs, &shape, &drawer)?`.
pub fn ais_shape_compute_hlr_presentation(
    projector: &Handle<Graphic3dCamera>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
    drawer: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    if shape.is_null() {
        return Ok(());
    }
    match shape.shape_type()? {
        ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Wire => {
            prs.set_display_priority(Graphic3dDisplayPriority::Below)?;
            return StdPrsWfShape::add(prs, shape, drawer);
        }
        ShapeEnum::Compound if shape.nb_children() == 0 => return Ok(()),
        _ => {}
    }
    let link = drawer.link();
    let default_drawer = link.get().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", "AIS_Shape::computeHlrPresentation: no link drawer")
    })?;
    if default_drawer.draw_hidden_line() {
        drawer.enable_draw_hidden_line();
    } else {
        drawer.disable_draw_hidden_line();
    }
    let previous = default_drawer.type_of_deflection();
    default_drawer.set_type_of_deflection(AspectTypeOfDeflection::Relative);
    if drawer.is_auto_triangulation() {
        StdPrsToolTriangulatedShape::clear_on_own_deflection_change(shape, drawer, true)?;
    }
    let computed = catch_native_failure(|| match drawer.type_of_hlr() {
        Prs3dTypeOfHlr::Algo => StdPrsHlrShape::default().compute_hlr(prs, shape, drawer, projector),
        Prs3dTypeOfHlr::PolyAlgo | Prs3dTypeOfHlr::NotSet => {
            StdPrsHlrPolyShape::default().compute_hlr(prs, shape, drawer, projector)
        }
    });
    if let Err(failure) = computed {
        let text = format!(
            "Error: AIS_Shape::Compute() HLR Algorithm has failed ({})",
            failure.message()
        );
        Message::default_messenger().send_fail_text(&AsciiString::from_bytes(text.as_bytes()));
        StdPrsWfShape::add(prs, shape, drawer)?;
    }
    default_drawer.set_type_of_deflection(previous);
    Ok(())
}

/// The native `AIS_Shape::DumpJson` body: the interactive object, the shape, the bounding
/// box, the initial angle and the bounding box flag.
/// Example: `ais_shape_dump_json(&*object, &mut stream, -1)?`.
pub fn ais_shape_dump_json<S: AisShapeSource + ?Sized>(
    object: &S,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "AIS_Shape")?;
    base_class(stream, depth, &|s, d| ais_interactive_object_dump_json(object, s, d))?;
    let fields = object.shape_base().fields.read().clone();
    field_dumped(stream, depth, "&myshape", &|s, d| fields.shape.dump_json(s, d))?;
    field_dumped(stream, depth, "&myBB", &|s, _| fields.bounds.dump_json(s))?;
    field_real(stream, "myInitAng", fields.initial_angle)?;
    field_integer(stream, "myCompBB", i32::from(fields.recompute_bounds))
}

/// `AIS_Shape::SelectionMode`: the selection mode of a shape type (0 for SHAPE).
/// Example: `ais_shape_selection_mode(ShapeEnum::Face) == 4`.
pub fn ais_shape_selection_mode(kind: ShapeEnum) -> i32 {
    match kind {
        ShapeEnum::Vertex => 1,
        ShapeEnum::Edge => 2,
        ShapeEnum::Wire => 3,
        ShapeEnum::Face => 4,
        ShapeEnum::Shell => 5,
        ShapeEnum::Solid => 6,
        ShapeEnum::CompSolid => 7,
        ShapeEnum::Compound => 8,
        ShapeEnum::Shape => 0,
    }
}

/// The non-virtual `AIS_Shape` members over any shape object.
pub trait AisShapeParameters: AisShapeSource {
    /// `Shape`: a copy of the shape; `object.shape()`.
    fn shape(&self) -> Shape {
        self.shape_base().fields.read().shape.clone()
    }
    /// `Set`: the same as `SetShape`; `object.set(&shape)`.
    fn set(&self, shape: &Shape) {
        AisShapeMembers::set_shape(self, shape);
    }
    /// `BoundingBox`: the box of the shape, recomputed after a shape change (void for an
    /// empty compound); named apart from the presentable bounding box of the presentations.
    /// Example: `object.shape_bounding_box()?`.
    fn shape_bounding_box(&self) -> Result<BoundingBox3, Failure> {
        shape_presentation_bounds_snapshot(self)
    }
    /// `SetTypeOfHLR`; `object.set_type_of_hlr(Prs3dTypeOfHlr::Algo)`.
    fn set_type_of_hlr(&self, kind: Prs3dTypeOfHlr) {
        self.attributes().set_type_of_hlr(kind);
    }
    /// `TypeOfHLR`; `object.type_of_hlr()`.
    fn type_of_hlr(&self) -> Prs3dTypeOfHlr {
        self.attributes().type_of_hlr()
    }
    /// `SetOwnDeviationCoefficient()`: resets an own coefficient, true when there was one.
    /// Example: `object.reset_own_deviation_coefficient()`.
    fn reset_own_deviation_coefficient(&self) -> bool {
        let drawer = self.attributes();
        let was_set = drawer.has_own_deviation_coefficient();
        if was_set {
            drawer.unset_deviation_coefficient();
        }
        was_set
    }
    /// `SetOwnDeviationAngle()`: resets an own angle, true when there was one.
    /// Example: `object.reset_own_deviation_angle()`.
    fn reset_own_deviation_angle(&self) -> bool {
        let drawer = self.attributes();
        let was_set = drawer.has_own_deviation_angle();
        if was_set {
            drawer.unset_deviation_angle();
        }
        was_set
    }
    /// `SetOwnDeviationCoefficient(coefficient)`: every presentation must be recomputed.
    /// Example: `object.set_own_deviation_coefficient(0.002)`.
    fn set_own_deviation_coefficient(&self, coefficient: f64) {
        self.attributes().set_deviation_coefficient(coefficient);
        self.set_to_update(-1);
    }
    /// `SetOwnDeviationAngle(angle)`: the wireframe must be recomputed.
    /// Example: `object.set_own_deviation_angle(0.3)`.
    fn set_own_deviation_angle(&self, angle: f64) {
        self.attributes().set_deviation_angle(angle);
        self.set_to_update(0);
    }
    /// `SetAngleAndDeviation`: the angle and the HLR deflection derived from it.
    /// Example: `object.set_angle_and_deviation(0.2)`.
    fn set_angle_and_deviation(&self, angle: f64) {
        let (_, deflection) = rustycad_tkhlr::hlr_brep::poly_hlr_angle_and_deflection(angle);
        self.set_own_deviation_angle(angle);
        self.set_own_deviation_coefficient(deflection);
        self.shape_base().fields.write().initial_angle = angle;
        self.set_to_update(-1);
    }
    /// `UserAngle`: the initial angle, or the context deviation angle when it is 0; a missing
    /// context is a `Standard_NullObject` failure (a null dereference natively).
    /// Example: `object.user_angle()?`.
    fn user_angle(&self) -> Result<f64, Failure> {
        let angle = self.shape_base().fields.read().initial_angle;
        if angle != 0. {
            return Ok(angle);
        }
        let context = self.get_context();
        let context = context.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_Shape::UserAngle: no context")
        })?;
        Ok(context.default_drawer().deviation_angle())
    }
    /// `OwnDeviationCoefficient`: whether the coefficient is own, with the coefficient and the
    /// previous one. Example: `let (own, coefficient, previous) = object.own_deviation_coefficient();`.
    fn own_deviation_coefficient(&self) -> (bool, f64, f64) {
        let drawer = self.attributes();
        (
            drawer.has_own_deviation_coefficient(),
            drawer.deviation_coefficient(),
            drawer.previous_deviation_coefficient(),
        )
    }
    /// `OwnDeviationAngle`: whether the angle is own, with the angle and the previous one.
    /// Example: `let (own, angle, previous) = object.own_deviation_angle();`.
    fn own_deviation_angle(&self) -> (bool, f64, f64) {
        let drawer = self.attributes();
        (
            drawer.has_own_deviation_angle(),
            drawer.deviation_angle(),
            drawer.previous_deviation_angle(),
        )
    }
    /// `TextureRepeatUV`; `object.texture_repeat_uv()`.
    fn texture_repeat_uv(&self) -> Point2 {
        self.shape_base().fields.read().uv_repeat
    }
    /// `SetTextureRepeatUV`; `object.set_texture_repeat_uv(Point2::new(2., 2.))`.
    fn set_texture_repeat_uv(&self, repeat: Point2) {
        self.shape_base().fields.write().uv_repeat = repeat;
    }
    /// `TextureOriginUV`; `object.texture_origin_uv()`.
    fn texture_origin_uv(&self) -> Point2 {
        self.shape_base().fields.read().uv_origin
    }
    /// `SetTextureOriginUV`; `object.set_texture_origin_uv(Point2::new(0.5, 0.))`.
    fn set_texture_origin_uv(&self, origin: Point2) {
        self.shape_base().fields.write().uv_origin = origin;
    }
    /// `TextureScaleUV`; `object.texture_scale_uv()`.
    fn texture_scale_uv(&self) -> Point2 {
        self.shape_base().fields.read().uv_scale
    }
    /// `SetTextureScaleUV`; `object.set_texture_scale_uv(Point2::new(3., 3.))`.
    fn set_texture_scale_uv(&self, scale: Point2) {
        self.shape_base().fields.write().uv_scale = scale;
    }
}
impl<S: AisShapeSource + ?Sized> AisShapeParameters for S {}
