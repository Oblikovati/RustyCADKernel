//! The setup members of `Prs3d_Drawer`: own default aspects copied from a defaults drawer or
//! the link, `ClearLocalAttributes`, `SetShaderProgram` and `SetShadingModel`.
use super::basic_aspect::prs3d_copy_aspects;
use super::drawer::Prs3dDrawerFields;
use super::{
    Prs3dArrowAspect, Prs3dDatumAspect, Prs3dDatumParts, Prs3dDimensionAspect, Prs3dDrawer,
    Prs3dIsoAspect, Prs3dLineAspect, Prs3dPlaneAspect, Prs3dPointAspect, Prs3dShadingAspect,
    Prs3dTextAspect, Prs3dTypeOfHlr, Prs3dVertexDrawMode,
};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkservice::aspect::{AspectTypeOfLine, AspectTypeOfMarker};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsMembers as _, Graphic3dAspectsRenderMembers as _, Graphic3dAspectsSource,
    Graphic3dGroupAspect, Graphic3dShaderProgram, Graphic3dTypeOfShadingModel,
};

/// The default colors of the own line aspects (`THE_DEF_COLOR_*`).
const FREE_BOUNDARY: NamedColor = NamedColor::GREEN;
const UN_FREE_BOUNDARY: NamedColor = NamedColor::YELLOW;
const FACE_BOUNDARY: NamedColor = NamedColor::BLACK;
const WIRE: NamedColor = NamedColor::RED;
const LINE: NamedColor = NamedColor::YELLOW;
const SEEN_LINE: NamedColor = NamedColor::YELLOW;
const HIDDEN_LINE: NamedColor = NamedColor::YELLOW;
const VECTOR: NamedColor = NamedColor::SKYBLUE;
const SECTION: NamedColor = NamedColor::ORANGE;

/// A line aspect slot of the drawer fields.
type LineSlot = fn(&mut Prs3dDrawerFields) -> &mut Handle<Prs3dLineAspect>;
/// The linked getter of a line aspect.
type LineGetter = fn(&Prs3dDrawer) -> Handle<Prs3dLineAspect>;

impl Prs3dDrawer {
    /// The drawer the own aspects copy from: `defaults` unless null or this drawer, else the link.
    fn setup_source(&self, defaults: &Handle<Prs3dDrawer>) -> Handle<Prs3dDrawer> {
        if !defaults.is_null() && !self.is_self(defaults) {
            return defaults.clone();
        }
        self.link()
    }
    /// Creates the own line aspect of a slot when it is null, with the source's Graphic3d
    /// aspect when the source has one; true when created.
    fn ensure_line(
        &self,
        slot: LineSlot,
        color: NamedColor,
        kind: AspectTypeOfLine,
        linked: LineGetter,
        source: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        if !slot(&mut self.fields.write()).is_null() {
            return Ok(false);
        }
        let own = Handle::new(Prs3dLineAspect::new(Color::named(color), kind, 1.0)?);
        if let Some(from) = source.get().map(linked).filter(|h| !h.is_null()) {
            prs3d_copy_aspects(&*own.aspect(), &*from.aspect());
        }
        *slot(&mut self.fields.write()) = own;
        Ok(true)
    }
    /// Creates an own iso aspect (gray75, solid, 1, one iso) copied from the source's.
    fn ensure_iso(
        &self,
        slot: fn(&mut Prs3dDrawerFields) -> &mut Handle<Prs3dIsoAspect>,
        linked: fn(&Prs3dDrawer) -> Handle<Prs3dIsoAspect>,
        source: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        if !slot(&mut self.fields.write()).is_null() {
            return Ok(false);
        }
        let own = Prs3dIsoAspect::new(
            Color::named(NamedColor::GRAY75),
            AspectTypeOfLine::Solid,
            1.0,
            1,
        )?;
        if let Some(from) = source.get().map(linked).filter(|h| !h.is_null()) {
            prs3d_copy_aspects(&*own.aspect(), &*from.aspect());
            own.set_number(from.number());
        }
        *slot(&mut self.fields.write()) = Handle::new(own);
        Ok(true)
    }
    /// `SetupOwnPointAspect`: a yellow plus marker copied from the defaults or the link;
    /// false when the aspect is already own. Example: `drawer.setup_own_point_aspect(&Handle::default())?`.
    pub fn setup_own_point_aspect(&self, defaults: &Handle<Prs3dDrawer>) -> Result<bool, Failure> {
        if self.has_own_point_aspect() {
            return Ok(false);
        }
        let own = Handle::new(Prs3dPointAspect::new(
            AspectTypeOfMarker::Plus,
            Color::named(NamedColor::YELLOW),
            1.0,
        )?);
        let source = self.setup_source(defaults);
        if let Some(from) = source
            .get()
            .map(Prs3dDrawer::point_aspect)
            .filter(|h| !h.is_null())
        {
            prs3d_copy_aspects(&*own.aspect(), &*from.aspect());
        }
        self.edit(|f| f.point_aspect = own);
        Ok(true)
    }
    /// `SetupOwnShadingAspect`: the default shading aspect copied from the defaults or the link;
    /// false when already own. Example: `drawer.setup_own_shading_aspect(&Handle::default())?`.
    pub fn setup_own_shading_aspect(
        &self,
        defaults: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        if self.has_own_shading_aspect() {
            return Ok(false);
        }
        let own = Handle::new(Prs3dShadingAspect::new()?);
        let source = self.setup_source(defaults);
        if let Some(from) = source
            .get()
            .map(Prs3dDrawer::shading_aspect)
            .filter(|h| !h.is_null())
        {
            prs3d_copy_aspects(&*own.aspect(), &*from.aspect());
        }
        self.edit(|f| f.shading_aspect = own);
        Ok(true)
    }
    /// `SetupOwnFaceBoundaryAspect`: a black face boundary aspect copied from the defaults or the
    /// link; false when already own. Example: `drawer.setup_own_face_boundary_aspect(&Handle::default())?`.
    pub fn setup_own_face_boundary_aspect(
        &self,
        defaults: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        let source = self.setup_source(defaults);
        self.ensure_line(
            |f| &mut f.face_boundary_aspect,
            FACE_BOUNDARY,
            AspectTypeOfLine::Solid,
            Prs3dDrawer::face_boundary_aspect,
            &source,
        )
    }
    /// `SetOwnLineAspects`: own iso, wire, line, seen, hidden and boundary line aspects; true
    /// when one was created. Example: `drawer.set_own_line_aspects(&Handle::default())?`.
    pub fn set_own_line_aspects(&self, defaults: &Handle<Prs3dDrawer>) -> Result<bool, Failure> {
        let source = self.setup_source(defaults);
        let mut updated =
            self.ensure_iso(|f| &mut f.u_iso_aspect, Prs3dDrawer::u_iso_aspect, &source)?;
        updated |= self.ensure_iso(|f| &mut f.v_iso_aspect, Prs3dDrawer::v_iso_aspect, &source)?;
        let lines: [(LineSlot, NamedColor, AspectTypeOfLine, LineGetter); 6] = [
            (
                |f| &mut f.wire_aspect,
                WIRE,
                AspectTypeOfLine::Solid,
                Prs3dDrawer::wire_aspect,
            ),
            (
                |f| &mut f.line_aspect,
                LINE,
                AspectTypeOfLine::Solid,
                Prs3dDrawer::line_aspect,
            ),
            (
                |f| &mut f.seen_line_aspect,
                SEEN_LINE,
                AspectTypeOfLine::Solid,
                Prs3dDrawer::seen_line_aspect,
            ),
            (
                |f| &mut f.hidden_line_aspect,
                HIDDEN_LINE,
                AspectTypeOfLine::Dash,
                Prs3dDrawer::hidden_line_aspect,
            ),
            (
                |f| &mut f.free_boundary_aspect,
                FREE_BOUNDARY,
                AspectTypeOfLine::Solid,
                Prs3dDrawer::free_boundary_aspect,
            ),
            (
                |f| &mut f.un_free_boundary_aspect,
                UN_FREE_BOUNDARY,
                AspectTypeOfLine::Solid,
                Prs3dDrawer::un_free_boundary_aspect,
            ),
        ];
        for (slot, color, kind, linked) in lines {
            updated |= self.ensure_line(slot, color, kind, linked, &source)?;
        }
        Ok(self.setup_own_face_boundary_aspect(defaults)? || updated)
    }
    /// `SetOwnDatumAspects`: own vector, section, plane, arrow and datum aspects; true when one
    /// was created. Example: `drawer.set_own_datum_aspects(&Handle::default())?`.
    pub fn set_own_datum_aspects(&self, defaults: &Handle<Prs3dDrawer>) -> Result<bool, Failure> {
        let source = self.setup_source(defaults);
        let mut updated = self.ensure_line(
            |f| &mut f.vector_aspect,
            VECTOR,
            AspectTypeOfLine::Solid,
            Prs3dDrawer::vector_aspect,
            &source,
        )?;
        updated |= self.ensure_line(
            |f| &mut f.section_aspect,
            SECTION,
            AspectTypeOfLine::Solid,
            Prs3dDrawer::section_aspect,
            &source,
        )?;
        if !self.has_own_plane_aspect() {
            updated = true;
            let plane = Handle::new(Prs3dPlaneAspect::new()?);
            self.edit(|f| f.plane_aspect = plane);
        }
        if !self.has_own_arrow_aspect() {
            updated = true;
            let arrow = Handle::new(Prs3dArrowAspect::new()?);
            self.edit(|f| f.arrow_aspect = arrow);
        }
        if !self.has_own_datum_aspect() {
            updated = true;
            let datum = Handle::new(Prs3dDatumAspect::new()?);
            let from = source
                .get()
                .map(Prs3dDrawer::datum_aspect)
                .unwrap_or_default();
            if let Some(from) = from.get() {
                datum.copy_aspects_from(from);
            }
            self.edit(|f| f.datum_aspect = datum);
        }
        Ok(updated)
    }
    /// `SetupOwnDefaults`: own numeric defaults and every own aspect, without link.
    /// Example: `drawer.setup_own_defaults()?`.
    pub fn setup_own_defaults(&self) -> Result<(), Failure> {
        self.edit(|f| {
            f.nb_points = 30;
            f.maximal_parameter_value = 500000.0;
            f.chordial_deviation = 0.0001;
            f.deviation_coefficient = 0.001;
            f.deviation_angle = 20.0 * std::f64::consts::PI / 180.0;
        });
        let none = Handle::default();
        self.setup_own_shading_aspect(&none)?;
        self.setup_own_point_aspect(&none)?;
        self.set_own_datum_aspects(&none)?;
        self.set_own_line_aspects(&none)?;
        self.set_text_aspect(Handle::new(Prs3dTextAspect::new()));
        self.set_dimension_aspect(Handle::new(Prs3dDimensionAspect::new()?));
        Ok(())
    }
    /// `ClearLocalAttributes`: drops every own attribute when the drawer has a link.
    /// Example: `drawer.clear_local_attributes()`.
    pub fn clear_local_attributes(&self) {
        if !self.has_link() {
            return;
        }
        self.edit(clear_own_aspects);
        self.unset_own_discretisation();
        self.unset_own_maximal_parameter_value();
        self.unset_own_type_of_deflection();
        self.unset_own_maximal_chordial_deviation();
        self.unset_deviation_coefficient();
        self.unset_deviation_angle();
        self.unset_own_iso_on_plane();
        self.unset_own_iso_on_triangulation();
        self.unset_own_is_auto_triangulation();
        self.unset_own_wire_draw();
        self.unset_own_line_arrow_draw();
        self.unset_own_draw_hidden_line();
        self.unset_own_free_boundary_draw();
        self.unset_own_un_free_boundary_draw();
        self.unset_own_face_boundary_draw();
        self.unset_own_dim_length_model_units();
        self.unset_own_dim_length_display_units();
        self.unset_own_dim_angle_model_units();
        self.unset_own_dim_angle_display_units();
        self.edit(|f| {
            (f.vertex_draw_mode, f.type_of_hlr) =
                (Prs3dVertexDrawMode::Inherited, Prs3dTypeOfHlr::NotSet)
        });
    }
}

/// Nullifies every own aspect, in the native order.
fn clear_own_aspects(f: &mut Prs3dDrawerFields) {
    f.u_iso_aspect = Handle::default();
    f.v_iso_aspect = Handle::default();
    f.free_boundary_aspect = Handle::default();
    f.un_free_boundary_aspect = Handle::default();
    f.face_boundary_aspect = Handle::default();
    f.wire_aspect = Handle::default();
    f.line_aspect = Handle::default();
    f.text_aspect = Handle::default();
    f.shading_aspect = Handle::default();
    f.point_aspect = Handle::default();
    f.plane_aspect = Handle::default();
    f.arrow_aspect = Handle::default();
    f.hidden_line_aspect = Handle::default();
    f.seen_line_aspect = Handle::default();
    f.vector_aspect = Handle::default();
    f.datum_aspect = Handle::default();
    f.dimension_aspect = Handle::default();
    f.section_aspect = Handle::default();
}

/// `setAspectProgram`: sets the program of a non-null Graphic3d aspect.
fn set_aspect_program<T: Graphic3dAspectsSource + RuntimeTyped>(
    program: &Handle<Graphic3dShaderProgram>,
    aspect: &Handle<T>,
) {
    if let Some(aspect) = aspect.get() {
        aspect.set_shader_program(program.clone());
    }
}

/// `setAspectProgram` of a non-null line aspect.
fn set_line_program(program: &Handle<Graphic3dShaderProgram>, line: &Handle<Prs3dLineAspect>) {
    if let Some(line) = line.get() {
        set_aspect_program(program, &line.aspect());
    }
}

impl Prs3dDrawer {
    /// `SetShaderProgram`: assigns a program to the own aspects of one kind, creating them first
    /// when asked; true when own aspects were created.
    /// Example: `drawer.set_shader_program(&program, Graphic3dGroupAspect::Line, false)?`.
    pub fn set_shader_program(
        &self,
        program: &Handle<Graphic3dShaderProgram>,
        aspect: Graphic3dGroupAspect,
        to_override_defaults: bool,
    ) -> Result<bool, Failure> {
        match aspect {
            Graphic3dGroupAspect::Line => self.set_line_programs(program, to_override_defaults),
            Graphic3dGroupAspect::Text => self.set_text_program(program, to_override_defaults),
            Graphic3dGroupAspect::Marker => {
                let updated =
                    to_override_defaults && self.setup_own_point_aspect(&Handle::default())?;
                let point = self.fields.read().point_aspect.clone();
                if let Some(point) = point.get() {
                    set_aspect_program(program, &point.aspect());
                }
                Ok(updated)
            }
            Graphic3dGroupAspect::FillArea => {
                let updated =
                    to_override_defaults && self.setup_own_shading_aspect(&Handle::default())?;
                let shading = self.fields.read().shading_aspect.clone();
                if let Some(shading) = shading.get() {
                    set_aspect_program(program, &shading.aspect());
                }
                Ok(updated)
            }
        }
    }
    fn set_line_programs(
        &self,
        program: &Handle<Graphic3dShaderProgram>,
        to_override: bool,
    ) -> Result<bool, Failure> {
        let mut updated = false;
        if to_override {
            updated = self.set_own_line_aspects(&Handle::default())? || updated;
            updated = self.set_own_datum_aspects(&Handle::default())? || updated;
        }
        let f = self.fields.snapshot();
        for iso in [&f.u_iso_aspect, &f.v_iso_aspect] {
            if let Some(iso) = iso.get() {
                set_aspect_program(program, &iso.aspect());
            }
        }
        let lines = [
            &f.wire_aspect,
            &f.line_aspect,
            &f.seen_line_aspect,
            &f.hidden_line_aspect,
            &f.vector_aspect,
            &f.section_aspect,
            &f.free_boundary_aspect,
            &f.un_free_boundary_aspect,
            &f.face_boundary_aspect,
        ];
        lines
            .into_iter()
            .for_each(|line| set_line_program(program, line));
        if let Some(plane) = f.plane_aspect.get() {
            [
                plane.edges_aspect(),
                plane.iso_aspect(),
                plane.arrow_aspect(),
            ]
            .iter()
            .for_each(|line| set_line_program(program, line));
        }
        if let Some(datum) = f.datum_aspect.get() {
            [
                Prs3dDatumParts::XAxis,
                Prs3dDatumParts::YAxis,
                Prs3dDatumParts::ZAxis,
            ]
            .into_iter()
            .for_each(|part| set_line_program(program, &datum.line_aspect(part)));
        }
        if let Some(arrow) = f.arrow_aspect.get() {
            set_aspect_program(program, &arrow.aspect());
        }
        Ok(updated)
    }
    fn set_text_program(
        &self,
        program: &Handle<Graphic3dShaderProgram>,
        to_override: bool,
    ) -> Result<bool, Failure> {
        let mut updated = false;
        if to_override && !self.has_own_text_aspect() {
            updated = true;
            let own = Handle::new(Prs3dTextAspect::new());
            let link = self.link();
            if let Some(from) = link
                .get()
                .map(Prs3dDrawer::text_aspect)
                .filter(|h| !h.is_null())
            {
                prs3d_copy_aspects(&*own.aspect(), &*from.aspect());
            }
            self.edit(|f| f.text_aspect = own);
        }
        let text = self.fields.read().text_aspect.clone();
        if let Some(text) = text.get() {
            set_aspect_program(program, &text.aspect());
        }
        Ok(updated)
    }
    /// `SetShadingModel`: sets the model of the own shading aspect, creating it first when
    /// asked; true when it was created. Example: `drawer.set_shading_model(Graphic3dTypeOfShadingModel::Phong, true)?`.
    pub fn set_shading_model(
        &self,
        model: Graphic3dTypeOfShadingModel,
        to_override_defaults: bool,
    ) -> Result<bool, Failure> {
        let updated = to_override_defaults && self.setup_own_shading_aspect(&Handle::default())?;
        let shading = self.fields.read().shading_aspect.clone();
        let fill = shading
            .get()
            .map(Prs3dShadingAspect::aspect)
            .unwrap_or_default();
        if let Some(fill) = fill.get() {
            fill.set_shading_model(model);
        }
        Ok(updated)
    }
}
