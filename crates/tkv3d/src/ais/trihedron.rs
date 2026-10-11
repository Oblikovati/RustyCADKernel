//! Martensite selectable trihedrons preserve OCCT 3d097a0 protected state and callbacks.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::{prs_mgr::*, prs3d::*};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::ExtendedString,
};
use rustycad_tkg3d::geom::GeomAxis2Placement;
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
/// Native owner modes; `AisTrihedronSelectionMode::Axes as i32`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AisTrihedronSelectionMode {
    /// Whole trihedron owner.
    EntireObject = 0,
    /// Origin owner.
    Origin = 1,
    /// Individual axis owners.
    Axes = 2,
    /// Main plane owners.
    MainPlanes = 3,
}
/// Native protected storage, mutable by derived objects; `base.fields.write().has_own_size = true`.
pub struct AisTrihedronFields {
    /// Aliased placement.
    pub component: Handle<GeomAxis2Placement>,
    /// Wire or shaded primitives, independent of AIS display mode.
    pub datum_mode: Prs3dDatumMode,
    /// Own size flag.
    pub has_own_size: bool,
    /// Native text ownership flag (setters leave it unchanged).
    pub has_own_text_color: bool,
    /// Native arrow ownership flag.
    pub has_own_arrow_color: bool,
    /// Native part-indexed labels.
    pub labels: [ExtendedString; Prs3dDatumParts::NB],
    /// Native part-indexed owner priorities.
    pub selection_priority: [i32; Prs3dDatumParts::NB],
    /// Last presentation's per-part groups.
    pub part_groups: [Handle<dyn Graphic3dGroup>; Prs3dDatumParts::NB],
    /// Selected parts in native traversal order.
    pub selected_parts: Vec<Prs3dDatumParts>,
    /// Empty line used to retain selectable hidden planes.
    pub hidden_line_aspect: Handle<Graphic3dAspectLine3d>,
    /// Cached arrays; selection uses the last computed geometry.
    pub primitives: [Handle<Graphic3dArrayOfPrimitives>; Prs3dDatumParts::NB],
}
/// Embeddable inherited and native protected state; `AisTrihedronBase::new(&placement)?`.
pub struct AisTrihedronBase {
    /// Inherited interactive fields.
    pub interactive: AisInteractiveObjectBase,
    /// Native protected fields.
    pub fields: SharedState<AisTrihedronFields>,
}
impl AisTrihedronBase {
    /// Initialize constructor priorities and labels; `AisTrihedronBase::new(&placement)?`.
    pub fn new(component: &Handle<GeomAxis2Placement>) -> Result<Self, Failure> {
        let interactive = AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView);
        interactive.selectable.fields.write().auto_hilight = false;
        Ok(Self {
            interactive,
            fields: SharedState::new(trihedron_initial_fields(component)?),
        })
    }
}
fn trihedron_initial_fields(
    component: &Handle<GeomAxis2Placement>,
) -> Result<AisTrihedronFields, Failure> {
    let mut priority = [0; Prs3dDatumParts::NB];
    priority[Prs3dDatumParts::None as usize] = 5;
    priority[Prs3dDatumParts::Origin as usize] = 8;
    for part in TRI_AXIS_PARTS {
        priority[part as usize] = 7;
    }
    for part in TRI_PLANE_PARTS {
        priority[part as usize] = 5;
    }
    let mut labels = std::array::from_fn(|_| ExtendedString::new(""));
    for (part, label) in TRI_AXIS_PARTS.into_iter().zip(["X", "Y", "Z"]) {
        labels[part as usize] = ExtendedString::new(label);
    }
    Ok(AisTrihedronFields {
        component: component.clone(),
        datum_mode: Prs3dDatumMode::WireFrame,
        has_own_size: false,
        has_own_text_color: false,
        has_own_arrow_color: false,
        labels,
        selection_priority: priority,
        part_groups: std::array::from_fn(|_| Handle::default()),
        selected_parts: Vec::new(),
        hidden_line_aspect: Handle::new(Graphic3dAspectLine3d::new(
            Color::named(NamedColor::WHITE),
            AspectTypeOfLine::Empty,
            1.,
        )?),
        primitives: std::array::from_fn(|_| Handle::default()),
    })
}
pub(super) const TRI_AXIS_PARTS: [Prs3dDatumParts; 3] = [
    Prs3dDatumParts::XAxis,
    Prs3dDatumParts::YAxis,
    Prs3dDatumParts::ZAxis,
];
pub(super) const TRI_PLANE_PARTS: [Prs3dDatumParts; 3] = [
    Prs3dDatumParts::XOYAxis,
    Prs3dDatumParts::YOZAxis,
    Prs3dDatumParts::XOZAxis,
];
/// Native interface and protected seam for application subclasses; `trihedron.size()`.
pub trait AisTrihedronSource: AisInteractiveObject {
    /// Embedded native state; `object.trihedron_base()`.
    fn trihedron_base(&self) -> &AisTrihedronBase;
    /// Native datum_display_mode value; `trihedron.datum_display_mode()`.
    fn datum_display_mode(&self) -> Prs3dDatumMode {
        self.trihedron_base().fields.read().datum_mode
    }
    /// Native component value; `trihedron.component()`.
    fn component(&self) -> Handle<GeomAxis2Placement> {
        self.trihedron_base().fields.read().component.clone()
    }
    /// Native has_own_size value; `trihedron.has_own_size()`.
    fn has_own_size(&self) -> bool {
        self.trihedron_base().fields.read().has_own_size
    }
    /// Native has_text_color value; `trihedron.has_text_color()`.
    fn has_text_color(&self) -> bool {
        self.trihedron_base().fields.read().has_own_text_color
    }
    /// Native has_arrow_color value; `trihedron.has_arrow_color()`.
    fn has_arrow_color(&self) -> bool {
        self.trihedron_base().fields.read().has_own_arrow_color
    }
    /// Native size value; `trihedron.size()`.
    fn size(&self) -> f64 {
        self.attributes()
            .datum_aspect()
            .axis_length(Prs3dDatumParts::XAxis)
    }
    /// Native to_draw_arrows value; `trihedron.to_draw_arrows()`.
    fn to_draw_arrows(&self) -> bool {
        self.attributes().datum_aspect().to_draw_arrows()
    }
    /// Native text_color value; `trihedron.text_color()`.
    fn text_color(&self) -> Color {
        self.attributes()
            .datum_aspect()
            .text_aspect(Prs3dDatumParts::XAxis)
            .aspect()
            .interior_color()
    }
    /// Native arrow_color value; `trihedron.arrow_color()`.
    fn arrow_color(&self) -> Color {
        self.attributes()
            .datum_aspect()
            .arrow_aspect()
            .aspect()
            .interior_color()
    }
    /// Assign drawing mode without scheduling updates; `trihedron.set_datum_display_mode(Prs3dDatumMode::Shaded)`.
    fn set_datum_display_mode(&self, mode: Prs3dDatumMode) {
        self.trihedron_base().fields.write().datum_mode = mode;
    }
    /// Retain geometry and flag presentation updates only; `trihedron.set_component(&placement)`.
    fn set_component(&self, placement: &Handle<GeomAxis2Placement>) {
        self.trihedron_base().fields.write().component = placement.clone();
        self.set_to_update_all();
    }
    /// Native part priority; `trihedron.selection_priority(Prs3dDatumParts::Origin)`.
    fn selection_priority(&self, part: Prs3dDatumParts) -> i32 {
        self.trihedron_base().fields.read().selection_priority[part as usize]
    }
    /// Assign without rebuilding owners; `trihedron.set_selection_priority(Prs3dDatumParts::Origin,9)`.
    fn set_selection_priority(&self, part: Prs3dDatumParts, priority: i32) {
        self.trihedron_base().fields.write().selection_priority[part as usize] = priority;
    }
    /// Copy a stored native label; `trihedron.label(Prs3dDatumParts::XAxis)`.
    fn label(&self, part: Prs3dDatumParts) -> ExtendedString {
        self.trihedron_base().fields.read().labels[part as usize].clone()
    }
    /// Assign without scheduling updates; `trihedron.set_label(Prs3dDatumParts::XAxis,&ExtendedString::new("U"))`.
    fn set_label(&self, part: Prs3dDatumParts, label: &ExtendedString) {
        self.trihedron_base().fields.write().labels[part as usize] = label.clone();
    }
    /// Last computed native array; `trihedron.array_of_primitives(Prs3dDatumParts::XAxis)`.
    fn array_of_primitives(&self, part: Prs3dDatumParts) -> Handle<Graphic3dArrayOfPrimitives> {
        self.trihedron_base().fields.read().primitives[part as usize].clone()
    }
    /// Preserve native set_own_datum_aspect ownership behavior; `trihedron.set_own_datum_aspect()?`.
    fn set_own_datum_aspect(&self) -> Result<(), Failure> {
        super::ais_trihedron_own_datum(self)
    }
    /// Preserve native set_size ownership behavior; `trihedron.set_size(10.)?`.
    fn set_size(&self, size: f64) -> Result<(), Failure> {
        super::ais_trihedron_set_size(self, size)
    }
    /// Preserve native unset_size ownership behavior; `trihedron.unset_size()?`.
    fn unset_size(&self) -> Result<(), Failure> {
        super::ais_trihedron_unset_size(self)
    }
    /// Preserve native set_datum_part_color ownership behavior; `trihedron.set_datum_part_color(Prs3dDatumParts::XAxis, red)?`.
    fn set_datum_part_color(&self, part: Prs3dDatumParts, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_part_color(self, part, color)
    }
    /// Preserve native datum_part_color ownership behavior; `trihedron.datum_part_color(Prs3dDatumParts::XAxis)?`.
    fn datum_part_color(&self, part: Prs3dDatumParts) -> Result<Color, Failure> {
        super::ais_trihedron_get_part_color(self, part)
    }
    /// Preserve native set_text_color ownership behavior; `trihedron.set_text_color(red)?`.
    fn set_text_color(&self, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_text_color(self, color)
    }
    /// Preserve native set_text_color_part ownership behavior; `trihedron.set_text_color_part(Prs3dDatumParts::XAxis, red)?`.
    fn set_text_color_part(&self, part: Prs3dDatumParts, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_text_part_color(self, part, color)
    }
    /// Preserve native set_arrow_color ownership behavior; `trihedron.set_arrow_color(red)?`.
    fn set_arrow_color(&self, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_arrow_color(self, color)
    }
    /// Preserve native set_arrow_color_part ownership behavior; `trihedron.set_arrow_color_part(Prs3dDatumParts::XAxis, red)?`.
    fn set_arrow_color_part(&self, part: Prs3dDatumParts, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_arrow_part_color(self, part, color)
    }
    /// Preserve native set_draw_arrows ownership behavior; `trihedron.set_draw_arrows(false)?`.
    fn set_draw_arrows(&self, draw: bool) -> Result<(), Failure> {
        super::ais_trihedron_draw_arrows(self, draw)
    }
    /// Preserve native set_origin_color ownership behavior; `trihedron.set_origin_color(red)?`.
    fn set_origin_color(&self, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_origin_color(self, color)
    }
    /// Preserve native set_x_axis_color ownership behavior; `trihedron.set_x_axis_color(red)?`.
    fn set_x_axis_color(&self, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_part_color(self, Prs3dDatumParts::XAxis, color)
    }
    /// Preserve native set_y_axis_color ownership behavior; `trihedron.set_y_axis_color(red)?`.
    fn set_y_axis_color(&self, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_part_color(self, Prs3dDatumParts::YAxis, color)
    }
    /// Preserve native set_axis_color ownership behavior; `trihedron.set_axis_color(red)?`.
    fn set_axis_color(&self, color: Color) -> Result<(), Failure> {
        super::ais_trihedron_part_color(self, Prs3dDatumParts::ZAxis, color)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisTrihedronSource,
    "AIS_Trihedron",
    dyn AisInteractiveObject
);
/// Selectable wire/shaded datum; `AisTrihedron::new(&placement)?`.
pub struct AisTrihedron {
    transient: StandardTransient,
    base: AisTrihedronBase,
}
rustycad_tkernel::impl_transient!(AisTrihedron,"AIS_Trihedron",dyn AisInteractiveObject,transient,interfaces:[dyn AisTrihedronSource,dyn AisInteractiveObject,dyn crate::select_mgr::SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisTrihedron {
    /// Bind the native datum constructor; `AisTrihedron::new(&placement)?`.
    pub fn new(component: &Handle<GeomAxis2Placement>) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisTrihedronBase::new(component)?,
        });
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
}
impl AisTrihedronSource for AisTrihedron {
    fn trihedron_base(&self) -> &AisTrihedronBase {
        &self.base
    }
}
impl AisInteractiveObject for AisTrihedron {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
    fn kind_of_interactive(&self) -> super::AisKindOfInteractive {
        super::AisKindOfInteractive::Datum
    }
    fn signature(&self) -> i32 {
        3
    }
}
