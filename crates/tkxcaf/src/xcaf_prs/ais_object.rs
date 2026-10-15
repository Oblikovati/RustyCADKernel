// Rust port of DataExchange/TKXCAF/XCAFPrs/XCAFPrs_AISObject.hxx and .cxx (the object and its
// AIS_ColoredShape virtuals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFPrs_AISObject`: the interactive object of an XDE shape label, an `AIS_ColoredShape`
//! whose sub-shape drawers follow the colors, materials and visibility of the document.
use super::ais_object_styles::{display_text, set_shape_material, set_style_to_drawer};
use super::{XCAFPrs, XCAFPrsStyle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, StandardTransient,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::{
    Graphic3dDisplayPriority, Graphic3dMaterialAspect, Graphic3dNameOfMaterial,
    Graphic3dStructureDisplay, Graphic3dStructureSource,
};
use rustycad_tkv3d::ais::{
    AisColoredDrawer, AisColoredShapeBase, AisColoredShapeSource, AisInteractiveObject,
    AisInteractiveObjectBase, AisKindOfInteractive, AisShapeMembers, AisShapeSource,
    ais_colored_shape_compute, ais_colored_shape_compute_selection, ais_colored_shape_set_color,
    ais_colored_shape_set_transparency, ais_colored_shape_set_width, ais_shape_color_into,
    ais_shape_compute_hlr, ais_shape_material, ais_shape_transparency, ais_shape_unset_color,
    ais_shape_unset_material,
};
use rustycad_tkv3d::prs_mgr::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectBase, PrsMgrPresentableObjectMembers,
    PrsMgrPresentationManager, prs_mgr_presentable_bind,
};
use rustycad_tkv3d::select_mgr::{
    SelectMgrSelectableObject, SelectMgrSelectableObjectBase, SelectMgrSelection,
};

/// The fields of `XCAFPrs_AISObject`.
struct AisObjectFields {
    /// `myLabel`: the label of the shape.
    label: TdfLabel,
    /// `myToSyncStyles`: dispatch the styles again within `Compute`.
    to_sync_styles: bool,
    /// The custom drawers `DispatchStyles` bound: `SetMaterial` visits them, since TKV3d
    /// keeps the iteration of `myShapeColors` private.
    drawers: Vec<Handle<AisColoredDrawer>>,
}

/// `XCAFPrs_AISObject`.
pub struct XCAFPrsAISObject {
    transient: StandardTransient,
    base: AisColoredShapeBase,
    fields: SharedState<AisObjectFields>,
}
rustycad_tkernel::impl_transient!(XCAFPrsAISObject, "XCAFPrs_AISObject", dyn AisColoredShapeSource, transient,
    interfaces: [dyn AisColoredShapeSource, dyn AisShapeSource, dyn AisInteractiveObject,
    dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl XCAFPrsAISObject {
    /// `XCAFPrs_AISObject(theLabel)`: an empty colored shape with its own plastified material,
    /// syncing its styles.
    pub fn new(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisColoredShapeBase::new(&Shape::default()),
            fields: SharedState::new(AisObjectFields {
                label: label.clone(),
                to_sync_styles: true,
                drawers: Vec::new(),
            }),
        });
        prs_mgr_presentable_bind(&object.clone().cast());
        let own = rustycad_tklcaf::live(&object, "XCAFPrs_AISObject")?;
        let material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
        set_shape_material(own, &own.attributes(), &material, (false, false))?;
        own.presentable_base().fields.write().has_own_material = true;
        Ok(object)
    }
    /// `GetLabel`.
    pub fn get_label(&self) -> TdfLabel {
        self.fields.read().label.clone()
    }
    /// `SetLabel(theLabel)`.
    pub fn set_label(&self, label: &TdfLabel) {
        self.fields.write().label = label.clone();
    }
    /// `DefaultStyle(theStyle)`: white surfaces and curves.
    pub fn default_style(&self) -> XCAFPrsStyle {
        let mut style = XCAFPrsStyle::new();
        style.set_color_surf(Color::named(NamedColor::WHITE));
        style.set_color_curv(Color::named(NamedColor::WHITE));
        style
    }
    /// `DispatchStyles(theToSyncStyles)`: the shape of the label, its default style, and one
    /// custom drawer per group of sub-shapes of the same style (the native default flag is
    /// false).
    pub fn dispatch_styles(&self, to_sync_styles: bool) -> Result<(), Failure> {
        {
            let mut fields = self.fields.write();
            fields.to_sync_styles = to_sync_styles;
            fields.drawers.clear();
        }
        self.base.shape_colors.clear(false);
        let label = self.get_label();
        let mut shape = Shape::default();
        if !crate::xcaf_doc::XCAFDocShapeTool::get_shape(&label, &mut shape)? || shape.is_null() {
            self.set_shape(&Shape::default());
            return Ok(());
        }
        self.set_shape(&shape);
        let mut settings = super::XCAFPrsStyleMap::default();
        let white =
            rustycad_tkernel::quantity::ColorRgba::from_rgb(Color::named(NamedColor::WHITE), 1.0);
        XCAFPrs::collect_style_settings(&label, &Location::default(), &mut settings, &white)?;
        let default = self.default_style();
        let drawer = self.attributes();
        let material = front_material(&drawer);
        set_style_to_drawer(&drawer, &default, &default, &material)?;
        super::ais_object_styles::bind_style_groups(self, &settings, &default)
    }
}

/// The custom drawers of the styles.
impl XCAFPrsAISObject {
    /// `myShapeColors`.
    pub(super) fn colored_shape_base_map(&self) -> &rustycad_tkv3d::ais::AisShapeDrawerMap {
        &self.base.shape_colors
    }
    /// Records a drawer bound by `DispatchStyles`.
    pub(super) fn remember_drawer(&self, drawer: &Handle<AisColoredDrawer>) {
        self.fields.write().drawers.push(drawer.clone());
    }
    /// The drawers bound by the last `DispatchStyles`.
    pub(super) fn dispatched_drawers(&self) -> Vec<Handle<AisColoredDrawer>> {
        self.fields.read().drawers.clone()
    }
}

/// The front material of the drawer's shading aspect.
pub(super) fn front_material(
    drawer: &Handle<rustycad_tkv3d::prs3d::Prs3dDrawer>,
) -> Graphic3dMaterialAspect {
    use rustycad_tkservice::graphic3d::Graphic3dAspectsMembers;
    let shading = drawer.shading_aspect();
    shading
        .get()
        .map(|s| s.aspect())
        .and_then(|a| a.get().map(|a| a.front_material()))
        .unwrap_or_default()
}

impl AisColoredShapeSource for XCAFPrsAISObject {
    fn colored_shape_base(&self) -> &AisColoredShapeBase {
        &self.base
    }
}
impl AisShapeSource for XCAFPrsAISObject {
    fn shape_base(&self) -> &rustycad_tkv3d::ais::AisShapeBase {
        &self.base.shape
    }
}
impl AisInteractiveObject for XCAFPrsAISObject {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.shape.interactive
    }
    /// The inherited `AIS_Shape` signature (0).
    fn signature(&self) -> i32 {
        0
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Shape
    }
}
impl SelectMgrSelectableObject for XCAFPrsAISObject {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_colored_shape_compute_selection(self, selection, mode)
    }
}

impl XCAFPrsAISObject {
    /// The test of `Compute`: no other presentation is up to date.
    fn must_map_styles(&self, prs: &dyn HandleReference<dyn Graphic3dStructureSource>) -> bool {
        let current = prs.object_address();
        !self
            .presentations()
            .iter()
            .any(|p| p.object_address() != current && !p.get().is_some_and(|p| p.must_be_updated()))
    }
    /// `Compute`: dispatches the styles when syncing and no other presentation is up to date,
    /// then the colored shape presentation of a non-empty shape, with the names on top in the
    /// view name mode.
    fn compute_object(
        &self,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        if self.fields.read().to_sync_styles && self.must_map_styles(prs) {
            self.dispatch_styles(true)?;
        }
        let shape = self.shape_ref().clone_shape();
        if shape.is_null()
            || (shape.shape_type()? == ShapeEnum::Compound && shape.nb_children() == 0)
        {
            return Ok(());
        }
        ais_colored_shape_compute(self, prs, mode)?;
        if XCAFPrs::get_view_name_mode() {
            let structure = prs.clone_handle();
            if let Some(s) = structure.get() {
                s.set_display_priority(Graphic3dDisplayPriority::Topmost)?;
            }
            let aspect = self.attributes().dimension_aspect();
            let text = aspect.get().map(|a| a.text_aspect()).unwrap_or_default();
            display_text(&self.get_label(), &structure, &text, &Location::default())?;
        }
        Ok(())
    }
}

impl PrsMgrPresentableObject for XCAFPrsAISObject {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.shape.interactive.presentable
    }
    fn compute_hlr(
        &self,
        projector: &Handle<rustycad_tkservice::graphic3d::Graphic3dCamera>,
        trsf: &Handle<rustycad_tkmath::toploc::Datum3>,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        ais_shape_compute_hlr(self, projector, trsf, prs)
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        (0..=2).contains(&mode)
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.compute_object(prs, mode)
    }
    fn color(&self) -> Color {
        let mut color = Color::default();
        ais_shape_color_into(self, &mut color);
        color
    }
    fn material(&self) -> Graphic3dNameOfMaterial {
        ais_shape_material(self)
    }
    fn transparency(&self) -> f64 {
        ais_shape_transparency(self)
    }
    fn set_color(&self, color: Color) -> Result<(), Failure> {
        ais_colored_shape_set_color(self, color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        ais_shape_unset_color(self)
    }
    fn set_width(&self, width: f64) -> Result<(), Failure> {
        ais_colored_shape_set_width(self, width)
    }
    fn unset_width(&self) -> Result<(), Failure> {
        self.set_width(1.0)
    }
    fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        ais_colored_shape_set_transparency(self, value)
    }
    fn unset_transparency(&self) -> Result<(), Failure> {
        self.set_transparency(0.0)
    }
    /// `SetMaterial(theMaterial)`: the material of the object and its default style, then
    /// the material of the custom drawers without their own (with their surface color).
    fn set_material(&self, material: &Graphic3dMaterialAspect) -> Result<(), Failure> {
        super::ais_object_styles::set_object_material(self, material)
    }
    fn unset_material(&self) -> Result<(), Failure> {
        ais_shape_unset_material(self)
    }
    rustycad_tkv3d::ais_presentable_overrides!();
}
