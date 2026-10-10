//! The value members of `Prs3d_Drawer`: deflection, discretisation, isolines, the drawing
//! flags, the vertex mode, the face boundary continuity and the dimension units.
use super::{Prs3dDrawer, Prs3dTypeOfHlr, Prs3dVertexDrawMode};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use std::f64::consts::PI;

/// Declares the getter, setter, `HasOwn` and `UnsetOwn` members of a boolean that is own when
/// its `has_own` flag is set or when there is no link; unsetting restores `$reset`.
macro_rules! own_flag_members {
    ($get:ident, $set:ident, $has:ident, $unset:ident, $field:ident, $own:ident, $reset:expr, $doc:literal) => {
        #[doc = concat!("`", $doc, "`, own or from the link; `drawer.", stringify!($get), "()`.")]
        pub fn $get(&self) -> bool {
            self.own_or_link(
                |f| f.$own.then_some(f.$field),
                Prs3dDrawer::$get,
                |f| f.$field,
            )
        }
        #[doc = concat!("Sets an own `", $doc, "`; `drawer.", stringify!($set), "(true)`.")]
        pub fn $set(&self, is_enabled: bool) {
            self.edit(|f| (f.$own, f.$field) = (true, is_enabled));
        }
        #[doc = concat!("Whether `", $doc, "` is own; `drawer.", stringify!($has), "()`.")]
        pub fn $has(&self) -> bool {
            self.fields.read().$own
        }
        #[doc = concat!("Drops the own `", $doc, "`; `drawer.", stringify!($unset), "()`.")]
        pub fn $unset(&self) {
            self.edit(|f| (f.$own, f.$field) = (false, $reset));
        }
    };
}

/// Declares the getter, setter, `HasOwn` and `UnsetOwn` members of a positive real that is own
/// when greater than 0, else the link's or `$default`.
macro_rules! own_positive_members {
    ($get:ident, $set:ident, $has:ident, $unset:ident, $field:ident, $default:expr, $doc:literal) => {
        #[doc = concat!("`", $doc, "`, own when > 0, else the link's or ", stringify!($default), "; `drawer.", stringify!($get), "()`.")]
        pub fn $get(&self) -> f64 {
            self.own_or_link(|f| (f.$field > 0.0).then_some(f.$field), Prs3dDrawer::$get, |_| $default)
        }
        #[doc = concat!("Sets `", $doc, "`; `drawer.", stringify!($set), "(0.5)`.")]
        pub fn $set(&self, value: f64) {
            self.edit(|f| f.$field = value);
        }
        #[doc = concat!("Whether `", $doc, "` is own; `drawer.", stringify!($has), "()`.")]
        pub fn $has(&self) -> bool {
            self.fields.read().$field > 0.0
        }
        #[doc = concat!("Drops the own `", $doc, "`; `drawer.", stringify!($unset), "()`.")]
        pub fn $unset(&self) {
            self.edit(|f| f.$field = -1.0);
        }
    };
}

/// Deflection, discretisation and HLR.
impl Prs3dDrawer {
    /// Sets an own deflection type; `drawer.set_type_of_deflection(AspectTypeOfDeflection::Absolute)`.
    pub fn set_type_of_deflection(&self, kind: AspectTypeOfDeflection) {
        self.edit(|f| (f.has_own_type_of_deflection, f.type_of_deflection) = (true, kind));
    }
    /// The deflection type, own or from the link; `drawer.type_of_deflection()`.
    pub fn type_of_deflection(&self) -> AspectTypeOfDeflection {
        self.own_or_link(
            |f| f.has_own_type_of_deflection.then_some(f.type_of_deflection),
            Prs3dDrawer::type_of_deflection,
            |f| f.type_of_deflection,
        )
    }
    /// Whether the deflection type is own; `drawer.has_own_type_of_deflection()`.
    pub fn has_own_type_of_deflection(&self) -> bool {
        self.fields.read().has_own_type_of_deflection
    }
    /// Drops the own deflection type (back to relative); `drawer.unset_own_type_of_deflection()`.
    pub fn unset_own_type_of_deflection(&self) {
        self.edit(|f| {
            (f.has_own_type_of_deflection, f.type_of_deflection) =
                (false, AspectTypeOfDeflection::Relative)
        });
    }
    own_positive_members!(
        maximal_chordial_deviation,
        set_maximal_chordial_deviation,
        has_own_maximal_chordial_deviation,
        unset_own_maximal_chordial_deviation,
        chordial_deviation,
        0.0001,
        "MaximalChordialDeviation"
    );
    /// Sets the HLR algorithm (`NotSet` means inherited); `drawer.set_type_of_hlr(Prs3dTypeOfHlr::Algo)`.
    pub fn set_type_of_hlr(&self, kind: Prs3dTypeOfHlr) {
        self.edit(|f| f.type_of_hlr = kind);
    }
    /// The HLR algorithm, own, from the link, else `PolyAlgo`; `drawer.type_of_hlr()`.
    pub fn type_of_hlr(&self) -> Prs3dTypeOfHlr {
        self.own_or_link(
            |f| (f.type_of_hlr != Prs3dTypeOfHlr::NotSet).then_some(f.type_of_hlr),
            Prs3dDrawer::type_of_hlr,
            |_| Prs3dTypeOfHlr::PolyAlgo,
        )
    }
    /// Whether the HLR algorithm is own; `drawer.has_own_type_of_hlr()`.
    pub fn has_own_type_of_hlr(&self) -> bool {
        self.fields.read().type_of_hlr != Prs3dTypeOfHlr::NotSet
    }
    own_positive_members!(
        maximal_parameter_value,
        set_maximal_parameter_value,
        has_own_maximal_parameter_value,
        unset_own_maximal_parameter_value,
        maximal_parameter_value,
        500000.0,
        "MaximalParameterValue"
    );
    /// Sets the number of curve points (-1 means inherited); `drawer.set_discretisation(50)`.
    pub fn set_discretisation(&self, value: i32) {
        self.edit(|f| f.nb_points = value);
    }
    /// The number of curve points, own, from the link, else 30; `drawer.discretisation()`.
    pub fn discretisation(&self) -> i32 {
        self.own_or_link(
            |f| (f.nb_points != -1).then_some(f.nb_points),
            Prs3dDrawer::discretisation,
            |_| 30,
        )
    }
    /// Whether the discretisation is own; `drawer.has_own_discretisation()`.
    pub fn has_own_discretisation(&self) -> bool {
        self.fields.read().nb_points != -1
    }
    /// Drops the own discretisation; `drawer.unset_own_discretisation()`.
    pub fn unset_own_discretisation(&self) {
        self.edit(|f| f.nb_points = -1);
    }
}

/// Deviation coefficient and angle with their previous values.
impl Prs3dDrawer {
    /// Sets the deviation coefficient, keeping the current one as previous.
    /// Example: `drawer.set_deviation_coefficient(0.01)`.
    pub fn set_deviation_coefficient(&self, coefficient: f64) {
        let previous = self.deviation_coefficient();
        self.edit(|f| {
            (f.previous_deviation_coefficient, f.deviation_coefficient) = (previous, coefficient)
        });
    }
    /// The deviation coefficient, own when > 0, else the link's or 0.001; `drawer.deviation_coefficient()`.
    pub fn deviation_coefficient(&self) -> f64 {
        self.own_or_link(
            |f| (f.deviation_coefficient > 0.0).then_some(f.deviation_coefficient),
            Prs3dDrawer::deviation_coefficient,
            |_| 0.001,
        )
    }
    /// `SetDeviationCoefficient()`: drops the own coefficient; `drawer.unset_deviation_coefficient()`.
    pub fn unset_deviation_coefficient(&self) {
        self.edit(|f| f.deviation_coefficient = -1.0);
    }
    /// Whether the coefficient is own; `drawer.has_own_deviation_coefficient()`.
    pub fn has_own_deviation_coefficient(&self) -> bool {
        self.fields.read().deviation_coefficient > 0.0
    }
    /// The previous own coefficient, 0 when not own; `drawer.previous_deviation_coefficient()`.
    pub fn previous_deviation_coefficient(&self) -> f64 {
        let f = self.fields.read();
        if f.deviation_coefficient > 0.0 {
            f.previous_deviation_coefficient
        } else {
            0.0
        }
    }
    /// Stores the own coefficient as previous; `drawer.update_previous_deviation_coefficient()`.
    pub fn update_previous_deviation_coefficient(&self) {
        if self.has_own_deviation_coefficient() {
            let current = self.deviation_coefficient();
            self.edit(|f| f.previous_deviation_coefficient = current);
        }
    }
    /// Sets the deviation angle, keeping the current one as previous; `drawer.set_deviation_angle(0.2)`.
    pub fn set_deviation_angle(&self, angle: f64) {
        let previous = self.deviation_angle();
        self.edit(|f| (f.previous_deviation_angle, f.deviation_angle) = (previous, angle));
    }
    /// The deviation angle, own when > 0, else the link's or 20 degrees; `drawer.deviation_angle()`.
    pub fn deviation_angle(&self) -> f64 {
        self.own_or_link(
            |f| (f.deviation_angle > 0.0).then_some(f.deviation_angle),
            Prs3dDrawer::deviation_angle,
            |_| 20.0 * PI / 180.0,
        )
    }
    /// `SetDeviationAngle()`: drops the own angle; `drawer.unset_deviation_angle()`.
    pub fn unset_deviation_angle(&self) {
        self.edit(|f| f.deviation_angle = -1.0);
    }
    /// Whether the angle is own; `drawer.has_own_deviation_angle()`.
    pub fn has_own_deviation_angle(&self) -> bool {
        self.fields.read().deviation_angle > 0.0
    }
    /// The previous own angle, 0 when not own; `drawer.previous_deviation_angle()`.
    pub fn previous_deviation_angle(&self) -> f64 {
        let f = self.fields.read();
        if f.deviation_angle > 0.0 {
            f.previous_deviation_angle
        } else {
            0.0
        }
    }
    /// Stores the own angle as previous; `drawer.update_previous_deviation_angle()`.
    pub fn update_previous_deviation_angle(&self) {
        if self.has_own_deviation_angle() {
            let current = self.deviation_angle();
            self.edit(|f| f.previous_deviation_angle = current);
        }
    }
    /// Deprecated `SetHLRAngle(theAngle)`; `drawer.set_hlr_angle(0.2)`.
    pub fn set_hlr_angle(&self, angle: f64) {
        self.set_deviation_angle(angle);
    }
    /// Deprecated `HLRAngle()`; `drawer.hlr_angle()`.
    pub fn hlr_angle(&self) -> f64 {
        self.deviation_angle()
    }
    /// Deprecated `SetHLRAngle()`; `drawer.unset_hlr_angle()`.
    pub fn unset_hlr_angle(&self) {
        self.unset_deviation_angle();
    }
    /// Deprecated `HasOwnHLRDeviationAngle()`; `drawer.has_own_hlr_deviation_angle()`.
    pub fn has_own_hlr_deviation_angle(&self) -> bool {
        self.has_own_deviation_angle()
    }
    /// Deprecated `PreviousHLRDeviationAngle()`; `drawer.previous_hlr_deviation_angle()`.
    pub fn previous_hlr_deviation_angle(&self) -> f64 {
        self.previous_deviation_angle()
    }
}

/// The boolean drawing flags.
impl Prs3dDrawer {
    own_flag_members!(
        iso_on_plane,
        set_iso_on_plane,
        has_own_iso_on_plane,
        unset_own_iso_on_plane,
        iso_on_plane,
        has_own_iso_on_plane,
        false,
        "IsoOnPlane"
    );
    own_flag_members!(
        iso_on_triangulation,
        set_iso_on_triangulation,
        has_own_iso_on_triangulation,
        unset_own_iso_on_triangulation,
        iso_on_triangulation,
        has_own_iso_on_triangulation,
        false,
        "IsoOnTriangulation"
    );
    own_flag_members!(
        is_auto_triangulation,
        set_auto_triangulation,
        has_own_is_auto_triangulation,
        unset_own_is_auto_triangulation,
        is_auto_triangulated,
        has_own_is_auto_triangulated,
        true,
        "IsAutoTriangulation"
    );
    own_flag_members!(
        wire_draw,
        set_wire_draw,
        has_own_wire_draw,
        unset_own_wire_draw,
        wire_draw,
        has_own_wire_draw,
        true,
        "WireDraw"
    );
    own_flag_members!(
        line_arrow_draw,
        set_line_arrow_draw,
        has_own_line_arrow_draw,
        unset_own_line_arrow_draw,
        line_arrow_draw,
        has_own_line_arrow_draw,
        false,
        "LineArrowDraw"
    );
    own_flag_members!(
        free_boundary_draw,
        set_free_boundary_draw,
        has_own_free_boundary_draw,
        unset_own_free_boundary_draw,
        free_boundary_draw,
        has_own_free_boundary_draw,
        true,
        "FreeBoundaryDraw"
    );
    own_flag_members!(
        un_free_boundary_draw,
        set_un_free_boundary_draw,
        has_own_un_free_boundary_draw,
        unset_own_un_free_boundary_draw,
        un_free_boundary_draw,
        has_own_un_free_boundary_draw,
        true,
        "UnFreeBoundaryDraw"
    );
    own_flag_members!(
        face_boundary_draw,
        set_face_boundary_draw,
        has_own_face_boundary_draw,
        unset_own_face_boundary_draw,
        face_boundary_draw,
        has_own_face_boundary_draw,
        false,
        "FaceBoundaryDraw"
    );
    /// Whether hidden lines are drawn, own or from the link; `drawer.draw_hidden_line()`.
    pub fn draw_hidden_line(&self) -> bool {
        self.own_or_link(
            |f| f.has_own_draw_hidden_line.then_some(f.draw_hidden_line),
            Prs3dDrawer::draw_hidden_line,
            |f| f.draw_hidden_line,
        )
    }
    /// Draws hidden lines (own); `drawer.enable_draw_hidden_line()`.
    pub fn enable_draw_hidden_line(&self) {
        self.edit(|f| (f.has_own_draw_hidden_line, f.draw_hidden_line) = (true, true));
    }
    /// Hides hidden lines (own); `drawer.disable_draw_hidden_line()`.
    pub fn disable_draw_hidden_line(&self) {
        self.edit(|f| (f.has_own_draw_hidden_line, f.draw_hidden_line) = (true, false));
    }
    /// Whether the hidden line flag is own; `drawer.has_own_draw_hidden_line()`.
    pub fn has_own_draw_hidden_line(&self) -> bool {
        self.fields.read().has_own_draw_hidden_line
    }
    /// Drops the own hidden line flag; `drawer.unset_own_draw_hidden_line()`.
    pub fn unset_own_draw_hidden_line(&self) {
        self.edit(|f| (f.has_own_draw_hidden_line, f.draw_hidden_line) = (false, false));
    }
}

/// Vertex mode and face boundary continuity.
impl Prs3dDrawer {
    /// Sets the vertex mode (`Inherited` means not own); `drawer.set_vertex_draw_mode(Prs3dVertexDrawMode::All)`.
    pub fn set_vertex_draw_mode(&self, mode: Prs3dVertexDrawMode) {
        self.edit(|f| f.vertex_draw_mode = mode);
    }
    /// The vertex mode, own, from the link, else `Isolated`; `drawer.vertex_draw_mode()`.
    pub fn vertex_draw_mode(&self) -> Prs3dVertexDrawMode {
        self.own_or_link(
            |f| {
                (f.vertex_draw_mode != Prs3dVertexDrawMode::Inherited).then_some(f.vertex_draw_mode)
            },
            Prs3dDrawer::vertex_draw_mode,
            |_| Prs3dVertexDrawMode::Isolated,
        )
    }
    /// Whether the vertex mode is own; `drawer.has_own_vertex_draw_mode()`.
    pub fn has_own_vertex_draw_mode(&self) -> bool {
        self.fields.read().vertex_draw_mode != Prs3dVertexDrawMode::Inherited
    }
    /// Whether the face boundary continuity is own; `drawer.has_own_face_boundary_upper_continuity()`.
    pub fn has_own_face_boundary_upper_continuity(&self) -> bool {
        self.fields.read().face_boundary_upper_continuity.is_some()
    }
    /// The most continuous edge class of face boundaries, own, from the link, else `CN`.
    /// Example: `drawer.face_boundary_upper_continuity()`.
    pub fn face_boundary_upper_continuity(&self) -> Shape {
        self.own_or_link(
            |f| f.face_boundary_upper_continuity,
            Prs3dDrawer::face_boundary_upper_continuity,
            |_| Shape::CN,
        )
    }
    /// Sets the face boundary continuity; `drawer.set_face_boundary_upper_continuity(Shape::C1)`.
    pub fn set_face_boundary_upper_continuity(&self, most_allowed: Shape) {
        self.edit(|f| f.face_boundary_upper_continuity = Some(most_allowed));
    }
    /// Drops the own face boundary continuity; `drawer.unset_face_boundary_upper_continuity()`.
    pub fn unset_face_boundary_upper_continuity(&self) {
        self.edit(|f| f.face_boundary_upper_continuity = None);
    }
}

/// Dimension model and display units.
impl Prs3dDrawer {
    /// Sets own model length units; `drawer.set_dim_length_model_units(&"mm".into())`.
    pub fn set_dim_length_model_units(&self, units: &AsciiString) {
        self.edit(|f| {
            f.has_own_dim_length_model_units = true;
            f.dimension_model_units.set_length_units(units);
        });
    }
    /// Sets own model angle units; `drawer.set_dim_angle_model_units(&"deg".into())`.
    pub fn set_dim_angle_model_units(&self, units: &AsciiString) {
        self.edit(|f| {
            f.has_own_dim_angle_model_units = true;
            f.dimension_model_units.set_angle_units(units);
        });
    }
    /// The model length units, own or from the link; `drawer.dim_length_model_units()`.
    pub fn dim_length_model_units(&self) -> AsciiString {
        self.own_or_link(
            |f| {
                f.has_own_dim_length_model_units
                    .then(|| f.dimension_model_units.get_length_units().clone())
            },
            Prs3dDrawer::dim_length_model_units,
            |f| f.dimension_model_units.get_length_units().clone(),
        )
    }
    /// The model angle units, own or from the link; `drawer.dim_angle_model_units()`.
    pub fn dim_angle_model_units(&self) -> AsciiString {
        self.own_or_link(
            |f| {
                f.has_own_dim_angle_model_units
                    .then(|| f.dimension_model_units.get_angle_units().clone())
            },
            Prs3dDrawer::dim_angle_model_units,
            |f| f.dimension_model_units.get_angle_units().clone(),
        )
    }
    /// Whether the model length units are own; `drawer.has_own_dim_length_model_units()`.
    pub fn has_own_dim_length_model_units(&self) -> bool {
        self.fields.read().has_own_dim_length_model_units
    }
    /// Drops the own model length units (back to "m"); `drawer.unset_own_dim_length_model_units()`.
    pub fn unset_own_dim_length_model_units(&self) {
        self.edit(|f| {
            f.has_own_dim_length_model_units = false;
            f.dimension_model_units
                .set_length_units(&AsciiString::from("m"));
        });
    }
    /// Whether the model angle units are own; `drawer.has_own_dim_angle_model_units()`.
    pub fn has_own_dim_angle_model_units(&self) -> bool {
        self.fields.read().has_own_dim_angle_model_units
    }
    /// Drops the own model angle units (back to "rad"); `drawer.unset_own_dim_angle_model_units()`.
    pub fn unset_own_dim_angle_model_units(&self) {
        self.edit(|f| {
            f.has_own_dim_angle_model_units = false;
            f.dimension_model_units
                .set_angle_units(&AsciiString::from("rad"));
        });
    }
    /// Sets own display length units; `drawer.set_dim_length_display_units(&"mm".into())`.
    pub fn set_dim_length_display_units(&self, units: &AsciiString) {
        self.edit(|f| {
            f.has_own_dim_length_display_units = true;
            f.dimension_display_units.set_length_units(units);
        });
    }
    /// Sets own display angle units; `drawer.set_dim_angle_display_units(&"rad".into())`.
    pub fn set_dim_angle_display_units(&self, units: &AsciiString) {
        self.edit(|f| {
            f.has_own_dim_angle_display_units = true;
            f.dimension_display_units.set_angle_units(units);
        });
    }
    /// The display length units, own or from the link; `drawer.dim_length_display_units()`.
    pub fn dim_length_display_units(&self) -> AsciiString {
        self.own_or_link(
            |f| {
                f.has_own_dim_length_display_units
                    .then(|| f.dimension_display_units.get_length_units().clone())
            },
            Prs3dDrawer::dim_length_display_units,
            |f| f.dimension_display_units.get_length_units().clone(),
        )
    }
    /// The display angle units, own or from the link; `drawer.dim_angle_display_units()`.
    pub fn dim_angle_display_units(&self) -> AsciiString {
        self.own_or_link(
            |f| {
                f.has_own_dim_angle_display_units
                    .then(|| f.dimension_display_units.get_angle_units().clone())
            },
            Prs3dDrawer::dim_angle_display_units,
            |f| f.dimension_display_units.get_angle_units().clone(),
        )
    }
    /// Whether the display length units are own; `drawer.has_own_dim_length_display_units()`.
    pub fn has_own_dim_length_display_units(&self) -> bool {
        self.fields.read().has_own_dim_length_display_units
    }
    /// Drops the own display length units (back to "m"); `drawer.unset_own_dim_length_display_units()`.
    pub fn unset_own_dim_length_display_units(&self) {
        self.edit(|f| {
            f.has_own_dim_length_display_units = false;
            f.dimension_display_units
                .set_length_units(&AsciiString::from("m"));
        });
    }
    /// Whether the display angle units are own; `drawer.has_own_dim_angle_display_units()`.
    pub fn has_own_dim_angle_display_units(&self) -> bool {
        self.fields.read().has_own_dim_angle_display_units
    }
    /// Drops the own display angle units (back to "deg"); `drawer.unset_own_dim_angle_display_units()`.
    pub fn unset_own_dim_angle_display_units(&self) {
        self.edit(|f| {
            f.has_own_dim_angle_display_units = false;
            f.dimension_display_units
                .set_angle_units(&AsciiString::from("deg"));
        });
    }
}
