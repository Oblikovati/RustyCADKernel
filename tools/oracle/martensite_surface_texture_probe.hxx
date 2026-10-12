#pragma once
// Actual retained standard2D surface sampler oracle; OCCT3d097a0, no image codec dependency.
#include "martensite_text_culling_probe.hxx"
#include <Graphic3d_Texture2D.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_Group.hxx>
#include <Graphic3d_Structure.hxx>
#include <Image_PixMap.hxx>
#include <fstream>
#include <Graphic3d_TextureParams.hxx>
#include <Graphic3d_CLight.hxx>
#include <Graphic3d_LightSet.hxx>
#include <iterator>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
static void texture_camera(const occ::handle<V3d_View>& view) {
  const auto camera=view->Camera();camera->SetEye(gp_Pnt(0,0,20));camera->SetCenter(gp_Pnt(0,0,0));
  camera->SetUp(gp_Dir(0,1,0));camera->SetScale(20);camera->SetAspect(1);camera->SetZRange(1,100);
  camera->SetProjectionType(Graphic3d_Camera::Projection_Orthographic);view->SetAutoZFitMode(false);
  view->SetBackgroundColor(Quantity_Color(0,0,1,Quantity_TOC_RGB));view->ChangeRenderingParams().NbMsaaSamples=0;
}
static occ::handle<Image_PixMap> surface_format_image(const std::string& mode) {
  if(mode.rfind("format-",0)!=0&&mode!="mask-exact")return {};
  const std::string name=mode=="mask-exact"?"cutoff":mode.substr(7);
  const char* names[]={"gray","alpha","rgb","bgr","rgb32","bgr32","rgba","bgra","gray-f",
    "alpha-f","rg-f","rgb-f","bgr-f","rgba-f","bgra-f","gray-half","rg-half","rgba-half","gray16"};
  int index=0;while(index<19&&name!=names[index])++index;
  if(index==19&&name!="cutoff")throw Standard_Failure(("received image format="+name+"; expected native 2D fixture format").c_str());
  std::ifstream input("crates/vulkan/tests/fixtures/surface_textures/"+name+".pixels",std::ios::binary);
  const std::vector<char> bytes((std::istreambuf_iterator<char>(input)),std::istreambuf_iterator<char>());
  const occ::handle<Image_PixMap> image=new Image_PixMap;
  if(!image->InitZero(name=="cutoff"?Image_Format_RGBAF:Image_Format(index+1),2,2)||bytes.size()!=image->SizeBytes())
    throw Standard_Failure(("received image bytes="+std::to_string(bytes.size())+"; expected packed2x2format"+name).c_str());
  std::memcpy(image->ChangeData(),bytes.data(),bytes.size());image->SetTopDown(true);return image;
}
static occ::handle<Graphic3d_Texture2D> texture_source(const std::string& mode) {
  if(mode.rfind("missing-file",0)==0)return new Graphic3d_Texture2D("Martensite-missing-standard2D-fixture.png");
  const auto formatted=surface_format_image(mode);
  if(!formatted.IsNull())return new Graphic3d_Texture2D(formatted);
  if(mode.rfind("bc",0)==0) {
    const std::string file=(mode.rfind("bc1-",0)==0&&mode!="bc1-rgba")?"bc1-rgb":mode;
    return new Graphic3d_Texture2D(("crates/vulkan/tests/fixtures/surface_textures/"+file+".dds").c_str());
  }
  const occ::handle<Image_PixMap> image=new Image_PixMap;
  image->InitZero(Image_Format_RGBA,2,2);image->SetTopDown(mode!="bottom-up");
  const uint8_t pixels[]={255,0,0,255,0,255,0,255,0,0,255,255,255,255,255,255};
  std::memcpy(image->ChangeData(),pixels,sizeof(pixels));
  if(mode=="mask"||mode=="mask-blend"||mode=="blend"||mode=="opaque-alpha"||mode=="auto-alpha") {
    for(int index=0;index<4;++index)image->ChangeData()[index*4+3]=uint8_t((index+1)*64- (index==3?1:0));
  }
  return new Graphic3d_Texture2D(image);
}
static Quantity_Color texture_rgb(float r,float g,float b) {return Quantity_Color(r,g,b,Quantity_TOC_RGB);}
static void texture_material(const occ::handle<Graphic3d_Aspects>& style) {
  Graphic3d_MaterialAspect material(Graphic3d_NameOfMaterial_Plastified);
  material.SetMaterialType(Graphic3d_MATERIAL_ASPECT);
  material.SetAmbientColor(texture_rgb(.25f,.5f,.75f));material.SetDiffuseColor(texture_rgb(.5f,.25f,.125f));
  material.SetSpecularColor(texture_rgb(0,0,0));material.SetEmissiveColor(texture_rgb(.125f,.25f,.5f));
  style->SetFrontMaterial(material);style->SetBackMaterial(material);style->SetInteriorColor(texture_rgb(.75f,.5f,.25f));
}
static void texture_lights(const occ::handle<V3d_View>& view) {
  const occ::handle<Graphic3d_CLight> ambient=new Graphic3d_CLight(Graphic3d_TypeOfLightSource_Ambient);
  ambient->SetColor(texture_rgb(.5f,.25f,.125f));
  const occ::handle<Graphic3d_LightSet> lights=new Graphic3d_LightSet;lights->Add(ambient);view->View()->SetLights(lights);
}
static void texture_parameter_style(const occ::handle<Graphic3d_Texture2D>& texture,const std::string& mode) {
  const auto params=texture->GetParams();
  if(mode=="nearest")params->SetFilter(Graphic3d_TOTF_NEAREST);
  if(mode=="bilinear")params->SetFilter(Graphic3d_TOTF_BILINEAR);
  if(mode=="clamp")params->SetRepeat(false);
  if(mode=="linear")texture->SetColorMap(false);
  if(mode=="no-modulate")params->SetModulate(false);
  if(mode=="no-mips")texture->SetMipmapsGeneration(false);
  if(mode=="generated-mips"||mode=="bc1-minified")params->SetScale(NCollection_Vec2<float>(128,128));
  if(mode=="anisotropy-fast")params->SetAnisoFilter(Graphic3d_LOTA_FAST);
  if(mode=="anisotropy-middle")params->SetAnisoFilter(Graphic3d_LOTA_MIDDLE);
  if(mode=="anisotropy-quality")params->SetAnisoFilter(Graphic3d_LOTA_QUALITY);
  if(mode=="bc1-base-level")params->SetLevelsRange(1,1000);
  if(mode=="transformed") {params->SetRotation(30);params->SetScale(NCollection_Vec2<float>(2,3));params->SetTranslation(NCollection_Vec2<float>(.125,.25));}
}
static void texture_surface_style(const occ::handle<Graphic3d_AspectFillArea3d>& style,const std::string& mode) {
  if(mode=="back-vertex")style->SetUseVertexColorForBackFaces(false);
  if(mode=="phong"||mode=="no-modulate")style->SetShadingModel(Graphic3d_TypeOfShadingModel_Phong);
  if(mode=="gouraud")style->SetShadingModel(Graphic3d_TypeOfShadingModel_Gouraud);
  if(mode=="facet")style->SetShadingModel(Graphic3d_TypeOfShadingModel_PhongFacet);
  if(mode=="mask"||mode=="mask-exact")style->SetAlphaMode(Graphic3d_AlphaMode_Mask,.5f);
  if(mode=="mask-blend")style->SetAlphaMode(Graphic3d_AlphaMode_MaskBlend,.5f);
  if(mode=="blend")style->SetAlphaMode(Graphic3d_AlphaMode_Blend,.5f);
  if(mode=="opaque-alpha")style->SetAlphaMode(Graphic3d_AlphaMode_Opaque,.5f);
}
static void texture_style(const occ::handle<Graphic3d_AspectFillArea3d>& style,
                          const occ::handle<Graphic3d_Texture2D>& texture,const std::string& fixture_mode) {
  const std::string mode=fixture_mode.rfind("missing-file-",0)==0?fixture_mode.substr(13):fixture_mode;
  texture_material(style);texture_parameter_style(texture,mode);texture_surface_style(style,mode);
}
static void projective_surface_geometry(const occ::handle<Graphic3d_Group>& group) {
  const occ::handle<Graphic3d_Buffer> buffer=new Graphic3d_Buffer(Graphic3d_Buffer::DefaultAllocator());
  const Graphic3d_Attribute attributes[]={{Graphic3d_TOA_POS,Graphic3d_TOD_VEC3},
    {Graphic3d_TOA_NORM,Graphic3d_TOD_VEC3},{Graphic3d_TOA_UV,Graphic3d_TOD_VEC4}};
  if(!buffer->Init(6,attributes,3))throw Standard_Failure("projective UV allocation; expected6vertices");
  const float points[][3]={{-8,-8,0},{8,-8,0},{8,8,0},{-8,8,0}};
  const float uv[][4]={{0,0,0,.5f},{1,0,0,1},{1,1,0,2},{0,1,0,.75f}};
  const int ranks[]={0,1,2,0,2,3};
  for(int index=0;index<6;++index) {
    float* vertex=reinterpret_cast<float*>(buffer->ChangeData()+index*40);
    std::memcpy(vertex,points[ranks[index]],12);vertex[3]=0;vertex[4]=0;vertex[5]=1;
    std::memcpy(vertex+6,uv[ranks[index]],16);
  }
  group->AddPrimitiveArray(Graphic3d_TOPA_TRIANGLES,nullptr,buffer,nullptr,true);
}
static occ::handle<Graphic3d_Structure> texture_geometry(const occ::handle<V3d_Viewer>& viewer,const occ::handle<Graphic3d_Texture2D>& texture,const std::string& mode) {
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(viewer->StructureManager());
  const auto group=structure->NewGroup();
  const occ::handle<Graphic3d_AspectFillArea3d> style=new Graphic3d_AspectFillArea3d;
  style->SetInteriorStyle(Aspect_IS_SOLID);style->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);style->SetPolygonOffsets(Aspect_POM_Off,0,0);
  style->SetTextureMap(texture);texture_style(style,texture,mode);style->SetTextureMapOn();group->SetGroupPrimitivesAspect(style);
  const occ::handle<Graphic3d_ArrayOfTriangles> vertices=new Graphic3d_ArrayOfTriangles(4,6,true,mode=="front-vertex"||mode=="back-vertex",true);
  vertices->AddVertex(gp_Pnt(-8,-8,0),gp_Dir(0,0,1),gp_Pnt2d(0,0));
  vertices->AddVertex(gp_Pnt(8,-8,0),gp_Dir(0,0,1),gp_Pnt2d(1,0));
  vertices->AddVertex(gp_Pnt(8,8,0),gp_Dir(0,0,1),gp_Pnt2d(1,1));
  vertices->AddVertex(gp_Pnt(-8,8,0),gp_Dir(0,0,1),gp_Pnt2d(0,1));
  const bool reversed=mode.rfind("back",0)==0;
  for(int index:(reversed?std::vector<int>{1,3,2,1,4,3}:std::vector<int>{1,2,3,1,3,4}))vertices->AddEdge(index);
  for(int rank=1;rank<=4;++rank) {
    if(mode=="front-vertex"||mode=="back-vertex")vertices->SetVertexColor(rank,texture_rgb(.25f,.5f,.75f));
    if(mode=="perspective") {const gp_Pnt point=vertices->Vertice(rank);vertices->SetVertice(rank,point.X(),point.Y(),rank<3?-4:4);}
  }
  if(mode=="projective")projective_surface_geometry(group);else group->AddPrimitiveArray(vertices);structure->Display();return structure;
}
static void texture_capture(const occ::handle<V3d_View>& view,const std::string& prefix) {
  for(auto kind:{Graphic3d_BT_RGBA,Graphic3d_BT_Depth}) {
    Image_PixMap pixels;
    if(!view->ToPixMap(pixels,64,64,kind,false,Graphic3d_ZLayerId_Default,true))
      throw Standard_Failure("native surface texture capture failed; expected64square buffers");
    std::ofstream output(prefix+(kind==Graphic3d_BT_RGBA?".rgba":".depth"),std::ios::binary);
    for(int row=0;row<64;++row)output.write(reinterpret_cast<const char*>(pixels.Row(row)),256);
  }
}
static void texture_mutate(const occ::handle<Graphic3d_Texture2D>& texture,const std::string& mode) {
  if(mode=="image-stale"||mode=="image-updated") {
    const auto image=texture_source("default")->GetImage(occ::handle<Image_SupportedFormats>());
    image->SetPixelColor(0,0,Quantity_ColorRGBA(0,0,0,1),false);texture->SetImage(image);
    if(mode=="image-updated")texture->UpdateRevision();
  }
  if(mode=="image-null-updated"||mode=="image-empty-updated") {
    const occ::handle<Image_PixMap> image=mode=="image-empty-updated"?occ::handle<Image_PixMap>(new Image_PixMap):occ::handle<Image_PixMap>();
    texture->SetImage(image);texture->UpdateRevision();
  }
  if(mode=="sampler-updated")texture->GetParams()->SetFilter(Graphic3d_TOTF_NEAREST);
  if(mode=="bc1-levels-updated"||mode=="bc1-filter-levels-updated") {
    texture->GetParams()->SetLevelsRange(1,1000);
    if(mode=="bc1-filter-levels-updated")texture->GetParams()->SetFilter(Graphic3d_TOTF_NEAREST);
  }
}
static void texture_view_fixture(const occ::handle<V3d_View>& view,const std::string& mode) {
  texture_camera(view);texture_lights(view);
  if(mode=="perspective")view->Camera()->SetProjectionType(Graphic3d_Camera::Projection_Perspective);
  if(mode!="clipped")return;
  const occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane;
  planes->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(1,0,0,0)));view->SetClipPlanes(planes);
}
static void texture_capture_mutation(const occ::handle<V3d_View>& view,
  const occ::handle<Graphic3d_Structure>& structure,const occ::handle<Graphic3d_Texture2D>& texture,
  const std::string& mode,const std::string& prefix) {
  if(mode=="image-repaired") {
    texture_mutate(texture,"image-empty-updated");structure->Groups().Last()->SynchronizeAspects();
    texture_capture(view,prefix+"-failed");texture_mutate(texture,"image-updated");
  }else texture_mutate(texture,mode);
  structure->Groups().Last()->SynchronizeAspects();texture_capture(view,prefix);
}
