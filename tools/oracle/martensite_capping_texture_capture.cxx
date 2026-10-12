// Actual cap geometry/material/alpha on the repaired owned-viewer probe; OCCT 3d097a0.
#include "martensite_surface_texture_probe.hxx"
#include <array>
#include <OpenGl_Caps.hxx>
static occ::handle<Graphic3d_AspectFillArea3d> unlit_aspects() {
  const occ::handle<Graphic3d_AspectFillArea3d> aspect=new Graphic3d_AspectFillArea3d;
  aspect->SetInteriorStyle(Aspect_IS_SOLID);aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  aspect->SetPolygonOffsets(Aspect_POM_Off,0,0);aspect->SetInteriorColor(texture_rgb(1,0,0));return aspect;
}
#include "martensite_vulkan_capping.cxx"
static std::string cap_texture_mode(const std::string& mode) {
  for(const auto name:{"bc1-rgb","bc1-rgba","bc2","bc3","mask-exact","mask-blend","mask","blend","opaque-alpha"})
    if(mode.find(name)!=std::string::npos)return name;
  return "nearest";
}
static void cap_object_properties(const occ::handle<Graphic3d_Group>& group,
  const occ::handle<Graphic3d_ClipPlane>& plane,const std::string& mode) {
  if(mode.find("object-texture")!=std::string::npos||mode.find("source-mask")!=std::string::npos) {
    const auto objectTexture=texture_source(mode.find("source-mask")!=std::string::npos?"mask":"nearest");
    objectTexture->GetParams()->SetFilter(Graphic3d_TOTF_NEAREST);
    group->Aspects()->SetTextureMap(objectTexture);group->Aspects()->SetTextureMapOn();
    plane->SetUseObjectTexture(mode.find("object-texture")!=std::string::npos);
    if(mode.find("source-mask")!=std::string::npos)group->Aspects()->SetAlphaMode(Graphic3d_AlphaMode_Mask,.5f);
  }
  if(mode.find("object-material")!=std::string::npos) {
    texture_material(group->Aspects());auto material=group->Aspects()->FrontMaterial();material.SetTransparency(.4f);
    group->Aspects()->SetFrontMaterial(material);plane->SetUseObjectMaterial(true);
  }
}
static occ::handle<Graphic3d_ClipPlane> textured_cap_scene(
  const occ::handle<V3d_View>& view,const occ::handle<Graphic3d_Structure>& structure,const std::string& mode) {
  const auto group=structure->NewGroup();group->SetGroupPrimitivesAspect(unlit_aspects());cap_cube(group,0);
  const auto plane=mode.find("oblique")!=std::string::npos?cap_plane(.6,0,-.8,-2):cap_plane(0,0,-1,0);
  const auto texture=texture_source(cap_texture_mode(mode));texture->GetParams()->SetFilter(Graphic3d_TOTF_NEAREST);
  if(mode.find("no-modulate")!=std::string::npos)texture->GetParams()->SetModulate(false);
  texture->GetParams()->SetScale(NCollection_Vec2<float>(.0625f,.0625f));
  if(mode.find("generated")!=std::string::npos)texture->GetParams()->SetGenMode(
    mode.find("eye")!=std::string::npos?Graphic3d_TOTM_EYE:Graphic3d_TOTM_OBJECT,
    NCollection_Vec4<float>(1,0,0,0),NCollection_Vec4<float>(0,0,-1,0));
  plane->CappingAspect()->SetTextureMap(texture);plane->CappingAspect()->SetTextureMapOn();
  plane->CappingAspect()->SetInteriorColor(texture_rgb(1,1,1));
  texture_surface_style(plane->CappingAspect(),cap_texture_mode(mode));
  if(mode.find("phong")!=std::string::npos||mode.find("gouraud")!=std::string::npos||mode.find("facet")!=std::string::npos) {
    texture_material(plane->CappingAspect());texture_lights(view);
    plane->CappingAspect()->SetShadingModel(mode.find("gouraud")!=std::string::npos?Graphic3d_TypeOfShadingModel_Gouraud:
      mode.find("facet")!=std::string::npos?Graphic3d_TypeOfShadingModel_PhongFacet:Graphic3d_TypeOfShadingModel_Phong);
  }
  cap_object_properties(group,plane,mode);
  const occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane;
  planes->Append(plane);view->SetClipPlanes(planes);structure->CalculateBoundBox();structure->Display();return plane;
}
static void cap_live_update(const occ::handle<Graphic3d_ClipPlane>& plane,const std::string& mode) {
  const auto texture=occ::handle<Graphic3d_Texture2D>::DownCast(plane->CappingAspect()->TextureMap());
  if(mode.find("image-updated")!=std::string::npos||mode.find("image-refreshed")!=std::string::npos)texture_mutate(texture,"image-updated");
  if(mode.find("image-refreshed")!=std::string::npos)plane->SetCappingTexture(texture);
  if(mode.find("sampler-updated")!=std::string::npos)texture->GetParams()->SetFilter(Graphic3d_TOTF_BILINEAR);
  if(mode.find("plane-updated")!=std::string::npos)plane->SetEquation(NCollection_Vec4<double>(.6,0,-.8,-1));
  if(mode.find("alpha-updated")!=std::string::npos)plane->CappingAspect()->SetAlphaMode(Graphic3d_AlphaMode_Blend,.5f);
}

int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected mode prefix").c_str());
  auto probe=create_policy_probe();texture_camera(probe.view);
  probe.workspace->GetGlContext()->caps->ffpEnable=std::string(argv[1]).find("generated")!=std::string::npos||std::string(argv[1]).find("fixed")!=std::string::npos;
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(probe.viewer->StructureManager());
  const auto plane=textured_cap_scene(probe.view,structure,argv[1]);
  probe.view->Redraw();cap_live_update(plane,argv[1]);texture_capture(probe.view,argv[2]);
  probe.view->Remove();return 0;
}
