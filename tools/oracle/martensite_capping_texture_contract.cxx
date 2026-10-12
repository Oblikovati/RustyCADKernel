// CPU section primitive/aspect provenance for Martensite; OCCT3d097a0, no GL drawing.
#include <OpenGl_CappingPlaneResource.hxx>
#include <OpenGl_Context.hxx>
#include <OpenGl_Aspects.hxx>
#include <OpenGl_Workspace.hxx>
#include <OpenGl_RenderFilter.hxx>
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_Aspects.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Image_PixMap.hxx>
#include <Graphic3d_Texture2D.hxx>
#include <fstream>
#include <iostream>
#include <iomanip>

static void capping_contract_color(const Quantity_ColorRGBA& color) {
  double r,g,b;color.GetRGB().Values(r,g,b,Quantity_TOC_RGB);
  std::cout<<' '<<r<<' '<<g<<' '<<b<<' '<<color.Alpha();
}
static occ::handle<Graphic3d_AspectFillArea3d> capping_contract_aspects(bool object) {
  const occ::handle<Graphic3d_AspectFillArea3d> aspect=new Graphic3d_AspectFillArea3d;
  aspect->SetInteriorColor(Quantity_Color(object?.6:.2,object?.7:.3,object?.8:.4,Quantity_TOC_RGB));
  aspect->SetInteriorColor(Quantity_ColorRGBA(aspect->InteriorColor(),object?.9f:.5f));
  auto front=aspect->FrontMaterial(),back=aspect->BackMaterial();
  front.SetTransparency(object?.4f:.1f);back.SetTransparency(object?.7f:.2f);
  aspect->SetFrontMaterial(front);aspect->SetBackMaterial(back);
  aspect->SetAlphaMode(Graphic3d_AlphaMode_MaskBlend,.25f);
  aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Phong);
  const occ::handle<Image_PixMap> image=new Image_PixMap;image->InitZero(Image_Format_RGBA,1,1);
  aspect->SetTextureMap(new Graphic3d_Texture2D(image));aspect->SetTextureMapOn();return aspect;
}
static void capping_contract_row(const occ::handle<OpenGl_Context>& context,int flags,const std::string& prefix) {
  const occ::handle<Graphic3d_ClipPlane> plane=new Graphic3d_ClipPlane(NCollection_Vec4<double>(.6,0,-.8,-2));
  plane->SetCappingAspect(capping_contract_aspects(false));
  plane->SetUseObjectMaterial((flags&1)!=0);plane->SetUseObjectTexture((flags&2)!=0);
  const auto object=capping_contract_aspects(true);
  const occ::handle<OpenGl_CappingPlaneResource> resource=new OpenGl_CappingPlaneResource(plane);
  resource->Update(context,object);
  if(flags==0) {std::ofstream orientation(prefix+".orientation",std::ios::binary);
    orientation.write(reinterpret_cast<const char*>(resource->Orientation().GetData()),64);}
  const auto aspect=resource->AspectFace()->Aspect();
  std::cout<<"ASPECT "<<flags<<' '<<aspect->FrontMaterial().Transparency()<<' '
    <<aspect->BackMaterial().Transparency()<<' '<<int(aspect->AlphaMode())<<' '
    <<aspect->AlphaCutoff()<<' '<<int(aspect->ShadingModel())<<' '
    <<(aspect->TextureMap()==object->TextureMap());
  capping_contract_color(aspect->InteriorColorRGBA());std::cout<<'\n';
}
static void capping_contract_vertices(const std::string& prefix) {
  const occ::handle<Graphic3d_ClipPlane> plane=new Graphic3d_ClipPlane;
  const occ::handle<OpenGl_CappingPlaneResource> resource=new OpenGl_CappingPlaneResource(plane);
  const auto attributes=resource->Primitives().Attributes();
  std::ofstream output(prefix+".vertices",std::ios::binary);
  output.write(reinterpret_cast<const char*>(attributes->Data()),attributes->Size());
  std::cout<<"VERTICES "<<attributes->NbElements<<' '<<attributes->Stride<<' '<<attributes->Size()<<'\n';
}
static void capping_contract_filter() {
  const occ::handle<OpenGl_Workspace> workspace=new OpenGl_Workspace(nullptr,occ::handle<OpenGl_Window>());
  workspace->SetRenderFilter(OpenGl_RenderFilter_FillModeOnly);
  const OpenGl_Aspects aspect(capping_contract_aspects(true));
  const occ::handle<Graphic3d_ClipPlane> plane=new Graphic3d_ClipPlane;
  const occ::handle<OpenGl_CappingPlaneResource> resource=new OpenGl_CappingPlaneResource(plane);
  std::cout<<"MASK_ASPECT_FILTER "<<workspace->ShouldRender(&aspect,nullptr)<<' '
    <<workspace->ShouldRender(&resource->Primitives(),nullptr)<<'\n';
}

int main(int argc,char** argv) {
  if(argc!=2)throw Standard_Failure("received cap CPUcontract arguments; expected output prefix");
  const occ::handle<OpenGl_Context> context=new OpenGl_Context;
  std::cout<<std::setprecision(17);capping_contract_vertices(argv[1]);capping_contract_filter();
  for(int flags=0;flags<4;++flags)capping_contract_row(context,flags,argv[1]);
}
