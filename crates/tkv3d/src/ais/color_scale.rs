//! Martensite color-scale interactive presentation, from OCCT 3d097a0.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::{
    quantity::*,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::{AsciiString, ExtendedString},
};
use rustycad_tkservice::{aspect::*, graphic3d::*};
#[derive(Clone)]
pub(super) struct ColorScaleRange {
    pub min: f64,
    pub max: f64,
    pub hls_min: [f64; 3],
    pub hls_max: [f64; 3],
    pub intervals: i32,
}
impl Default for ColorScaleRange {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            hls_min: [230.0, 1.0, 1.0],
            hls_max: [0.0, 1.0, 1.0],
            intervals: 10,
        }
    }
}
#[derive(Clone)]
pub(super) struct ColorScaleInputs {
    pub title: ExtendedString,
    pub format: AsciiString,
    pub colors: Vec<Color>,
    pub labels: Vec<ExtendedString>,
    pub color_type: AspectTypeOfColorScaleData,
    pub label_type: AspectTypeOfColorScaleData,
    pub border: bool,
    pub reversed: bool,
    pub logarithmic: bool,
    pub smooth: bool,
}
impl Default for ColorScaleInputs {
    fn default() -> Self {
        Self {
            title: ExtendedString::default(),
            format: AsciiString::from("%.4g"),
            colors: Vec::new(),
            labels: Vec::new(),
            color_type: AspectTypeOfColorScaleData::Auto,
            label_type: AspectTypeOfColorScaleData::Auto,
            border: true,
            reversed: false,
            logarithmic: false,
            smooth: false,
        }
    }
}
#[derive(Clone)]
pub(super) struct ColorScaleLayout {
    pub label_pos: AspectTypeOfColorScalePosition,
    pub title_pos: AspectTypeOfColorScalePosition,
    pub x: i32,
    pub y: i32,
    pub breadth: i32,
    pub height: i32,
    pub text_height: i32,
    pub spacing: i32,
}
impl Default for ColorScaleLayout {
    fn default() -> Self {
        Self {
            label_pos: AspectTypeOfColorScalePosition::Right,
            title_pos: AspectTypeOfColorScalePosition::Left,
            x: 0,
            y: 0,
            breadth: 0,
            height: 0,
            text_height: 20,
            spacing: 5,
        }
    }
}
#[derive(Clone, Default)]
pub(super) struct ColorScaleFields {
    pub range: ColorScaleRange,
    pub inputs: ColorScaleInputs,
    pub layout: ColorScaleLayout,
}
/// An actual colored bar, frame and text presentation; `AisColorScale::new()?`.
pub struct AisColorScale {
    transient: StandardTransient,
    pub(super) base: AisInteractiveObjectBase,
    pub(super) fields: SharedState<ColorScaleFields>,
}
rustycad_tkernel::impl_transient!(AisColorScale,"AIS_ColorScale",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn PrsMgrPresentableObject,dyn SelectMgrSelectableObject]);
impl AisColorScale {
    /// Create an opaque unlit scale; set its dimensions before display. `AisColorScale::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::default(),
        });
        object.set_display_mode(0);
        object
            .attributes()
            .setup_own_shading_aspect(&Handle::default())?;
        let shading = object.attributes().shading_aspect().aspect();
        shading.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        shading.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
        shading.set_interior_color(Color::named(NamedColor::WHITE));
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
}
impl AisInteractiveObject for AisColorScale {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}
impl SelectMgrSelectableObject for AisColorScale {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    // Native ColorScale deliberately contributes no selection entities.
    fn compute_selection(
        &self,
        _: &dyn HandleReference<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
