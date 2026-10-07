//! `V3d_Trihedron`: the screen-corner axes of a view, drawn by its own structure with
//! trihedron transform persistence.
use super::computed_structure::{V3dComputedStructure, V3dStructureComputer};
use super::{V3dTypeOfAxe, V3dView};
use crate::prs3d::{Prs3dBasicAspect as _, Prs3dShadingAspect, Prs3dTextAspect};
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::{
    AspectInteriorStyle, AspectTypeOfFacingModel, AspectTypeOfTriedronPosition,
};
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_TOPMOST, Graphic3dAspectsMembers as _, Graphic3dAspectsRenderMembers as _,
    Graphic3dCViewMembers as _, Graphic3dDisplayPriority, Graphic3dMaterialAspect,
    Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
    Graphic3dTransModeFlags, Graphic3dTransformPers, Graphic3dTransformPersMembers as _,
    Graphic3dTransformPersSource as _, Graphic3dTypeOfMaterial, Graphic3dViewAffinity,
};
use rustycad_tkservice::json_dump::{class_begin, field_dumped, field_integer, field_real};
use std::io;

/// `THE_INTERNAL_SCALE_FACTOR`.
pub(super) const INTERNAL_SCALE_FACTOR: f64 = 500.0;
/// `THE_CYLINDER_LENGTH` (a float natively).
pub(super) const CYLINDER_LENGTH: f32 = 0.75;
/// `THE_CIRCLE_SERMENTS_NB`.
pub(super) const CIRCLE_SEGMENTS_NB: i32 = 24;
/// `Font_NOF_ASCII_MONO`.
const FONT_ASCII_MONO: &str = "Courier";

/// The mutable fields of the trihedron.
#[derive(Clone)]
pub(super) struct TrihedronState {
    pub(super) labels: [AsciiString; 3],
    pub(super) structure: Handle<dyn Graphic3dStructureSource>,
    pub(super) scale: f64,
    pub(super) ratio: f64,
    pub(super) diameter: f64,
    pub(super) nb_facettes: i32,
    pub(super) is_wireframe: bool,
    pub(super) to_compute: bool,
}

/// The trihedron of a view.
///
/// Example: `let trihedron = V3dTrihedron::new()?; trihedron.set_scale(0.1); trihedron.display(&view)?;`.
pub struct V3dTrihedron {
    transient: StandardTransient,
    pub(super) sphere_shading_aspect: Handle<Prs3dShadingAspect>,
    pub(super) text_aspects: [Handle<Prs3dTextAspect>; 3],
    pub(super) arrow_shading_aspects: [Handle<Prs3dShadingAspect>; 3],
    transform_pers: Handle<Graphic3dTransformPers>,
    this: SharedState<WeakHandle<V3dTrihedron>>,
    pub(super) state: SharedState<TrihedronState>,
}
rustycad_tkernel::impl_transient!(V3dTrihedron, "V3d_Trihedron", StandardTransient, transient,
    interfaces: [dyn V3dStructureComputer]);

/// The material of the trihedron shading: no specular, aspect color.
fn shading_material() -> Graphic3dMaterialAspect {
    let mut material = Graphic3dMaterialAspect::default();
    material.set_specular_color(Color::named(NamedColor::BLACK));
    material.set_material_type(Graphic3dTypeOfMaterial::Aspect);
    material
}

/// A solid, textured shading aspect of a color with the trihedron material.
fn trihedron_shading(color: Color) -> Result<Handle<Prs3dShadingAspect>, Failure> {
    let shading = Prs3dShadingAspect::new()?;
    if let Some(fill) = shading.aspect().get() {
        fill.set_texture_map_on(true);
        fill.set_interior_style(AspectInteriorStyle::Solid);
    }
    shading.set_material(&shading_material(), AspectTypeOfFacingModel::BothSide);
    shading.set_color(color, AspectTypeOfFacingModel::BothSide)?;
    Ok(Handle::new(shading))
}

/// A Courier label aspect of height 16, left and bottom aligned.
fn trihedron_text() -> Handle<Prs3dTextAspect> {
    let text = Prs3dTextAspect::new();
    text.set_font(FONT_ASCII_MONO);
    text.set_height(16.0);
    text.set_horizontal_justification(
        rustycad_tkservice::graphic3d::Graphic3dHorizontalTextAlignment::Left,
    );
    text.set_vertical_justification(
        rustycad_tkservice::graphic3d::Graphic3dVerticalTextAlignment::Bottom,
    );
    Handle::new(text)
}

impl V3dTrihedron {
    /// A trihedron at the lower-left corner: red, green and blue arrows and white origin.
    /// Example: `V3dTrihedron::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let pers = Graphic3dTransformPers::with_corner(
            Graphic3dTransModeFlags::TRIEDRON_PERS,
            AspectTypeOfTriedronPosition::LEFT_LOWER,
            Vec2::new(0, 0),
        )?;
        let colors = [NamedColor::RED, NamedColor::GREEN, NamedColor::BLUE];
        let arrows = [
            trihedron_shading(Color::named(colors[0]))?,
            trihedron_shading(Color::named(colors[1]))?,
            trihedron_shading(Color::named(colors[2]))?,
        ];
        let state = TrihedronState {
            labels: [
                AsciiString::from("X"),
                AsciiString::from("Y"),
                AsciiString::from("Z"),
            ],
            structure: Handle::default(),
            scale: 1.0,
            ratio: 0.8,
            diameter: 0.05,
            nb_facettes: 12,
            is_wireframe: false,
            to_compute: true,
        };
        let trihedron = Handle::new(Self {
            transient: StandardTransient::default(),
            sphere_shading_aspect: trihedron_shading(Color::named(NamedColor::WHITE))?,
            text_aspects: [trihedron_text(), trihedron_text(), trihedron_text()],
            arrow_shading_aspects: arrows,
            transform_pers: Handle::new(pers),
            this: SharedState::new(WeakHandle::default()),
            state: SharedState::new(state),
        });
        *trihedron.this.write() = trihedron.downgrade();
        trihedron.set_position(AspectTypeOfTriedronPosition::LEFT_LOWER)?;
        Ok(trihedron)
    }
    /// Whether the axes are drawn as lines; `trihedron.is_wireframe()`.
    pub fn is_wireframe(&self) -> bool {
        self.state.read().is_wireframe
    }
    /// Switches wireframe drawing (recomputed on change); `trihedron.set_wireframe(true)`.
    pub fn set_wireframe(&self, as_wireframe: bool) {
        let mut state = self.state.write();
        if state.is_wireframe != as_wireframe {
            (state.to_compute, state.is_wireframe) = (true, as_wireframe);
        }
    }
    /// The transform persistence; `trihedron.transform_persistence()`.
    pub fn transform_persistence(&self) -> Handle<Graphic3dTransformPers> {
        self.transform_pers.clone()
    }
    /// `SetPosition`: the corner and the offset `scale * 500` along the attached sides.
    /// Example: `trihedron.set_position(AspectTypeOfTriedronPosition::RIGHT_UPPER)?`.
    pub fn set_position(&self, position: AspectTypeOfTriedronPosition) -> Result<(), Failure> {
        let scale = self.state.read().scale;
        let shift = (scale * INTERNAL_SCALE_FACTOR) as i32;
        let mut offset = Vec2::new(0, 0);
        if position.0
            & (AspectTypeOfTriedronPosition::LEFT.0 | AspectTypeOfTriedronPosition::RIGHT.0)
            != 0
        {
            offset.x = shift;
        }
        if position.0
            & (AspectTypeOfTriedronPosition::TOP.0 | AspectTypeOfTriedronPosition::BOTTOM.0)
            != 0
        {
            offset.y = shift;
        }
        if let Some(pers) = self.transform_pers.get() {
            pers.set_corner_2d(position)?;
            pers.set_offset_2d(offset)?;
        }
        Ok(())
    }
    /// The scale; `trihedron.scale()`.
    pub fn scale(&self) -> f64 {
        self.state.read().scale
    }
    /// Sets the scale (recomputed beyond `Precision::Confusion`); `trihedron.set_scale(0.1)`.
    pub fn set_scale(&self, scale: f64) {
        let mut state = self.state.write();
        if (state.scale - scale).abs() > confusion() {
            state.to_compute = true;
        }
        state.scale = scale;
    }
    /// The size ratio; `trihedron.size_ratio()`.
    pub fn size_ratio(&self) -> f64 {
        self.state.read().ratio
    }
    /// Sets the size ratio; `trihedron.set_size_ratio(0.8)`.
    pub fn set_size_ratio(&self, ratio: f64) {
        let mut state = self.state.write();
        if (state.ratio - ratio).abs() > confusion() {
            state.to_compute = true;
        }
        state.ratio = ratio;
    }
    /// The arrow diameter; `trihedron.arrow_diameter()`.
    pub fn arrow_diameter(&self) -> f64 {
        self.state.read().diameter
    }
    /// Sets the arrow diameter; `trihedron.set_arrow_diameter(0.05)`.
    pub fn set_arrow_diameter(&self, diameter: f64) {
        let mut state = self.state.write();
        if (state.diameter - diameter).abs() > confusion() {
            state.to_compute = true;
        }
        state.diameter = diameter;
    }
    /// The facet count of the shaded parts; `trihedron.nb_facets()`.
    pub fn nb_facets(&self) -> i32 {
        self.state.read().nb_facettes
    }
    /// Sets the facet count; `trihedron.set_nb_facets(12)`.
    pub fn set_nb_facets(&self, nb_facets: i32) {
        let mut state = self.state.write();
        if state.nb_facettes != nb_facets {
            state.to_compute = true;
        }
        state.nb_facettes = nb_facets;
    }
    /// The label aspect of an axis; `trihedron.label_aspect(V3dTypeOfAxe::X)`.
    pub fn label_aspect(&self, axis: V3dTypeOfAxe) -> Handle<Prs3dTextAspect> {
        self.text_aspects[axis as usize].clone()
    }
    /// `SetLabelsColor(x, y, z)`; `trihedron.set_labels_colors(red, green, blue)`.
    pub fn set_labels_colors(&self, x_color: Color, y_color: Color, z_color: Color) {
        for (aspect, color) in self.text_aspects.iter().zip([x_color, y_color, z_color]) {
            aspect.set_color(color);
        }
    }
    /// `SetLabelsColor(color)`: one color for the three labels; `trihedron.set_labels_color(white)`.
    pub fn set_labels_color(&self, color: Color) {
        self.set_labels_colors(color, color, color);
    }
    /// The arrow aspect of an axis; `trihedron.arrow_aspect(V3dTypeOfAxe::Y)`.
    pub fn arrow_aspect(&self, axis: V3dTypeOfAxe) -> Handle<Prs3dShadingAspect> {
        self.arrow_shading_aspects[axis as usize].clone()
    }
    /// `SetArrowsColor`; `trihedron.set_arrows_color(red, green, blue)?`.
    pub fn set_arrows_color(
        &self,
        x_color: Color,
        y_color: Color,
        z_color: Color,
    ) -> Result<(), Failure> {
        for (aspect, color) in self
            .arrow_shading_aspects
            .iter()
            .zip([x_color, y_color, z_color])
        {
            if let Some(aspect) = aspect.get() {
                aspect.set_color(color, AspectTypeOfFacingModel::BothSide)?;
            }
        }
        Ok(())
    }
    /// The origin sphere aspect; `trihedron.origin_aspect()`.
    pub fn origin_aspect(&self) -> Handle<Prs3dShadingAspect> {
        self.sphere_shading_aspect.clone()
    }
    /// The label of an axis; `trihedron.label(V3dTypeOfAxe::Z)`.
    pub fn label(&self, axis: V3dTypeOfAxe) -> AsciiString {
        self.state.read().labels[axis as usize].clone()
    }
    /// `SetLabels` (recomputed when one changes); `trihedron.set_labels(&x, &y, &z)`.
    pub fn set_labels(&self, x: &AsciiString, y: &AsciiString, z: &AsciiString) {
        let mut state = self.state.write();
        if state.labels[0] != *x || state.labels[1] != *y || state.labels[2] != *z {
            state.to_compute = true;
            state.labels = [x.clone(), y.clone(), z.clone()];
        }
    }
    /// `Display`: creates the structure on the first call (topmost layer, visible only in
    /// the view), recomputes when needed and displays it. Example: `trihedron.display(&view)?`.
    pub fn display(&self, view: &V3dView) -> Result<(), Failure> {
        if self.state.read().structure.is_null() {
            self.create_structure(view)?;
        }
        if self.state.read().to_compute {
            self.compute()?;
        }
        if let Some(structure) = self.state.read().structure.get() {
            structure.display();
        }
        Ok(())
    }
    fn create_structure(&self, view: &V3dView) -> Result<(), Failure> {
        let Some(viewer) = view.viewer().get().map(|v| v.structure_manager()) else {
            return Ok(());
        };
        let owner: Handle<dyn V3dStructureComputer> = self.this.read().upgrade().cast();
        let structure = V3dComputedStructure::create(&viewer, &owner)?;
        if let Some(value) = structure.get() {
            value.set_transform_persistence(&self.transform_pers.clone().cast());
            value.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
            value.set_display_priority(Graphic3dDisplayPriority::Highlight)?;
            value.set_infinite_state(true);
            if let Some(cstructure) = value.cstructure().get() {
                let affinity = Graphic3dViewAffinity::default();
                affinity.set_visible(false);
                affinity.set_view_visible(view.view().identification(), true);
                cstructure.cstructure_base().fields.write().view_affinity = Handle::new(affinity);
            }
        }
        let mut state = self.state.write();
        (state.structure, state.to_compute) = (structure, true);
        Ok(())
    }
    /// `Erase`: erases and forgets the structure; `trihedron.erase()`.
    pub fn erase(&self) {
        let structure = std::mem::take(&mut self.state.write().structure);
        if let Some(value) = structure.get() {
            value.erase();
        }
    }
}

impl V3dStructureComputer for V3dTrihedron {
    /// `TrihedronStructure::Compute`; the native `void` member reports no failure.
    fn compute_structure(&self) {
        let _ = self.compute();
    }
}

impl Drop for V3dTrihedron {
    /// The destructor erases the trihedron.
    fn drop(&mut self) {
        self.erase();
    }
}

impl V3dTrihedron {
    /// `DumpJson`; `trihedron.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "V3d_Trihedron")?;
        let sphere = &self.sphere_shading_aspect;
        field_dumped(stream, depth, "mySphereShadingAspect.get()", &|s, d| {
            sphere.dump_json(s, d)
        })?;
        for text in &self.text_aspects {
            field_dumped(stream, depth, "aTextAspect.get()", &|s, d| {
                text.dump_json(s, d)
            })?;
        }
        for arrow in &self.arrow_shading_aspects {
            field_dumped(stream, depth, "anArrowShadinAspect.get()", &|s, d| {
                arrow.dump_json(s, d)
            })?;
        }
        let state = self.state.read().clone();
        if let Some(structure) = state.structure.get() {
            field_dumped(stream, depth, "myStructure.get()", &|s, d| {
                structure.dump_json(s, d)
            })?;
        }
        let pers = &self.transform_pers;
        field_dumped(stream, depth, "myTransformPers.get()", &|s, d| {
            pers.dump_json(s, d)
        })?;
        field_real(stream, "myScale", state.scale)?;
        field_real(stream, "myRatio", state.ratio)?;
        field_real(stream, "myDiameter", state.diameter)?;
        field_integer(stream, "myNbFacettes", state.nb_facettes)?;
        field_integer(stream, "myIsWireframe", i32::from(state.is_wireframe))?;
        field_integer(stream, "myToCompute", i32::from(state.to_compute))
    }
}
