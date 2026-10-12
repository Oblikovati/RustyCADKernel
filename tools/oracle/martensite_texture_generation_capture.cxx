// Actual programmable/forced-FFP contracts for Martensite; OCCT 3d097a0.
#include "martensite_surface_texture_probe.hxx"
#include <Graphic3d_Texture2Dplane.hxx>
#include <OpenGl_Caps.hxx>

static occ::handle<Graphic3d_Texture2Dplane> generation_texture(const std::string& mode) {
  const auto image=texture_source("nearest")->GetImage(occ::handle<Image_SupportedFormats>());
  const occ::handle<Graphic3d_Texture2Dplane> texture=new Graphic3d_Texture2Dplane(image);
  const auto params=texture->GetParams();params->SetFilter(Graphic3d_TOTF_NEAREST);
  const auto generation=mode.rfind("object",0)==0?Graphic3d_TOTM_OBJECT:
    mode.rfind("eye",0)==0?Graphic3d_TOTM_EYE:Graphic3d_TOTM_MANUAL;
  params->SetGenMode(generation,NCollection_Vec4<float>(.0625f,0,.0125f,.5f),
    NCollection_Vec4<float>(0,.0625f,0,.5f));
  return texture;
}
static void generation_geometry(const occ::handle<Graphic3d_Group>& group,const std::string& mode) {
  if(mode.find("-q")!=std::string::npos) {projective_surface_geometry(group);return;}
  const bool uv=mode.find("no-uv")==std::string::npos;
  const occ::handle<Graphic3d_ArrayOfTriangles> array=new Graphic3d_ArrayOfTriangles(4,6,true,false,uv);
  const float positions[][3]={{-8,-8,0},{8,-8,0},{8,8,0},{-8,8,0}};
  for(int rank=0;rank<4;++rank) {
    const gp_Pnt point(positions[rank][0],positions[rank][1],positions[rank][2]);
    if(uv)array->AddVertex(point,gp_Dir(0,0,1),gp_Pnt2d(rank==1||rank==2?1:0,rank>=2?1:0));
    else array->AddVertex(point,gp_Dir(0,0,1));
  }
  for(int index:{1,2,3,1,3,4})array->AddEdge(index);
  group->AddPrimitiveArray(array);
}
static occ::handle<Graphic3d_Structure> generation_structure(
  const occ::handle<V3d_View>& view,const occ::handle<Graphic3d_Texture2Dplane>& texture,
  const std::string& mode) {
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(view->Viewer()->StructureManager());
  const auto group=structure->NewGroup();
  const occ::handle<Graphic3d_AspectFillArea3d> aspect=new Graphic3d_AspectFillArea3d;
  aspect->SetInteriorStyle(Aspect_IS_SOLID);aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  aspect->SetInteriorColor(texture_rgb(1,1,1));aspect->SetPolygonOffsets(Aspect_POM_Off,0,0);
  aspect->SetTextureMap(texture);aspect->SetTextureMapOn();group->SetGroupPrimitivesAspect(aspect);
  generation_geometry(group,mode);
  if(mode.find("moved")!=std::string::npos) {gp_Trsf transform;transform.SetTranslation(gp_Vec(2,1,1));group->SetTransformation(transform);}
  if(mode.find("camera")!=std::string::npos) {view->Camera()->SetEye(gp_Pnt(2,1,20));view->Camera()->SetCenter(gp_Pnt(2,1,0));}
  structure->Display();return structure;
}
static void generation_update(const occ::handle<Graphic3d_Texture2Dplane>& texture,const std::string& mode) {
  if(mode.find("updated")==std::string::npos)return;
  texture->SetPlaneS(.03125f,0,0,.25f);texture->SetPlaneT(0,.03125f,0,.25f);
}
static void generation_group_translation(const occ::handle<Graphic3d_Group>& group,double x) {
  gp_Trsf transform;transform.SetTranslation(gp_Vec(x,0,0));group->SetTransformation(transform);
}
static void generation_shared_groups(const occ::handle<Graphic3d_Structure>& structure,const std::string& mode) {
  if(mode.find("shared")==std::string::npos)return;
  const auto first=structure->Groups().First();generation_group_translation(first,-4);
  if(mode.find("reset")!=std::string::npos) {
    const auto middle=structure->NewGroup();
    const occ::handle<Graphic3d_AspectFillArea3d> plain=new Graphic3d_AspectFillArea3d;
    plain->SetInteriorStyle(Aspect_IS_SOLID);plain->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
    middle->SetGroupPrimitivesAspect(plain);generation_geometry(middle,"manual-uv");
    generation_group_translation(middle,100);
  }
  const auto second=structure->NewGroup();second->SetGroupPrimitivesAspect(first->Aspects());
  generation_geometry(second,mode);generation_group_translation(second,4);
}
int main(int argc,char** argv) {
  if(argc!=4)throw Standard_Failure("received texture generation arguments; expected programmable|fixed mode prefix");
  const std::string path=argv[1],mode=argv[2];
  if(path!="programmable"&&path!="fixed")throw Standard_Failure(("received path="+path+"; expected programmable|fixed").c_str());
  auto probe=create_policy_probe();texture_camera(probe.view);
  probe.workspace->GetGlContext()->caps->ffpEnable=path=="fixed";
  const auto texture=generation_texture(mode);const auto structure=generation_structure(probe.view,texture,mode);
  generation_shared_groups(structure,mode);
  probe.view->Redraw();generation_update(texture,mode);structure->Groups().First()->SynchronizeAspects();
  texture_capture(probe.view,argv[3]);probe.view->Remove();
}
