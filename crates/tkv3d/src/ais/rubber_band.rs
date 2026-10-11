//! Martensite screen-space rubber bands, translated from OCCT 3d097a0 AIS_RubberBand.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::{prs_mgr::*, prs3d::*};
use rustycad_tkernel::{
    ncollection::{Sequence, Vec2},
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
/// Native protected storage for subclasses; `base.fields.points.write().append(Vec2::new(1, 2))`.
pub struct AisRubberBandFields {
    /// Stable sequence identity; release a read guard before editing points.
    pub points: SharedState<Sequence<Vec2<i32>>>,
    /// Cached filling array, retained when triangulation fails.
    pub triangles: SharedState<Handle<Graphic3dArrayOfTriangles>>,
    /// Cached boundary array, reused when its vertex count matches.
    pub borders: SharedState<Handle<Graphic3dArrayOfPolylines>>,
    /// Automatic boundary closure.
    pub is_polygon_closed: SharedState<bool>,
}
/// Embeddable interactive and protected state; `AisRubberBandBase::new(white, solid, 1., true)?`.
pub struct AisRubberBandBase {
    /// Inherited interactive state.
    pub interactive: AisInteractiveObjectBase,
    /// Native protected state.
    pub fields: AisRubberBandFields,
}
impl AisRubberBandBase {
    /// Initialize an unfilled native line configuration; `AisRubberBandBase::new(white, solid, 1., true)?`.
    pub fn new(
        color: Color,
        kind: AspectTypeOfLine,
        width: f64,
        closed: bool,
    ) -> Result<Self, Failure> {
        let interactive = AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView);
        let drawer = interactive.presentable.fields.read().drawer.clone();
        drawer.set_line_aspect(Handle::new(Prs3dLineAspect::new(color, kind, width)?));
        drawer.set_shading_aspect(Handle::new(rubber_band_shading()?));
        Ok(Self {
            interactive,
            fields: AisRubberBandFields {
                points: SharedState::default(),
                triangles: SharedState::default(),
                borders: SharedState::default(),
                is_polygon_closed: SharedState::new(closed),
            },
        })
    }
}
fn rubber_band_shading() -> Result<Prs3dShadingAspect, Failure> {
    let shading = Prs3dShadingAspect::new()?;
    shading.set_material(
        &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified),
        AspectTypeOfFacingModel::BothSide,
    );
    let fill = shading.aspect();
    fill.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    fill.set_interior_style(AspectInteriorStyle::Empty);
    fill.set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
    shading.set_transparency(1., AspectTypeOfFacingModel::BothSide)?;
    shading.set_color(
        Color::named(NamedColor::WHITE),
        AspectTypeOfFacingModel::BothSide,
    )?;
    Ok(shading)
}
/// Native extension contract for rubber-band subclasses; `band.set_rectangle(1, 2, 10, 20)`.
pub trait AisRubberBandSource: AisInteractiveObject {
    /// Embedded protected state; `band.rubber_band_base()`.
    fn rubber_band_base(&self) -> &AisRubberBandBase;
    /// Borrow the live sequence, rather than a snapshot; `band.points().read().value(1)?`.
    fn points(&self) -> &SharedState<Sequence<Vec2<i32>>> {
        &self.rubber_band_base().fields.points
    }
    /// Replace points in native order without normalizing bounds; `band.set_rectangle(10, 20, 1, 2)`.
    fn set_rectangle(&self, min_x: i32, min_y: i32, max_x: i32, max_y: i32) {
        let mut points = self.points().write();
        points.clear(None);
        for (x, y) in [
            (min_x, min_y),
            (min_x, max_y),
            (max_x, max_y),
            (max_x, min_y),
        ] {
            points.append(Vec2::new(x, y));
        }
    }
    /// Append without scheduling redisplay; `band.add_point(Vec2::new(10, 20))`.
    fn add_point(&self, point: Vec2<i32>) {
        self.points().write().append(point);
    }
    /// Remove the last point, raising on an empty sequence; `band.remove_last_point()?`.
    fn remove_last_point(&self) -> Result<(), Failure> {
        let mut points = self.points().write();
        let last = points.len();
        points.remove(last)
    }
    /// Clear points while retaining caches; `band.clear_points()`.
    fn clear_points(&self) {
        self.points().write().clear(None);
    }
    /// Native line color; `band.line_color()`.
    fn line_color(&self) -> Color {
        self.attributes().line_aspect().aspect().interior_color()
    }
    /// Assign the line aspect without changing AIS color ownership; `band.set_line_color(red)`.
    fn set_line_color(&self, color: Color) {
        self.attributes().line_aspect().set_color(color);
    }
    /// Native filling color; `band.fill_color()`.
    fn fill_color(&self) -> Color {
        self.attributes()
            .shading_aspect()
            .color(AspectTypeOfFacingModel::BothSide)
    }
    /// Assign both material sides; `band.set_fill_color(red)?`.
    fn set_fill_color(&self, color: Color) -> Result<(), Failure> {
        self.attributes()
            .shading_aspect()
            .set_color(color, AspectTypeOfFacingModel::BothSide)
    }
    /// Assign a positive width with native float narrowing; `band.set_line_width(2.)?`.
    fn set_line_width(&self, width: f64) -> Result<(), Failure> {
        self.attributes().line_aspect().set_width(width)
    }
    /// Native line width; `band.line_width()`.
    fn line_width(&self) -> f64 {
        f64::from(self.attributes().line_aspect().aspect().line_width())
    }
    /// Assign the boundary style; `band.set_line_type(AspectTypeOfLine::Dash)`.
    fn set_line_type(&self, kind: AspectTypeOfLine) {
        self.attributes().line_aspect().set_type_of_line(kind);
    }
    /// Native boundary style; `band.line_type()`.
    fn line_type(&self) -> AspectTypeOfLine {
        self.attributes().line_aspect().aspect().line_kind()
    }
    /// Assign transparency in `[0, 1]`; `band.set_fill_transparency(0.25)?`.
    fn set_fill_transparency(&self, transparency: f64) -> Result<(), Failure> {
        self.attributes()
            .shading_aspect()
            .set_transparency(transparency, AspectTypeOfFacingModel::BothSide)
    }
    /// Native float-narrowed transparency; `band.fill_transparency()`.
    fn fill_transparency(&self) -> f64 {
        self.attributes()
            .shading_aspect()
            .transparency(AspectTypeOfFacingModel::BothSide)
    }
    /// Enable or disable filling without clearing cached triangles; `band.set_filling(true)`.
    fn set_filling(&self, enabled: bool) {
        let style = if enabled {
            AspectInteriorStyle::Solid
        } else {
            AspectInteriorStyle::Empty
        };
        self.attributes()
            .shading_aspect()
            .aspect()
            .set_interior_style(style);
    }
    /// Enable, then assign transparency and color in native exception order; `band.set_filling_color(red, 0.25)?`.
    fn set_filling_color(&self, color: Color, transparency: f64) -> Result<(), Failure> {
        self.set_filling(true);
        self.set_fill_transparency(transparency)?;
        self.set_fill_color(color)
    }
    /// Every nonempty interior style counts as filling; `band.is_filling()`.
    fn is_filling(&self) -> bool {
        self.attributes().shading_aspect().aspect().interior_style() != AspectInteriorStyle::Empty
    }
    /// Native closure flag; `band.is_polygon_closed()`.
    fn is_polygon_closed(&self) -> bool {
        *self.rubber_band_base().fields.is_polygon_closed.read()
    }
    /// Assign closure without scheduling redisplay; `band.set_polygon_closed(false)`.
    fn set_polygon_closed(&self, closed: bool) {
        *self.rubber_band_base().fields.is_polygon_closed.write() = closed;
    }
    /// Native protected triangulation hook; `band.fill_triangles()?`.
    fn fill_triangles(&self) -> Result<bool, Failure> {
        super::ais_rubber_band_fill_triangles(self)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisRubberBandSource,
    "AIS_RubberBand",
    dyn AisInteractiveObject
);
/// Rectangle/polygon selection overlay; `AisRubberBand::new()?`.
pub struct AisRubberBand {
    transient: StandardTransient,
    base: AisRubberBandBase,
}
rustycad_tkernel::impl_transient!(AisRubberBand,"AIS_RubberBand",dyn AisInteractiveObject,transient,interfaces:[dyn AisRubberBandSource,dyn AisInteractiveObject,dyn crate::select_mgr::SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisRubberBand {
    /// Default white solid closed boundary; `AisRubberBand::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        Self::with_line(
            Color::named(NamedColor::WHITE),
            AspectTypeOfLine::Solid,
            1.,
            true,
        )
    }
    /// Native line-only constructor; `AisRubberBand::with_line(red, AspectTypeOfLine::Dash, 2., false)?`.
    pub fn with_line(
        color: Color,
        kind: AspectTypeOfLine,
        width: f64,
        closed: bool,
    ) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisRubberBandBase::new(color, kind, width, closed)?,
        });
        prs_mgr_presentable_bind(&object.cast());
        ais_rubber_band_initialize_overlay(&*object)?;
        Ok(object)
    }
    /// Native filled constructor; `AisRubberBand::with_filling(white, solid, red, 0.5, 1., true)?`.
    pub fn with_filling(
        line: Color,
        kind: AspectTypeOfLine,
        fill: Color,
        transparency: f64,
        width: f64,
        closed: bool,
    ) -> Result<Handle<Self>, Failure> {
        let object = Self::with_line(line, kind, width, closed)?;
        // OCCT constructor assigns color before transparency; SetFilling overload does the reverse.
        object.set_fill_color(fill)?;
        object.set_filling(true);
        object.set_fill_transparency(transparency)?;
        Ok(object)
    }
}
/// Bind screen persistence for an application subclass after binding its handle; `ais_rubber_band_initialize_overlay(&*band)?`.
pub fn ais_rubber_band_initialize_overlay<O: AisRubberBandSource + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let persistence = Handle::new(Graphic3dTransformPers::with_corner(
        Graphic3dTransModeFlags::TMF_2D,
        AspectTypeOfTriedronPosition::LEFT_LOWER,
        Vec2::new(0, 0),
    )?);
    object.set_transform_persistence(&persistence.cast());
    object.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP_OSD);
    Ok(())
}
impl AisRubberBandSource for AisRubberBand {
    fn rubber_band_base(&self) -> &AisRubberBandBase {
        &self.base
    }
}
impl AisInteractiveObject for AisRubberBand {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
}
impl Drop for AisRubberBand {
    fn drop(&mut self) {
        // OCCT releases protected screen geometry before destruction of the interactive base.
        self.base.fields.points.write().clear(None);
        *self.base.fields.triangles.write() = Handle::default();
        *self.base.fields.borders.write() = Handle::default();
    }
}
