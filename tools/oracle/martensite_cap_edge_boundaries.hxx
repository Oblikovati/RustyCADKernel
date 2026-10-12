#pragma once
// OCCT3d097a0: unsampled mapped line binding must refresh a shared cap resource.
#include <Graphic3d_ArrayOfSegments.hxx>
static void cap_edge_boundary(const occ::handle<Graphic3d_Group>& group,
  const occ::handle<Graphic3d_Texture2D>& texture,const std::string& mode) {
  if(mode.find("source-edges")!=std::string::npos) {
    group->Aspects()->SetDrawEdges(true);
    if(mode.find("edges-disabled")!=std::string::npos)group->Aspects()->SetDrawEdges(false);
    group->Aspects()->SetEdgeColor(Quantity_ColorRGBA(0,1,1,1));
    if(mode.find("line-offset")!=std::string::npos)group->Aspects()->SetPolygonOffsets(Aspect_POM_Line,2,3);
    if(mode.find("fill-offset")!=std::string::npos)group->Aspects()->SetPolygonOffsets(Aspect_POM_Fill,2,3);
    if(mode.find("both-offset")!=std::string::npos)group->Aspects()->SetPolygonOffsets(Aspect_POM_Fill|Aspect_POM_Line,2,3);
    if(mode.find("dashed")!=std::string::npos)group->Aspects()->SetEdgeLineType(Aspect_TOL_DASH);
  }
  if(mode.find("mapped-line")==std::string::npos)return;
  const occ::handle<Graphic3d_AspectFillArea3d> aspect=new Graphic3d_AspectFillArea3d;
  aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  aspect->SetTextureMap(texture);aspect->SetTextureMapOn();
  aspect->SetInteriorColor(Quantity_ColorRGBA(0,1,1,1));
  group->SetPrimitivesAspect(aspect);
  const occ::handle<Graphic3d_ArrayOfSegments> lines=new Graphic3d_ArrayOfSegments(2);
  lines->AddVertex(-8.f,8.f,-1.f);lines->AddVertex(8.f,8.f,-1.f);group->AddPrimitiveArray(lines);
}
