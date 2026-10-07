//! Direct translation of Graphic3d_Aspects_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectFillArea3d, Graphic3dAspects, Graphic3dAspectsEqual, Graphic3dAspectsMembers,
    Graphic3dAspectsRenderMembers, Graphic3dMaterialAspect, Graphic3dNameOfMaterial,
    Graphic3dTypeOfReflection, Graphic3dTypeOfShadingModel,
};

/// A material with every reflection color black.
fn zero_material() -> Graphic3dMaterialAspect {
    let mut material = Graphic3dMaterialAspect::default();
    let black = Color::named(NamedColor::BLACK);
    material.set_ambient_color(black);
    material.set_diffuse_color(black);
    material.set_specular_color(black);
    material.set_emissive_color(black);
    material
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_Default() {
    let aspect = Handle::new(Graphic3dAspects::default());
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Default);
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_ExplicitUnlit() {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Unlit);
}

#[test]
fn Graphic3d_AspectsTest__VertexColorForBackFaces_Default() {
    let aspect = Handle::new(Graphic3dAspects::default());
    assert!(aspect.to_use_vertex_color_for_back_faces());
}

#[test]
fn Graphic3d_AspectsTest__VertexColorForBackFaces_SetterAndEquality() {
    let aspect1 = Handle::new(Graphic3dAspects::default());
    let aspect2 = Handle::new(Graphic3dAspects::default());
    assert!(aspect1.is_equal(&*aspect2));
    aspect1.set_use_vertex_color_for_back_faces(false);
    assert!(!aspect1.to_use_vertex_color_for_back_faces());
    assert!(!aspect1.is_equal(&*aspect2));
    aspect2.set_use_vertex_color_for_back_faces(false);
    assert!(aspect1.is_equal(&*aspect2));
}

#[test]
fn Graphic3d_AspectsTest__VertexColorForBackFaces_FillAreaAspect() {
    let aspect = Handle::new(Graphic3dAspectFillArea3d::default());
    assert!(aspect.to_use_vertex_color_for_back_faces());
    aspect.set_use_vertex_color_for_back_faces(false);
    assert!(!aspect.to_use_vertex_color_for_back_faces());
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_IndependentOfMaterial() {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    aspect.set_front_material(zero_material());
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Phong);
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_AllValues() {
    let aspect = Handle::new(Graphic3dAspects::default());
    use Graphic3dTypeOfShadingModel as M;
    for model in [
        M::Unlit,
        M::PhongFacet,
        M::Gouraud,
        M::Phong,
        M::Pbr,
        M::PbrFacet,
    ] {
        aspect.set_shading_model(model);
        assert_eq!(aspect.shading_model(), model);
    }
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_ExplicitUnlitWithColor() {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspect.set_interior_color(Color::named(NamedColor::AZURE));
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Unlit);
    assert_eq!(aspect.interior_color(), Color::named(NamedColor::AZURE));
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_SurvivesMaterialChange() {
    let aspect = Handle::new(Graphic3dAspectFillArea3d::default());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    aspect.set_front_material(Graphic3dMaterialAspect::new(
        Graphic3dNameOfMaterial::Plastified,
    ));
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Phong);
    aspect.set_back_material(Graphic3dMaterialAspect::new(
        Graphic3dNameOfMaterial::Default,
    ));
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Phong);
    aspect.set_front_material(zero_material());
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Phong);
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_SetAfterMaterial() {
    let aspect = Handle::new(Graphic3dAspectFillArea3d::default());
    aspect.set_front_material(Graphic3dMaterialAspect::new(
        Graphic3dNameOfMaterial::Plastified,
    ));
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Unlit);
}

#[test]
fn Graphic3d_AspectsTest__ReflectionMode_IndependentOfShadingModel() {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Phong);
    aspect.set_front_material(zero_material());
    let stored = aspect.front_material();
    use Graphic3dTypeOfReflection as R;
    for kind in [R::Ambient, R::Diffuse, R::Specular, R::Emission] {
        assert!(!stored.reflection_mode(kind));
    }
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Phong);
}

#[test]
fn Graphic3d_AspectsTest__ShadingModel_PbrNotAffectedByPhongMaterial() {
    let aspect = Handle::new(Graphic3dAspects::default());
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Pbr);
    aspect.set_front_material(zero_material());
    assert_eq!(aspect.shading_model(), Graphic3dTypeOfShadingModel::Pbr);
}
