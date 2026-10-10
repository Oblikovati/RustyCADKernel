//! `Prs3d_Drawer`: the presentation attributes of an interactive object, each one either own or
//! inherited from the link drawer.
//!
//! The class derives from `Graphic3d_PresentationAttributes` (its base fields are the embedded
//! [`Graphic3dPresentationAttributesBase`]); the drawer's own fields live in one
//! [`SharedState`]. A getter that falls back to the link reads its own value, releases the lock
//! and then asks the link, so a drawer linked to itself cannot deadlock (natively it recurses
//! forever). The members are split by topic: drawer_values.rs (deflection, isolines, flags,
//! units), drawer_aspects.rs (the aspect handles), drawer_setup.rs (own defaults, shader
//! program, shading model) and drawer_dump.rs.
use super::{
    Prs3dArrowAspect, Prs3dDatumAspect, Prs3dDimensionAspect, Prs3dDimensionUnits, Prs3dIsoAspect,
    Prs3dLineAspect, Prs3dPlaneAspect, Prs3dPointAspect, Prs3dShadingAspect, Prs3dTextAspect,
    Prs3dTypeOfHlr, Prs3dVertexDrawMode,
};
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use rustycad_tkservice::graphic3d::{
    Graphic3dPresentationAttributes, Graphic3dPresentationAttributesBase,
    Graphic3dPresentationAttributesSource,
};

/// The protected fields of `Prs3d_Drawer`.
#[derive(Clone)]
pub struct Prs3dDrawerFields {
    /// `myLink`.
    pub link: Handle<Prs3dDrawer>,
    /// `myNbPoints` (-1 when not own).
    pub nb_points: i32,
    /// `myMaximalParameterValue` (<= 0 when not own).
    pub maximal_parameter_value: f64,
    /// `myChordialDeviation` (<= 0 when not own).
    pub chordial_deviation: f64,
    /// `myTypeOfDeflection`.
    pub type_of_deflection: AspectTypeOfDeflection,
    /// `myHasOwnTypeOfDeflection`.
    pub has_own_type_of_deflection: bool,
    /// `myTypeOfHLR`.
    pub type_of_hlr: Prs3dTypeOfHlr,
    /// `myDeviationCoefficient` (<= 0 when not own).
    pub deviation_coefficient: f64,
    /// `myPreviousDeviationCoefficient`; uninitialized natively until the first set, 0 here.
    pub previous_deviation_coefficient: f64,
    /// `myDeviationAngle` (<= 0 when not own).
    pub deviation_angle: f64,
    /// `myPreviousDeviationAngle`; uninitialized natively until the first set, 0 here.
    pub previous_deviation_angle: f64,
    /// `myIsoOnPlane`.
    pub iso_on_plane: bool,
    /// `myHasOwnIsoOnPlane`.
    pub has_own_iso_on_plane: bool,
    /// `myIsoOnTriangulation`.
    pub iso_on_triangulation: bool,
    /// `myHasOwnIsoOnTriangulation`.
    pub has_own_iso_on_triangulation: bool,
    /// `myIsAutoTriangulated`.
    pub is_auto_triangulated: bool,
    /// `myHasOwnIsAutoTriangulated`.
    pub has_own_is_auto_triangulated: bool,
    /// `myUIsoAspect`.
    pub u_iso_aspect: Handle<Prs3dIsoAspect>,
    /// `myVIsoAspect`.
    pub v_iso_aspect: Handle<Prs3dIsoAspect>,
    /// `myWireAspect`.
    pub wire_aspect: Handle<Prs3dLineAspect>,
    /// `myWireDraw`.
    pub wire_draw: bool,
    /// `myHasOwnWireDraw`.
    pub has_own_wire_draw: bool,
    /// `myPointAspect`.
    pub point_aspect: Handle<Prs3dPointAspect>,
    /// `myLineAspect`.
    pub line_aspect: Handle<Prs3dLineAspect>,
    /// `myTextAspect`.
    pub text_aspect: Handle<Prs3dTextAspect>,
    /// `myShadingAspect`.
    pub shading_aspect: Handle<Prs3dShadingAspect>,
    /// `myPlaneAspect`.
    pub plane_aspect: Handle<Prs3dPlaneAspect>,
    /// `mySeenLineAspect`.
    pub seen_line_aspect: Handle<Prs3dLineAspect>,
    /// `myArrowAspect`.
    pub arrow_aspect: Handle<Prs3dArrowAspect>,
    /// `myLineArrowDraw`.
    pub line_arrow_draw: bool,
    /// `myHasOwnLineArrowDraw`.
    pub has_own_line_arrow_draw: bool,
    /// `myHiddenLineAspect`.
    pub hidden_line_aspect: Handle<Prs3dLineAspect>,
    /// `myDrawHiddenLine`.
    pub draw_hidden_line: bool,
    /// `myHasOwnDrawHiddenLine`.
    pub has_own_draw_hidden_line: bool,
    /// `myVectorAspect`.
    pub vector_aspect: Handle<Prs3dLineAspect>,
    /// `myVertexDrawMode`.
    pub vertex_draw_mode: Prs3dVertexDrawMode,
    /// `myDatumAspect`.
    pub datum_aspect: Handle<Prs3dDatumAspect>,
    /// `mySectionAspect`.
    pub section_aspect: Handle<Prs3dLineAspect>,
    /// `myFreeBoundaryAspect`.
    pub free_boundary_aspect: Handle<Prs3dLineAspect>,
    /// `myFreeBoundaryDraw`.
    pub free_boundary_draw: bool,
    /// `myHasOwnFreeBoundaryDraw`.
    pub has_own_free_boundary_draw: bool,
    /// `myUnFreeBoundaryAspect`.
    pub un_free_boundary_aspect: Handle<Prs3dLineAspect>,
    /// `myUnFreeBoundaryDraw`.
    pub un_free_boundary_draw: bool,
    /// `myHasOwnUnFreeBoundaryDraw`.
    pub has_own_un_free_boundary_draw: bool,
    /// `myFaceBoundaryAspect`.
    pub face_boundary_aspect: Handle<Prs3dLineAspect>,
    /// `myFaceBoundaryUpperContinuity` (natively -1 for `None`).
    pub face_boundary_upper_continuity: Option<Shape>,
    /// `myFaceBoundaryDraw`.
    pub face_boundary_draw: bool,
    /// `myHasOwnFaceBoundaryDraw`.
    pub has_own_face_boundary_draw: bool,
    /// `myDimensionAspect`.
    pub dimension_aspect: Handle<Prs3dDimensionAspect>,
    /// `myDimensionModelUnits`.
    pub dimension_model_units: Prs3dDimensionUnits,
    /// `myHasOwnDimLengthModelUnits`.
    pub has_own_dim_length_model_units: bool,
    /// `myHasOwnDimAngleModelUnits`.
    pub has_own_dim_angle_model_units: bool,
    /// `myDimensionDisplayUnits`.
    pub dimension_display_units: Prs3dDimensionUnits,
    /// `myHasOwnDimLengthDisplayUnits`.
    pub has_own_dim_length_display_units: bool,
    /// `myHasOwnDimAngleDisplayUnits`.
    pub has_own_dim_angle_display_units: bool,
}

impl Default for Prs3dDrawerFields {
    /// The native constructor: everything inherited, display angles in degrees.
    fn default() -> Self {
        let mut display_units = Prs3dDimensionUnits::default();
        display_units.set_angle_units(&AsciiString::from("deg"));
        Self {
            link: Handle::default(),
            nb_points: -1,
            maximal_parameter_value: -1.0,
            chordial_deviation: -1.0,
            type_of_deflection: AspectTypeOfDeflection::Relative,
            has_own_type_of_deflection: false,
            type_of_hlr: Prs3dTypeOfHlr::NotSet,
            deviation_coefficient: -1.0,
            previous_deviation_coefficient: 0.0,
            deviation_angle: -1.0,
            previous_deviation_angle: 0.0,
            iso_on_plane: false,
            has_own_iso_on_plane: false,
            iso_on_triangulation: false,
            has_own_iso_on_triangulation: false,
            is_auto_triangulated: true,
            has_own_is_auto_triangulated: false,
            u_iso_aspect: Handle::default(),
            v_iso_aspect: Handle::default(),
            wire_aspect: Handle::default(),
            wire_draw: true,
            has_own_wire_draw: false,
            point_aspect: Handle::default(),
            line_aspect: Handle::default(),
            text_aspect: Handle::default(),
            shading_aspect: Handle::default(),
            plane_aspect: Handle::default(),
            seen_line_aspect: Handle::default(),
            arrow_aspect: Handle::default(),
            line_arrow_draw: false,
            has_own_line_arrow_draw: false,
            hidden_line_aspect: Handle::default(),
            draw_hidden_line: false,
            has_own_draw_hidden_line: false,
            vector_aspect: Handle::default(),
            vertex_draw_mode: Prs3dVertexDrawMode::Inherited,
            datum_aspect: Handle::default(),
            section_aspect: Handle::default(),
            free_boundary_aspect: Handle::default(),
            free_boundary_draw: true,
            has_own_free_boundary_draw: false,
            un_free_boundary_aspect: Handle::default(),
            un_free_boundary_draw: true,
            has_own_un_free_boundary_draw: false,
            face_boundary_aspect: Handle::default(),
            face_boundary_upper_continuity: None,
            face_boundary_draw: false,
            has_own_face_boundary_draw: false,
            dimension_aspect: Handle::default(),
            dimension_model_units: Prs3dDimensionUnits::default(),
            has_own_dim_length_model_units: false,
            has_own_dim_angle_model_units: false,
            dimension_display_units: display_units,
            has_own_dim_length_display_units: false,
            has_own_dim_angle_display_units: false,
        }
    }
}

/// The attributes of a presentation, own or inherited from a link drawer.
///
/// Example: `let drawer = Prs3dDrawer::default(); drawer.set_discretisation(50); assert_eq!(drawer.discretisation(), 50);`.
#[derive(Default)]
pub struct Prs3dDrawer {
    transient: StandardTransient,
    attributes: Graphic3dPresentationAttributesBase,
    pub(crate) fields: SharedState<Prs3dDrawerFields>,
}
rustycad_tkernel::impl_transient!(Prs3dDrawer, "Prs3d_Drawer", Graphic3dPresentationAttributes, transient,
    interfaces: [dyn Graphic3dPresentationAttributesSource]);

impl Graphic3dPresentationAttributesSource for Prs3dDrawer {
    fn presentation_attributes_base(&self) -> &Graphic3dPresentationAttributesBase {
        &self.attributes
    }
    fn dump_json(
        &self,
        stream: &mut rustycad_tkernel::standard::dump::DumpStream,
        depth: i32,
    ) -> std::io::Result<()> {
        super::drawer_dump::drawer_dump_json(self, stream, depth)
    }
}

impl Prs3dDrawer {
    /// A new drawer that inherits everything; `Prs3dDrawer::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// A copy of the fields; `drawer.fields_snapshot().nb_points`.
    pub fn fields_snapshot(&self) -> Prs3dDrawerFields {
        self.fields.snapshot()
    }
    /// The own value when `own` gives one, else the link's (`from_link`), else `fallback`.
    pub(crate) fn own_or_link<T>(
        &self,
        own: impl FnOnce(&Prs3dDrawerFields) -> Option<T>,
        from_link: impl FnOnce(&Prs3dDrawer) -> T,
        fallback: impl FnOnce(&Prs3dDrawerFields) -> T,
    ) -> T {
        let link = {
            let fields = self.fields.read();
            if let Some(value) = own(&fields) {
                return value;
            }
            if fields.link.is_null() {
                return fallback(&fields);
            }
            fields.link.clone()
        };
        link.get()
            .map_or_else(|| fallback(&self.fields.read()), from_link)
    }
    /// The aspect handle when own or without link, else the link's.
    pub(crate) fn aspect_or_link<T: rustycad_tkernel::standard::RuntimeTyped>(
        &self,
        own: impl FnOnce(&Prs3dDrawerFields) -> Handle<T>,
        from_link: impl FnOnce(&Prs3dDrawer) -> Handle<T>,
    ) -> Handle<T> {
        let (value, link) = {
            let fields = self.fields.read();
            (own(&fields), fields.link.clone())
        };
        match link.get() {
            Some(linked) if value.is_null() => from_link(linked),
            _ => value,
        }
    }
    /// Applies a change to the fields; `drawer.edit(|f| f.nb_points = 5)`.
    pub(crate) fn edit(&self, change: impl FnOnce(&mut Prs3dDrawerFields)) {
        change(&mut self.fields.write());
    }
    /// The link drawer; `drawer.link()`.
    pub fn link(&self) -> Handle<Prs3dDrawer> {
        self.fields.read().link.clone()
    }
    /// Whether a link drawer is set; `drawer.has_link()`.
    pub fn has_link(&self) -> bool {
        !self.fields.read().link.is_null()
    }
    /// `Link(theDrawer)`, the same as `SetLink`; `drawer.link_to(&defaults)`.
    pub fn link_to(&self, drawer: &Handle<Prs3dDrawer>) {
        self.set_link(drawer);
    }
    /// Sets the link drawer; `drawer.set_link(&defaults)`.
    pub fn set_link(&self, drawer: &Handle<Prs3dDrawer>) {
        self.fields.write().link = drawer.clone();
    }
    /// Whether `other` is this very drawer (the native `theDefaults != this`).
    pub(crate) fn is_self(&self, other: &Handle<Prs3dDrawer>) -> bool {
        std::ptr::eq(other.object_address(), (self as *const Self).cast())
    }
}
