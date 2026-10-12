//! `Prs3d_ShadingAspect`: the fill area attributes of shaded presentations.
use super::basic_aspect::{Prs3dBasicAspect, dump_graphic_aspects};
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::aspect::{AspectInteriorStyle, AspectTypeOfFacingModel, AspectTypeOfLine};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectFillArea3d, Graphic3dAspectsMembers as _, Graphic3dAspectsRenderMembers as _,
    Graphic3dMaterialAspect, Graphic3dNameOfMaterial, Graphic3dTypeOfMaterial,
};
use rustycad_tkservice::json_dump::class_begin;
use std::io;

/// The shading attributes of a presentation.
///
/// Example: `let aspect = Prs3dShadingAspect::new()?; aspect.set_color(red, AspectTypeOfFacingModel::BothSide)?;`.
pub struct Prs3dShadingAspect {
    transient: StandardTransient,
    pub(super) aspect: SharedState<Handle<Graphic3dAspectFillArea3d>>,
}
rustycad_tkernel::impl_transient!(Prs3dShadingAspect, "Prs3d_ShadingAspect", dyn Prs3dBasicAspect, transient,
    interfaces: [dyn Prs3dBasicAspect]);

/// Whether a facing model touches the front side.
fn touches_front(model: AspectTypeOfFacingModel) -> bool {
    matches!(
        model,
        AspectTypeOfFacingModel::FrontSide | AspectTypeOfFacingModel::BothSide
    )
}

/// Whether a facing model touches the back side.
fn touches_back(model: AspectTypeOfFacingModel) -> bool {
    matches!(
        model,
        AspectTypeOfFacingModel::BackSide | AspectTypeOfFacingModel::BothSide
    )
}

impl Prs3dShadingAspect {
    /// The default: a solid interior of the brass ambient color with brass materials.
    /// Example: `Prs3dShadingAspect::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Brass);
        let color = material.ambient_color();
        let aspect = Graphic3dAspectFillArea3d::new(
            AspectInteriorStyle::Solid,
            color,
            color,
            AspectTypeOfLine::Solid,
            1.0,
            material.clone(),
            material,
        )?;
        Ok(Self::from_aspect(Handle::new(aspect)))
    }
    /// Wraps an existing Graphic3d aspect; `Prs3dShadingAspect::from_aspect(aspect)`.
    pub fn from_aspect(aspect: Handle<Graphic3dAspectFillArea3d>) -> Self {
        Self {
            transient: StandardTransient::default(),
            aspect: SharedState::new(aspect),
        }
    }
    /// `SetColor`: the material and interior colors of the chosen sides; a one-sided change
    /// turns material distinction on. Example: `aspect.set_color(red, AspectTypeOfFacingModel::BothSide)?`.
    pub fn set_color(&self, color: Color, model: AspectTypeOfFacingModel) -> Result<(), Failure> {
        let aspect = self.aspect();
        let Some(fill) = aspect.get() else {
            return Ok(());
        };
        if model != AspectTypeOfFacingModel::BothSide {
            fill.set_distinguish(true);
        }
        if touches_front(model) {
            let mut material = fill.front_material();
            material.set_color(color)?;
            fill.set_front_material(material);
            fill.set_interior_color(color);
        }
        if touches_back(model) {
            let mut material = fill.back_material();
            material.set_color(color)?;
            fill.set_back_material(material);
            fill.set_back_interior_color(color);
        }
        Ok(())
    }
    /// `Color`: the interior color of an `Aspect` material, else the material color.
    /// Example: `aspect.color(AspectTypeOfFacingModel::FrontSide)`.
    pub fn color(&self, model: AspectTypeOfFacingModel) -> Color {
        let aspect = self.aspect();
        let Some(fill) = aspect.get() else {
            return Color::default();
        };
        let (material, interior) = if model == AspectTypeOfFacingModel::BackSide {
            (fill.back_material(), fill.back_interior_color())
        } else {
            (fill.front_material(), fill.interior_color())
        };
        if material.material_type() == Graphic3dTypeOfMaterial::Aspect {
            interior
        } else {
            material.color()
        }
    }
    /// `SetMaterial` of the chosen sides; `aspect.set_material(&gold, AspectTypeOfFacingModel::FrontSide)`.
    pub fn set_material(&self, material: &Graphic3dMaterialAspect, model: AspectTypeOfFacingModel) {
        let aspect = self.aspect();
        let Some(fill) = aspect.get() else {
            return;
        };
        if model != AspectTypeOfFacingModel::BothSide {
            fill.set_distinguish(true);
        }
        if touches_front(model) {
            fill.set_front_material(material.clone());
        }
        if touches_back(model) {
            fill.set_back_material(material.clone());
        }
    }
    /// `Material` of a side (the front one for both sides); `aspect.material(AspectTypeOfFacingModel::BackSide)`.
    pub fn material(&self, model: AspectTypeOfFacingModel) -> Graphic3dMaterialAspect {
        let aspect = self.aspect();
        let Some(fill) = aspect.get() else {
            return Graphic3dMaterialAspect::default();
        };
        if model == AspectTypeOfFacingModel::BackSide {
            fill.back_material()
        } else {
            fill.front_material()
        }
    }
    /// `SetTransparency` of the materials and interior alphas of the chosen sides; a value
    /// outside [0, 1] raises `Graphic3d_MaterialDefinitionError`.
    /// Example: `aspect.set_transparency(0.5, AspectTypeOfFacingModel::BothSide)?`.
    pub fn set_transparency(
        &self,
        value: f64,
        model: AspectTypeOfFacingModel,
    ) -> Result<(), Failure> {
        let aspect = self.aspect();
        let Some(fill) = aspect.get() else {
            return Ok(());
        };
        if model != AspectTypeOfFacingModel::BothSide {
            fill.set_distinguish(true);
        }
        let alpha = 1.0f32 - value as f32;
        if touches_front(model) {
            let mut material = fill.front_material();
            material.set_transparency(value as f32)?;
            fill.set_front_material(material);
            fill.set_interior_color_rgba(ColorRgba::from_rgb(fill.interior_color(), alpha));
        }
        if touches_back(model) {
            let mut material = fill.back_material();
            material.set_transparency(value as f32)?;
            fill.set_back_material(material);
            fill.set_back_interior_color_rgba(ColorRgba::from_rgb(
                fill.back_interior_color(),
                alpha,
            ));
        }
        Ok(())
    }
    /// `Transparency` of a side's material; `aspect.transparency(AspectTypeOfFacingModel::FrontSide)`.
    pub fn transparency(&self, model: AspectTypeOfFacingModel) -> f64 {
        f64::from(self.material(model).transparency())
    }
    /// `ToUseVertexColorForBackFaces`; `aspect.to_use_vertex_color_for_back_faces()`.
    pub fn to_use_vertex_color_for_back_faces(&self) -> bool {
        self.aspect()
            .get()
            .is_some_and(|fill| fill.to_use_vertex_color_for_back_faces())
    }
    /// `SetUseVertexColorForBackFaces`; `aspect.set_use_vertex_color_for_back_faces(true)`.
    pub fn set_use_vertex_color_for_back_faces(&self, to_use: bool) {
        if let Some(fill) = self.aspect().get() {
            fill.set_use_vertex_color_for_back_faces(to_use);
        }
    }
    /// The Graphic3d aspect; `aspect.aspect()`.
    pub fn aspect(&self) -> Handle<Graphic3dAspectFillArea3d> {
        self.aspect.read().clone()
    }
    /// Replaces the Graphic3d aspect; `aspect.set_aspect(other)`.
    pub fn set_aspect(&self, aspect: Handle<Graphic3dAspectFillArea3d>) {
        *self.aspect.write() = aspect;
    }
}

impl Prs3dBasicAspect for Prs3dShadingAspect {
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_ShadingAspect")?;
        dump_graphic_aspects(stream, depth, "myAspect.get()", &self.aspect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::quantity::NamedColor;

    #[test]
    fn one_sided_color_turns_distinction_on() {
        let aspect = Prs3dShadingAspect::new().unwrap();
        assert!(!aspect.aspect().distinguish());
        aspect
            .set_color(
                Color::named(NamedColor::RED),
                AspectTypeOfFacingModel::BackSide,
            )
            .unwrap();
        assert!(aspect.aspect().distinguish());
        assert_eq!(
            aspect.aspect().back_interior_color(),
            Color::named(NamedColor::RED)
        );
    }

    #[test]
    fn transparency_updates_material_and_alpha() {
        let aspect = Prs3dShadingAspect::new().unwrap();
        aspect
            .set_transparency(0.25, AspectTypeOfFacingModel::BothSide)
            .unwrap();
        assert_eq!(
            aspect.transparency(AspectTypeOfFacingModel::FrontSide),
            0.25
        );
        assert_eq!(aspect.transparency(AspectTypeOfFacingModel::BackSide), 0.25);
        assert_eq!(aspect.aspect().interior_color_rgba().alpha(), 0.75);
        assert!(
            aspect
                .set_transparency(2.0, AspectTypeOfFacingModel::FrontSide)
                .is_err()
        );
    }

    #[test]
    fn material_selection_by_side() {
        let aspect = Prs3dShadingAspect::new().unwrap();
        let gold = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
        aspect.set_material(&gold, AspectTypeOfFacingModel::FrontSide);
        assert_eq!(
            aspect.material(AspectTypeOfFacingModel::FrontSide).name(),
            gold.name()
        );
        assert_ne!(
            aspect.material(AspectTypeOfFacingModel::BackSide).name(),
            gold.name()
        );
    }
}
