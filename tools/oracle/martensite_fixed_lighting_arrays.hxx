#pragma once
#include <Graphic3d_ArrayOfQuadrangles.hxx>
#include <Graphic3d_ArrayOfQuadrangleStrips.hxx>
#include <Graphic3d_ArrayOfTriangleStrips.hxx>
#include <Graphic3d_ArrayOfTriangleFans.hxx>
#include <Graphic3d_ArrayOfPolygons.hxx>
#include <Graphic3d_ArrayOfPolylines.hxx>
static occ::handle<Graphic3d_ArrayOfPrimitives> fixed_array_constructor(const std::string& mode) {
  const auto flags=Graphic3d_ArrayFlags(Graphic3d_ArrayFlags_VertexNormal|Graphic3d_ArrayFlags_VertexColor);
  if(mode=="lit-facet-quad")return new Graphic3d_ArrayOfQuadrangles(4,4,flags);
  if(mode=="lit-facet-quad-strip")return new Graphic3d_ArrayOfQuadrangleStrips(4,0,flags);
  if(mode=="lit-facet-triangle-strip")return new Graphic3d_ArrayOfTriangleStrips(4,0,flags);
  if(mode=="lit-facet-fan")return new Graphic3d_ArrayOfTriangleFans(4,0,flags);
  if(mode=="lit-facet-polygon")return new Graphic3d_ArrayOfPolygons(4,0,4,flags);
  if(mode.rfind("lit-strip-",0)==0)return new Graphic3d_ArrayOfPolylines(4,0,4,flags);
  return {};
}
static occ::handle<Graphic3d_ArrayOfPrimitives> fixed_array_override(const std::string& mode) {
  const auto vertices=fixed_array_constructor(mode);if(vertices.IsNull())return {};
  const float points[4][3]={{-8,-8,0},{8,-8,0},{8,8,0},{-8,8,0}};
  const float normals[4][3]={{0,0,1},{.6f,0,.8f},{0,.6f,.8f},{.6f,.6f,.529150262f}};
  const float colors[4][3]={{.25f,.5f,.75f},{.75f,.25f,.5f},{.5f,.75f,.25f},{.125f,.375f,.625f}};
  const bool strip=mode.find("quad-strip")!=std::string::npos||mode.find("triangle-strip")!=std::string::npos;
  for(int rank=0;rank<4;++rank) {
    const int index=strip&&rank>=2?5-rank:rank;
    vertices->SetVertice(rank+1,points[index][0],points[index][1],points[index][2]);
    vertices->SetVertexNormal(rank+1,normals[index][0],normals[index][1],normals[index][2]);
    vertices->SetVertexColor(rank+1,lighting_rgb(colors[index][0],colors[index][1],colors[index][2]));
  }
  if(!strip&&mode!="lit-facet-fan")for(int index:{1,2,3,4})vertices->AddEdge(index);
  return vertices;
}
