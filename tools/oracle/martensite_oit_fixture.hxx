#pragma once
// Actual OCCT3d097a0 retained geometry and parameters, not hand-written GL OIT.
#include "martensite_surface_texture_probe.hxx"
#include <Graphic3d_ArrayOfPoints.hxx>
#include <Graphic3d_ArrayOfSegments.hxx>
#include <Graphic3d_Text.hxx>
#include <Graphic3d_AspectText3d.hxx>
static Quantity_ColorRGBA oit_native_palette(int rank,const std::string& scene) {
  if(scene=="occluded"&&rank==2)return Quantity_ColorRGBA(1,1,1,1);
  if(scene!="five")return Quantity_ColorRGBA(Quantity_Color(rank==0?1.:0.,rank==1?1.:0.,0.,Quantity_TOC_RGB),rank==0?.5f:.75f);
  const float colors[][4]={{1,0,0,.25f},{0,1,0,.5f},{0,0,1,.75f},{1,1,0,.4f},{1,0,1,.6f}};
  const auto& color=colors[rank];return Quantity_ColorRGBA(Quantity_Color(color[0],color[1],color[2],Quantity_TOC_RGB),color[3]);
}
static occ::handle<Graphic3d_AspectFillArea3d> oit_native_aspect(int rank,const std::string& scene="two") {
  const occ::handle<Graphic3d_AspectFillArea3d> aspect=new Graphic3d_AspectFillArea3d;
  aspect->SetInteriorStyle(Aspect_IS_SOLID);
  aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  aspect->SetFaceCulling(Graphic3d_TypeOfBackfacingModel_DoubleSided);
  aspect->SetPolygonOffsets(Aspect_POM_Off,0,0);
  const auto color=oit_native_palette(rank,scene);aspect->SetInteriorColor(color);
  auto material=aspect->FrontMaterial();material.SetTransparency(1.f-color.Alpha());
  aspect->SetFrontMaterial(material);
  auto alpha=Graphic3d_AlphaMode_Blend;
  if(scene=="opaque"||scene=="far-boundary-opaque")alpha=Graphic3d_AlphaMode_Opaque;
  if(scene=="mask"||scene=="one-mask")alpha=Graphic3d_AlphaMode_Mask;
  if(scene=="mask-blend")alpha=Graphic3d_AlphaMode_MaskBlend;
  if(scene=="auto")alpha=Graphic3d_AlphaMode_BlendAuto;
  aspect->SetAlphaMode(scene=="occluded"&&rank==2?Graphic3d_AlphaMode_Opaque:alpha,.5f);
  if(scene.rfind("texture-",0)==0) {
    const auto texture=texture_source("blend");
    texture_style(aspect,texture,scene.substr(8));
    aspect->SetTextureMap(texture);aspect->SetTextureMapOn();
    aspect->SetInteriorColor(color);
    material=aspect->FrontMaterial();material.SetTransparency(1.f-color.Alpha());
    aspect->SetFrontMaterial(material);aspect->SetBackMaterial(material);
    aspect->SetAlphaMode(Graphic3d_AlphaMode_Blend,.5f);
  }
  return aspect;
}
static occ::handle<Graphic3d_ArrayOfTriangles> oit_native_quad(int rank,const std::string& scene="two") {
  const occ::handle<Graphic3d_ArrayOfTriangles> vertices=new Graphic3d_ArrayOfTriangles(6,0,true,false,true);
  const float corners[][2]={{-6,-6},{6,-6},{6,6},{-6,-6},{6,6},{-6,6}};
  const float z=scene=="tied"?2.f:scene=="near-boundary"?19.f:(scene=="far-boundary"||scene=="far-boundary-opaque")?-80.f:scene=="far-interior"?-79.f:
    scene=="occluded"&&rank==2?4.f:scene=="five"?6.f-3.f*rank:(rank==0?2.f:-2.f);
  for(const auto& corner:corners)vertices->AddVertex(gp_Pnt(corner[0],corner[1],z),gp_Dir(0,0,1),gp_Pnt2d((corner[0]+6)/12,(corner[1]+6)/12));
  return vertices;
}
static Aspect_TypeOfDisplayText oit_text_display(const std::string& scene) {
  if(scene=="text-shadow")return Aspect_TODT_SHADOW;
  if(scene=="text-subtitle")return Aspect_TODT_SUBTITLE;
  if(scene=="text-decal")return Aspect_TODT_DEKALE;
  if(scene=="text-xor")return Aspect_TODT_BLEND;
  return Aspect_TODT_NORMAL;
}
static void oit_native_text(const occ::handle<Graphic3d_Structure>& structure,const std::string& scene) {
  const occ::handle<Graphic3d_AspectText3d> style=new Graphic3d_AspectText3d(texture_rgb(.18,.4,.7),"DejaVu Sans",1,0,Aspect_TOST_NORMAL,oit_text_display(scene));
  style->SetColorSubTitle(Quantity_ColorRGBA(.25f,.5f,.75f,.5f));
  const occ::handle<Graphic3d_Text> text=new Graphic3d_Text(16);text->SetText("AV");text->SetPosition(gp_Pnt(0,0,4));
  text->SetHorizontalAlignment(Graphic3d_HTA_CENTER);text->SetVerticalAlignment(Graphic3d_VTA_CENTER);
  const auto group=structure->NewGroup();group->SetPrimitivesAspect(style);group->AddText(text);
}
static void oit_native_overlay(const occ::handle<Graphic3d_Structure>& structure,const std::string& scene) {
  if(scene!="sprite-mask"&&scene!="sprite-low-mask"&&scene!="sprite-opaque"&&scene!="line-mask")return;
  const auto aspects=oit_native_aspect(0,"mask");
  aspects->SetInteriorColor(Quantity_ColorRGBA(1,1,1,.5f));
  aspects->SetMarkerType(Aspect_TOM_PLUS);aspects->SetMarkerScale(3);
  if(scene=="sprite-low-mask")aspects->SetAlphaMode(Graphic3d_AlphaMode_Mask,.1f);
  if(scene=="sprite-opaque")aspects->SetAlphaMode(Graphic3d_AlphaMode_Opaque,.5f);
  const auto group=structure->NewGroup();group->SetGroupPrimitivesAspect(aspects);
  if(scene.rfind("sprite-",0)==0) {
    const occ::handle<Graphic3d_ArrayOfPoints> points=new Graphic3d_ArrayOfPoints(1);
    points->AddVertex(0.f,0.f,4.f);group->AddPrimitiveArray(points);return;
  }
  const occ::handle<Graphic3d_ArrayOfSegments> line=new Graphic3d_ArrayOfSegments(2);
  line->AddVertex(-6.f,0.f,4.f);line->AddVertex(6.f,0.f,4.f);group->AddPrimitiveArray(line);
}
static occ::handle<Graphic3d_Structure> oit_native_geometry(
  const occ::handle<V3d_Viewer>& viewer,bool reverse,const std::string& scene="two") {
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(viewer->StructureManager());
  structure->SetZLayer(Graphic3d_ZLayerId_Default);
  const int count=scene.rfind("one-",0)==0?1:(scene=="five"?5:scene=="occluded"?3:2);
  for(int index=0;index<count;++index) {
    const int rank=reverse?count-1-index:index;const auto group=structure->NewGroup();
    group->SetGroupPrimitivesAspect(oit_native_aspect(rank,scene));
    group->AddPrimitiveArray(oit_native_quad(rank,scene));
  }
  oit_native_overlay(structure,scene);if(scene.rfind("text-",0)==0)oit_native_text(structure,scene);structure->Display();return structure;
}
static void oit_native_parameters(const occ::handle<V3d_View>& view,const std::string& mode,
                                  int samples,float factor,int layers) {
  auto& params=view->ChangeRenderingParams();params.NbMsaaSamples=samples;
  params.OitDepthFactor=factor;params.NbOitDepthPeelingLayers=layers;
  if(mode=="weighted")params.TransparencyMethod=Graphic3d_RTM_BLEND_OIT;
  else if(mode=="peeling")params.TransparencyMethod=Graphic3d_RTM_DEPTH_PEELING_OIT;
  else if(mode=="unordered")params.TransparencyMethod=Graphic3d_RTM_BLEND_UNORDERED;
  else throw Standard_Failure(("received OIT mode="+mode+"; expected weighted, peeling or unordered").c_str());
}
