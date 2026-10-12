// Native same-pass mesh edges and mapped-aspect lines; OCCT 3d097a0.
#include "martensite_surface_texture_probe.hxx"
#include <Graphic3d_ArrayOfSegments.hxx>
#include <Graphic3d_ArrayOfPolylines.hxx>
#include <OpenGl_Caps.hxx>

static bool edge_case_has(const std::string& mode,const std::string& token) {
  return mode.find(token)!=std::string::npos;
}
#include "martensite_triangle_edge_geometry.hxx"
static occ::handle<Graphic3d_AspectFillArea3d> edge_case_aspect(
  const occ::handle<Graphic3d_Texture2D>& texture,const std::string& mode) {
  const occ::handle<Graphic3d_AspectFillArea3d> aspect=new Graphic3d_AspectFillArea3d;
  aspect->SetInteriorStyle(Aspect_IS_SOLID);aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  aspect->SetPolygonOffsets(Aspect_POM_Off,0,0);texture_material(aspect);
  aspect->SetTextureMap(texture);aspect->SetTextureMapOn();
  if(edge_case_has(mode,"unmapped"))aspect->SetTextureMapOff();
  if(edge_case_has(mode,"phong"))aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Phong);
  if(edge_case_has(mode,"gouraud"))aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Gouraud);
  if(edge_case_has(mode,"facet"))aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_PhongFacet);
  if(edge_case_has(mode,"mask-blend"))aspect->SetAlphaMode(Graphic3d_AlphaMode_MaskBlend,.5f);
  else if(edge_case_has(mode,"mask"))aspect->SetAlphaMode(Graphic3d_AlphaMode_Mask,.5f);
  else if(edge_case_has(mode,"blend"))aspect->SetAlphaMode(Graphic3d_AlphaMode_Blend,.5f);
  if(edge_case_has(mode,"node-alpha"))aspect->SetInteriorColor(Quantity_ColorRGBA(.75f,.5f,.25f,.25f));
  if(edge_case_has(mode,"mat-alpha")) {
    auto material=aspect->FrontMaterial();material.SetTransparency(.5f);aspect->SetFrontMaterial(material);
  }
  return aspect;
}
static void edge_case_parameters(const occ::handle<Graphic3d_AspectFillArea3d>& aspect,
  const std::string& mode) {
  aspect->SetEdgeOn();aspect->SetEdgeColor(Quantity_Color(0,1,1,Quantity_TOC_RGB));
  aspect->SetEdgeWidth(edge_case_has(mode,"wide")?3.f:1.f);
  if(edge_case_has(mode,"dashed"))aspect->SetEdgeLineType(Aspect_TOL_DASH);
  if(edge_case_has(mode,"alpha"))aspect->SetEdgeColor(Quantity_ColorRGBA(0,1,1,edge_case_has(mode,"negative")?-.25f:.25f));
  if(edge_case_has(mode,"background"))aspect->SetEdgeColor(Quantity_ColorRGBA(0,1,1,.05f));
  if(edge_case_has(mode,"skip-first"))aspect->SetSkipFirstEdge(true);
  if(edge_case_has(mode,"front-cull"))aspect->SetFaceCulling(Graphic3d_TypeOfBackfacingModel_FrontCulled);
  if(edge_case_has(mode,"back-cull"))aspect->SetFaceCulling(Graphic3d_TypeOfBackfacingModel_BackCulled);
}
static void edge_case_lines(const occ::handle<Graphic3d_Group>& group,const std::string& mode) {
  const bool polyline=edge_case_has(mode,"polyline");
  const bool normals=edge_case_has(mode,"phong")||edge_case_has(mode,"gouraud")||edge_case_has(mode,"facet");
  const auto kind=polyline?Graphic3d_TOPA_POLYLINES:Graphic3d_TOPA_SEGMENTS;
  const auto flags=Graphic3d_ArrayFlags((normals?Graphic3d_ArrayFlags_VertexNormal:0)
    |(edge_case_has(mode,"vertex")?Graphic3d_ArrayFlags_VertexColor:0));
  const auto array=Graphic3d_ArrayOfPrimitives::CreateArray(kind,4,polyline?1:0,4,flags);
  if(polyline)array->AddBound(4);
  for(const gp_Pnt& point:{gp_Pnt(-8,-8,0),gp_Pnt(8,-8,0),gp_Pnt(8,8,0),gp_Pnt(-8,8,0)}) {
    const int rank=array->AddVertex(point);if(normals)array->SetVertexNormal(rank,gp_Dir(0,0,1));
    if(edge_case_has(mode,"vertex"))array->SetVertexColor(rank,texture_rgb(.25f,.5f,.75f));
    array->AddEdge(rank);
  }
  group->AddPrimitiveArray(array);
}
static occ::handle<Graphic3d_Structure> edge_case_structure(
  const occ::handle<V3d_View>& view,const occ::handle<Graphic3d_AspectFillArea3d>& aspect,
  const std::string& mode) {
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(view->Viewer()->StructureManager());
  const auto group=structure->NewGroup();group->SetGroupPrimitivesAspect(aspect);
  if(edge_case_has(mode,"line"))edge_case_lines(group,mode);
  else edge_case_triangles(group,mode);
  structure->Display();return structure;
}
int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected mode prefix").c_str());
  const std::string mode=argv[1];auto probe=create_policy_probe();edge_case_view(probe.view,mode);
  const auto texture=texture_source(edge_case_has(mode,"mask")||edge_case_has(mode,"blend")?"mask":"nearest");
  texture->GetParams()->SetFilter(Graphic3d_TOTF_NEAREST);
  if(edge_case_has(mode,"no-modulate"))texture->GetParams()->SetModulate(false);
  const auto aspect=edge_case_aspect(texture,mode);edge_case_parameters(aspect,mode);
  probe.workspace->GetGlContext()->caps->usePolygonMode=edge_case_has(mode,"polygon");
  const auto structure=edge_case_structure(probe.view,aspect,mode);
  texture_capture(probe.view,argv[2]);probe.view->Remove();
}
