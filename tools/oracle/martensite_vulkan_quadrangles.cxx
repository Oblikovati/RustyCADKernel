// Real GL_QUADS/GL_QUAD_STRIP with colored vertices expose the driver's diagonal choice.
#include "martensite_vulkan_bound_array.hxx"
#include <array>
#include <vector>
static occ::handle<Graphic3d_AspectFillArea3d> quad_style(double r,double g,double b,float transparency,bool opaque) {
  auto style=unlit_aspects();style->SetInteriorColor(Quantity_Color(r,g,b,Quantity_TOC_RGB));style->SetInteriorColor(Quantity_ColorRGBA(style->InteriorColor(),1.f-transparency));auto material=style->FrontMaterial();material.SetTransparency(transparency);style->SetFrontMaterial(material);
  if(opaque)style->SetAlphaMode(Graphic3d_AlphaMode_Opaque);return style;
}
static std::vector<std::array<float,3>> quad_probe_points(const std::string& mode,float z) {
  if(mode=="quad-strips")return {{-6,-4,z},{-6,4,z},{0,-4,z},{0,4,z},{6,-4,z},{6,4,z}};
  if(mode=="quad-bounds")return {{-7,-4,z},{-7,4,z},{-1,-4,z},{-1,4,z},{1,-4,z},{1,4,z},{7,-4,z},{7,4,z}};
  return {{-6,-4,z},{6,-4,z},{6,4,z},{-6,4,z}};
}
static occ::handle<Graphic3d_ArrayOfPrimitives> quad_probe_array(const std::string& mode,double offset,float z) {
  const auto points=quad_probe_points(mode,z);const bool strip=mode=="quad-strips"||mode=="quad-bounds";
  const bool colored=mode=="quad-gradient"||mode=="quad-strips";
  auto flags=colored?Graphic3d_ArrayFlags_VertexColor:mode=="quad-bounds"?Graphic3d_ArrayFlags_BoundColor:Graphic3d_ArrayFlags_None;
  occ::handle<Graphic3d_ArrayOfPrimitives> array=new MartensiteOracleBoundArray(strip?Graphic3d_TOPA_QUADRANGLESTRIPS:Graphic3d_TOPA_QUADRANGLES,points.size(),mode=="quad-bounds"?2:0,mode=="quad-indexed"?4:0,flags);
  const double colors[4][3]={{1,0,0},{0,1,0},{1,1,1},{0,0,1}};
  for(size_t i=0;i<points.size();++i){const auto p=points[i];int rank=array->AddVertex(float(p[0]+offset),p[1],p[2]);if(colored)array->SetVertexColor(rank,colors[i%4][0],colors[i%4][1],colors[i%4][2]);}
  if(mode=="quad-indexed")for(int index:{4,3,2,1})array->AddEdge(index);
  if(mode=="quad-bounds"){array->AddBound(4,1,0,0);array->AddBound(4,0,1,0);}return array;
}
struct MartensiteOracleQuadScene { occ::handle<Graphic3d_Structure> primary,occluder; };
static MartensiteOracleQuadScene quadrangle_probe_scene(const occ::handle<V3d_Viewer>& viewer,const std::string& mode,double offset) {
  occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(viewer->StructureManager());auto group=structure->NewGroup();
  const bool transparent=mode=="quad-layers"||mode=="quad-clear-depth"||mode=="quad-material-only"||mode=="quad-transparent"||mode=="quad-occlusion"||mode=="quad-overlap"||mode=="quad-alpha-opaque";
  group->SetPrimitivesAspect(quad_style(0,1,0,transparent?.5f:0.f,mode=="quad-alpha-opaque"));if(mode=="quad-material-only") {auto materialOnly=quad_style(0,1,0,.5f,false);materialOnly->SetInteriorColor(Quantity_ColorRGBA(materialOnly->InteriorColor(),1.f));group->SetPrimitivesAspect(materialOnly);}group->AddPrimitiveArray(quad_probe_array(mode,offset,0));
  if(mode=="quad-occlusion"||mode=="quad-overlap") {group->SetPrimitivesAspect(quad_style(1,0,0,mode=="quad-overlap"?.5f:0.f,false));group->AddPrimitiveArray(quad_probe_array("quad-transparent",offset,1));}
  occ::handle<Graphic3d_Structure> occluder;
  if(mode=="quad-layers"||mode=="quad-clear-depth") {
    Graphic3d_ZLayerSettings settings;settings.SetClearDepth(mode=="quad-clear-depth");Graphic3d_ZLayerId layer;
    viewer->AddZLayer(layer,settings);occluder=new Graphic3d_Structure(viewer->StructureManager());occluder->SetZLayer(layer);
    auto back=occluder->NewGroup();back->SetPrimitivesAspect(quad_style(1,0,0,0.f,false));back->AddPrimitiveArray(quad_probe_array("quad-transparent",offset,-1));occluder->Display();
  }
  structure->Display();return {structure,occluder};
}
