//! Color references bind one physical subfield at getter time, even when material type later changes.
use crate::prs3d::Prs3dShadingAspect;
use rustycad_tkernel::{
    quantity::Color,
    standard::{Handle, HandleObserver},
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use std::marker::PhantomData;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CubeColorLeaf {
    FrontInterior,
    BackInterior,
    FrontMaterial,
    BackMaterial,
}
/// Borrow one retained native color subfield; `let alias=cube.box_color_ref(); cube.set_box_color(red)?; alias.value()`.
pub struct AisViewCubeColorRef<'a> {
    aspect: HandleObserver<dyn Graphic3dAspectsSource>,
    leaf: CubeColorLeaf,
    scope: PhantomData<&'a super::AisViewCubePresentationBase>,
}
impl<'a> AisViewCubeColorRef<'a> {
    pub(super) fn shading(
        shading: &Prs3dShadingAspect,
        model: AspectTypeOfFacingModel,
        scope: &'a super::AisViewCubePresentationBase,
    ) -> Self {
        let aspect = shading.aspect();
        let back = model == AspectTypeOfFacingModel::BackSide;
        let material = if back {
            aspect.back_material()
        } else {
            aspect.front_material()
        };
        let interior = material.material_type() == Graphic3dTypeOfMaterial::Aspect;
        let leaf = match (back, interior) {
            (false, true) => CubeColorLeaf::FrontInterior,
            (true, true) => CubeColorLeaf::BackInterior,
            (false, false) => CubeColorLeaf::FrontMaterial,
            (true, false) => CubeColorLeaf::BackMaterial,
        };
        Self::field(&aspect.cast(), leaf, scope)
    }
    pub(super) fn text(
        aspect: &Handle<Graphic3dAspectText3d>,
        scope: &'a super::AisViewCubePresentationBase,
    ) -> Self {
        Self::field(&aspect.cast(), CubeColorLeaf::FrontInterior, scope)
    }
    pub(super) fn field(
        aspect: &Handle<dyn Graphic3dAspectsSource>,
        leaf: CubeColorLeaf,
        _: &'a super::AisViewCubePresentationBase,
    ) -> Self {
        Self {
            aspect: aspect.observe(),
            leaf,
            scope: PhantomData,
        }
    }
    /// Copy current RGB from the same bound field; `alias.value().red()`.
    pub fn value(&self) -> Color {
        let fields = self.aspect.aspects_base().fields.read();
        match self.leaf {
            CubeColorLeaf::FrontInterior => *fields.interior_color.rgb(),
            CubeColorLeaf::BackInterior => *fields.back_interior_color.rgb(),
            CubeColorLeaf::FrontMaterial => fields.front_material.color(),
            CubeColorLeaf::BackMaterial => fields.back_material.color(),
        }
    }
    /// Compare physical source object and fixed subfield; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &AisViewCubeColorRef<'_>) -> bool {
        self.leaf == other.leaf && self.aspect.object_address() == other.aspect.object_address()
    }
    /// Observe the owning Graphic3d aspect identity without acquiring native ownership; `alias.aspect_address()`.
    pub fn aspect_address(&self) -> *const () {
        self.aspect.object_address()
    }
}
