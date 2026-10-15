//! `PrsDim_Dimension`: the abstract dimension, a trait over embedded base state with the
//! value, label, plane and flyout properties and the selection geometry.
use super::{PrsDimDisplaySpecialSymbol, PrsDimKindOfDimension};
use crate::ais::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrTypeOfPresentation3d};
use crate::prs3d::Prs3dDimensionAspect;
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection};
use crate::std_prs::StdPrsBRepFontCache;
use rustycad_tkernel::resource::ResourceHost;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tkernel::units::Units;
use rustycad_tkernel::units_api::UnitsApi;
use rustycad_tkmath::gp::{self, Direction3, Plane3, Point3};
use std::sync::{Arc, OnceLock};

/// `PrsDim_Dimension::GeometryType`: the measured shapes.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimGeometryType {
    /// `GeometryType_UndefShapes`.
    #[default]
    UndefShapes = 0,
    /// `GeometryType_Edge`.
    Edge,
    /// `GeometryType_Face`.
    Face,
    /// `GeometryType_Points`.
    Points,
    /// `GeometryType_Edges`.
    Edges,
    /// `GeometryType_Faces`.
    Faces,
    /// `GeometryType_EdgeFace`.
    EdgeFace,
    /// `GeometryType_EdgeVertex`.
    EdgeVertex,
}

/// `PrsDim_Dimension::LabelPosition` flags.
pub mod label_position {
    /// `LabelPosition_None`.
    pub const NONE: i32 = 0x00;
    /// `LabelPosition_Left`.
    pub const LEFT: i32 = 0x01;
    /// `LabelPosition_Right`.
    pub const RIGHT: i32 = 0x02;
    /// `LabelPosition_HCenter`.
    pub const H_CENTER: i32 = 0x04;
    /// `LabelPosition_HMask`.
    pub const H_MASK: i32 = LEFT | RIGHT | H_CENTER;
    /// `LabelPosition_Above`.
    pub const ABOVE: i32 = 0x10;
    /// `LabelPosition_Below`.
    pub const BELOW: i32 = 0x20;
    /// `LabelPosition_VCenter`.
    pub const V_CENTER: i32 = 0x40;
    /// `LabelPosition_VMask`.
    pub const V_MASK: i32 = ABOVE | BELOW | V_CENTER;
}

/// `PrsDim_Dimension::ValueType`: a computed value or a custom real or text.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimValueType {
    /// `ValueType_Computed`.
    #[default]
    Computed = 0,
    /// `ValueType_CustomReal`.
    CustomReal,
    /// `ValueType_CustomText`.
    CustomText,
}

/// `PrsDim_Dimension::ComputeMode`: the presentation parts (the selection modes).
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrsDimComputeMode {
    /// `ComputeMode_All`.
    #[default]
    All = 0,
    /// `ComputeMode_Line`.
    Line = 1,
    /// `ComputeMode_Text`.
    Text = 2,
}

/// `SelectionGeometry::Arrow`: an arrow tip and direction.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PrsDimSelectionArrow {
    /// `Position`.
    pub position: Point3,
    /// `Direction`.
    pub direction: Direction3,
}

/// `PrsDim_Dimension::SelectionGeometry`: the drawn geometry the selection is built from.
#[derive(Clone, Debug, PartialEq)]
pub struct PrsDimSelectionGeometry {
    /// `TextPos`: the center of the label.
    pub text_pos: Point3,
    /// `TextDir`: the direction of the label.
    pub text_dir: Direction3,
    /// `TextWidth`.
    pub text_width: f64,
    /// `TextHeight`.
    pub text_height: f64,
    /// `DimensionLine`: the polylines of the dimension line.
    pub dimension_line: Vec<Vec<Point3>>,
    /// `Arrows`.
    pub arrows: Vec<PrsDimSelectionArrow>,
    /// `IsComputed`.
    pub is_computed: bool,
}
impl Default for PrsDimSelectionGeometry {
    fn default() -> Self {
        Self {
            text_pos: gp::origin(),
            text_dir: gp::dx(),
            text_width: 0.0,
            text_height: 0.0,
            dimension_line: Vec::new(),
            arrows: Vec::new(),
            is_computed: false,
        }
    }
}
impl PrsDimSelectionGeometry {
    /// `Clear`: the line and arrows and/or the label of a compute mode.
    /// Example: `geometry.clear(PrsDimComputeMode::All as i32)`.
    pub fn clear(&mut self, mode: i32) {
        if mode == PrsDimComputeMode::All as i32 || mode == PrsDimComputeMode::Line as i32 {
            self.dimension_line.clear();
            self.arrows.clear();
        }
        if mode == PrsDimComputeMode::All as i32 || mode == PrsDimComputeMode::Text as i32 {
            self.text_pos = gp::origin();
            self.text_dir = gp::dx();
            self.text_width = 0.0;
            self.text_height = 0.0;
        }
        self.is_computed = false;
    }
    /// `NewCurve`: a new empty polyline of the dimension line; `geometry.new_curve()`.
    pub fn new_curve(&mut self) -> &mut Vec<Point3> {
        self.dimension_line.push(Vec::new());
        self.dimension_line.last_mut().expect("pushed curve")
    }
    /// `NewArrow`: a new arrow; `geometry.new_arrow()`.
    pub fn new_arrow(&mut self) -> &mut PrsDimSelectionArrow {
        self.arrows.push(PrsDimSelectionArrow::default());
        self.arrows.last_mut().expect("pushed arrow")
    }
}

/// The protected fields of `PrsDim_Dimension`.
#[derive(Clone)]
pub struct PrsDimDimensionFields {
    /// `mySelectionGeom`.
    pub selection_geom: PrsDimSelectionGeometry,
    /// `mySelToleranceForText2d`.
    pub sel_tolerance_for_text_2d: f64,
    /// `myValueType`.
    pub value_type: PrsDimValueType,
    /// `myCustomValue`.
    pub custom_value: f64,
    /// `myCustomStringValue`.
    pub custom_string_value: ExtendedString,
    /// `myFixedTextPosition`.
    pub fixed_text_position: Point3,
    /// `myIsTextPositionFixed`.
    pub is_text_position_fixed: bool,
    /// `mySpecialSymbol` (a UTF-16 unit).
    pub special_symbol: u16,
    /// `myDisplaySpecialSymbol`.
    pub display_special_symbol: PrsDimDisplaySpecialSymbol,
    /// `myGeometryType`.
    pub geometry_type: PrsDimGeometryType,
    /// `myPlane`.
    pub plane: Plane3,
    /// `myIsPlaneCustom`.
    pub is_plane_custom: bool,
    /// `myFlyout`.
    pub flyout: f64,
    /// `myIsGeometryValid`.
    pub is_geometry_valid: bool,
    /// `myBRepFontCache` (private).
    pub brep_font_cache: Handle<StdPrsBRepFontCache>,
    /// `myKindOfDimension` (private).
    pub kind_of_dimension: PrsDimKindOfDimension,
}

/// The embedded base of every dimension; `PrsDimDimensionBase::new(PrsDimKindOfDimension::Length)`.
pub struct PrsDimDimensionBase {
    /// The interactive object base.
    pub interactive: AisInteractiveObjectBase,
    /// The protected fields.
    pub fields: SharedState<PrsDimDimensionFields>,
}
impl PrsDimDimensionBase {
    /// The native constructor state of a dimension kind.
    /// Example: `PrsDimDimensionBase::new(PrsDimKindOfDimension::Radius)`.
    pub fn new(kind: PrsDimKindOfDimension) -> Self {
        Self {
            interactive: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::new(PrsDimDimensionFields {
                selection_geom: PrsDimSelectionGeometry::default(),
                sel_tolerance_for_text_2d: 0.0,
                value_type: PrsDimValueType::Computed,
                custom_value: 0.0,
                custom_string_value: ExtendedString::default(),
                fixed_text_position: Point3::default(),
                is_text_position_fixed: false,
                special_symbol: u16::from(b' '),
                display_special_symbol: PrsDimDisplaySpecialSymbol::No,
                geometry_type: PrsDimGeometryType::UndefShapes,
                plane: Plane3::default(),
                is_plane_custom: false,
                flyout: 0.0,
                is_geometry_valid: false,
                brep_font_cache: Handle::default(),
                kind_of_dimension: kind,
            }),
        }
    }
}

/// The units conversions of the dimension values (`UnitsAPI`, a native global).
pub fn prs_dim_units_api() -> Arc<SharedState<UnitsApi>> {
    static API: OnceLock<Arc<SharedState<UnitsApi>>> = OnceLock::new();
    API.get_or_init(|| {
        Arc::new(SharedState::new(UnitsApi::new(
            Arc::new(Units::new()),
            ResourceHost::native(),
        )))
    })
    .clone()
}

/// `PrsDim_Dimension` (abstract): the virtual members with their native default bodies.
pub trait PrsDimDimensionSource: AisInteractiveObject {
    /// The embedded base; `dimension.dimension_base()`.
    fn dimension_base(&self) -> &PrsDimDimensionBase;
    /// `SetCustomPlane`: the user plane (the fixed label position reset), recomputed when
    /// valid; `dimension.set_custom_plane(&plane)`.
    fn set_custom_plane(&self, plane: &Plane3) {
        prs_dim_dimension_set_custom_plane(self, plane);
    }
    /// `SetTextPosition`: nothing by default; `dimension.set_text_position(&point)`.
    fn set_text_position(&self, position: &Point3) {
        let _ = position;
    }
    /// `GetTextPosition`: the origin by default; `dimension.get_text_position()`.
    fn get_text_position(&self) -> Point3 {
        Point3::default()
    }
    /// `GetDisplayUnits`: empty by default; `dimension.get_display_units()`.
    fn get_display_units(&self) -> AsciiString {
        AsciiString::default()
    }
    /// `GetModelUnits`: empty by default; `dimension.get_model_units()`.
    fn get_model_units(&self) -> AsciiString {
        AsciiString::default()
    }
    /// `SetDisplayUnits`: nothing by default; `dimension.set_display_units(&units)`.
    fn set_display_units(&self, units: &AsciiString) {
        let _ = units;
    }
    /// `SetModelUnits`: nothing by default; `dimension.set_model_units(&units)`.
    fn set_model_units(&self, units: &AsciiString) {
        let _ = units;
    }
    /// `IsValid`: valid geometry in a valid plane; `dimension.is_valid()`.
    fn is_valid(&self) -> bool {
        let (valid, plane) = {
            let fields = self.dimension_base().fields.read();
            (fields.is_geometry_valid, fields.plane)
        };
        valid && self.check_plane(&plane)
    }
    /// `CheckPlane`: any plane by default; `dimension.check_plane(&plane)`.
    fn check_plane(&self, plane: &Plane3) -> bool {
        let _ = plane;
        true
    }
    /// `ComputeValue`: zero by default; `dimension.compute_value()`.
    fn compute_value(&self) -> f64 {
        0.0
    }
    /// `ComputeFlyoutLinePoints`: the ends of the dimension line at the flyout distance from
    /// two points, normal to them in the plane; `dimension.compute_flyout_line_points(&a, &b)?`.
    fn compute_flyout_line_points(
        &self,
        first: &Point3,
        second: &Point3,
    ) -> Result<(Point3, Point3), Failure> {
        super::dimension_linear::prs_dim_dimension_flyout_line_points(self, first, second)
    }
    /// `ComputeFlyoutSelection`: nothing by default.
    /// Example: `dimension.compute_flyout_selection(&selection, &owner)?`.
    fn compute_flyout_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        let _ = (selection, owner);
        Ok(())
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn PrsDimDimensionSource,
    "PrsDim_Dimension",
    dyn AisInteractiveObject
);

/// The native `PrsDim_Dimension::SetCustomPlane` body; `prs_dim_dimension_set_custom_plane(&*d, &p)`.
pub fn prs_dim_dimension_set_custom_plane<D: PrsDimDimensionSource + ?Sized>(
    dimension: &D,
    plane: &Plane3,
) {
    {
        let mut fields = dimension.dimension_base().fields.write();
        fields.plane = *plane;
        fields.is_plane_custom = true;
    }
    dimension.unset_fixed_text_position();
    if dimension.is_valid() {
        dimension.set_to_update_all();
    }
}

/// The non-virtual members of every dimension.
pub trait PrsDimDimensionMembers: PrsDimDimensionSource {
    /// `SetBRepFontCache`: the font cache of 3D labels, recomputed when changed.
    /// Example: `dimension.set_brep_font_cache(&cache)`.
    fn set_brep_font_cache(&self, cache: &Handle<StdPrsBRepFontCache>) {
        {
            let mut fields = self.dimension_base().fields.write();
            if fields.brep_font_cache == *cache {
                return;
            }
            fields.brep_font_cache = cache.clone();
        }
        self.set_to_update_all();
    }
    /// `BRepFontCache`; `dimension.brep_font_cache()`.
    fn brep_font_cache(&self) -> Handle<StdPrsBRepFontCache> {
        self.dimension_base().fields.read().brep_font_cache.clone()
    }
    /// `GetValue`: the custom real, else the computed value; `dimension.get_value()`.
    fn get_value(&self) -> f64 {
        let (kind, value) = {
            let fields = self.dimension_base().fields.read();
            (fields.value_type, fields.custom_value)
        };
        if kind == PrsDimValueType::CustomReal {
            value
        } else {
            self.compute_value()
        }
    }
    /// `SetComputedValue`; `dimension.set_computed_value()`.
    fn set_computed_value(&self) {
        self.dimension_base().fields.write().value_type = PrsDimValueType::Computed;
    }
    /// `SetCustomValue(double)`: a custom real value, recomputed when changed.
    /// Example: `dimension.set_custom_value(2.5)`.
    fn set_custom_value(&self, value: f64) {
        {
            let mut fields = self.dimension_base().fields.write();
            if fields.value_type == PrsDimValueType::CustomReal && fields.custom_value == value {
                return;
            }
            fields.value_type = PrsDimValueType::CustomReal;
            fields.custom_value = value;
        }
        self.set_to_update_all();
    }
    /// `SetCustomValue(text)`: a custom label, recomputed when changed.
    /// Example: `dimension.set_custom_value_text(&"A".into())`.
    fn set_custom_value_text(&self, value: &ExtendedString) {
        {
            let mut fields = self.dimension_base().fields.write();
            if fields.value_type == PrsDimValueType::CustomText
                && fields.custom_string_value == *value
            {
                return;
            }
            fields.value_type = PrsDimValueType::CustomText;
            fields.custom_string_value = value.clone();
        }
        self.set_to_update_all();
    }
    /// `GetCustomValue`; `dimension.get_custom_value()`.
    fn get_custom_value(&self) -> ExtendedString {
        self.dimension_base()
            .fields
            .read()
            .custom_string_value
            .clone()
    }
    /// `GetPlane`; `dimension.get_plane()`.
    fn get_plane(&self) -> Plane3 {
        self.dimension_base().fields.read().plane
    }
    /// `GetGeometryType`; `dimension.get_geometry_type()`.
    fn get_geometry_type(&self) -> i32 {
        self.dimension_base().fields.read().geometry_type as i32
    }
    /// `UnsetCustomPlane`; `dimension.unset_custom_plane()`.
    fn unset_custom_plane(&self) {
        self.dimension_base().fields.write().is_plane_custom = false;
    }
    /// `IsTextPositionCustom`; `dimension.is_text_position_custom()`.
    fn is_text_position_custom(&self) -> bool {
        self.dimension_base().fields.read().is_text_position_fixed
    }
    /// `DimensionAspect`; `dimension.dimension_aspect()`.
    fn dimension_aspect(&self) -> Handle<Prs3dDimensionAspect> {
        self.attributes().dimension_aspect()
    }
    /// `SetDimensionAspect`: the drawer dimension aspect, recomputed.
    /// Example: `dimension.set_dimension_aspect(&aspect)`.
    fn set_dimension_aspect(&self, aspect: &Handle<Prs3dDimensionAspect>) {
        self.attributes().set_dimension_aspect(aspect.clone());
        self.set_to_update_all();
    }
    /// `KindOfDimension`; `dimension.kind_of_dimension()`.
    fn kind_of_dimension(&self) -> PrsDimKindOfDimension {
        self.dimension_base().fields.read().kind_of_dimension
    }
    /// `DisplaySpecialSymbol`; `dimension.display_special_symbol()`.
    fn display_special_symbol(&self) -> PrsDimDisplaySpecialSymbol {
        self.dimension_base().fields.read().display_special_symbol
    }
    /// `SetDisplaySpecialSymbol`, recomputed when changed.
    /// Example: `dimension.set_display_special_symbol(PrsDimDisplaySpecialSymbol::Before)`.
    fn set_display_special_symbol(&self, display: PrsDimDisplaySpecialSymbol) {
        {
            let mut fields = self.dimension_base().fields.write();
            if fields.display_special_symbol == display {
                return;
            }
            fields.display_special_symbol = display;
        }
        self.set_to_update_all();
    }
    /// `SpecialSymbol`; `dimension.special_symbol()`.
    fn special_symbol(&self) -> u16 {
        self.dimension_base().fields.read().special_symbol
    }
    /// `SetSpecialSymbol`, recomputed when changed; `dimension.set_special_symbol(u16::from(b'R'))`.
    fn set_special_symbol(&self, symbol: u16) {
        {
            let mut fields = self.dimension_base().fields.write();
            if fields.special_symbol == symbol {
                return;
            }
            fields.special_symbol = symbol;
        }
        self.set_to_update_all();
    }
    /// `UnsetFixedTextPosition`; `dimension.unset_fixed_text_position()`.
    fn unset_fixed_text_position(&self) {
        let mut fields = self.dimension_base().fields.write();
        fields.is_text_position_fixed = false;
        fields.fixed_text_position = gp::origin();
    }
    /// `SelToleranceForText2d`; `dimension.sel_tolerance_for_text_2d()`.
    fn sel_tolerance_for_text_2d(&self) -> f64 {
        self.dimension_base()
            .fields
            .read()
            .sel_tolerance_for_text_2d
    }
    /// `SetSelToleranceForText2d`, recomputed when changed; `dimension.set_sel_tolerance_for_text_2d(2.0)`.
    fn set_sel_tolerance_for_text_2d(&self, tolerance: f64) {
        {
            let mut fields = self.dimension_base().fields.write();
            if fields.sel_tolerance_for_text_2d == tolerance {
                return;
            }
            fields.sel_tolerance_for_text_2d = tolerance;
        }
        self.set_to_update_all();
    }
    /// `GetFlyout`; `dimension.get_flyout()`.
    fn get_flyout(&self) -> f64 {
        self.dimension_base().fields.read().flyout
    }
    /// `SetFlyout`: the flyout (the fixed label position reset), recomputed when changed.
    /// Example: `dimension.set_flyout(15.0)`.
    fn set_flyout(&self, flyout: f64) {
        {
            let mut fields = self.dimension_base().fields.write();
            if fields.flyout == flyout {
                return;
            }
            fields.flyout = flyout;
        }
        self.unset_fixed_text_position();
        self.set_to_update_all();
    }
    /// `ValueToDisplayUnits`: the value converted from the model to the display units.
    /// Example: `dimension.value_to_display_units()?`.
    fn value_to_display_units(&self) -> Result<f64, Failure> {
        let api = prs_dim_units_api();
        let model = self.get_model_units();
        let display = self.get_display_units();
        let value = self.get_value();
        api.write()
            .any_to_any(value, model.as_bytes(), display.as_bytes())
    }
}
impl<T: PrsDimDimensionSource + ?Sized> PrsDimDimensionMembers for T {}
