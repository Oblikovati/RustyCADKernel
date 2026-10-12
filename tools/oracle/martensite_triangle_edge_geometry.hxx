#pragma once
// Real native array topology, independent of the shader policy under test.
#include <array>
// The concrete triangle factory ignores bounds; the real protected generic constructor retains them.
class MartensiteBoundedTriangleArray final : public Graphic3d_ArrayOfPrimitives {
public:
  MartensiteBoundedTriangleArray(int vertices,int indices,Graphic3d_ArrayFlags flags)
    : Graphic3d_ArrayOfPrimitives(Graphic3d_TOPA_TRIANGLES,vertices,2,indices,flags) {}
};
static std::array<gp_Pnt,4> edge_fixture_positions() {
  return {gp_Pnt(-8,-8,0),gp_Pnt(8,-8,0),gp_Pnt(8,8,0),gp_Pnt(-8,8,0)};
}
static std::array<int,6> edge_fixture_ranks(const std::string& mode) {
  if(edge_case_has(mode,"back"))return {0,2,1,0,3,2};
  return {0,1,2,0,2,3};
}
static occ::handle<Graphic3d_ArrayOfPrimitives> edge_triangle_array(const std::string& mode) {
  const bool direct=edge_case_has(mode,"unindexed"),bounds=edge_case_has(mode,"bounds");
  auto flags=Graphic3d_ArrayFlags(Graphic3d_ArrayFlags_VertexNormal|Graphic3d_ArrayFlags_VertexTexel);
  if(edge_case_has(mode,"vertex"))flags=Graphic3d_ArrayFlags(flags|Graphic3d_ArrayFlags_VertexColor);
  const occ::handle<Graphic3d_ArrayOfPrimitives> array=bounds
    ?occ::handle<Graphic3d_ArrayOfPrimitives>(new MartensiteBoundedTriangleArray(direct?6:4,direct?0:6,flags))
    :Graphic3d_ArrayOfPrimitives::CreateArray(Graphic3d_TOPA_TRIANGLES,direct?6:4,0,direct?0:6,flags);
  if(bounds) {array->AddBound(3);array->AddBound(3);}
  const auto points=edge_fixture_positions();const auto ranks=edge_fixture_ranks(mode);
  for(int index=0;index<(direct?6:4);++index) {
    const int rank=direct?ranks[index]:index;
    gp_Pnt point=points[rank];if(edge_case_has(mode,"perspective"))point.SetZ(rank<2?-4:4);
    const int vertex=array->AddVertex(point,gp_Dir(0,0,1),gp_Pnt2d(rank==1||rank==2?1:0,rank>=2?1:0));
    if(edge_case_has(mode,"vertex"))array->SetVertexColor(vertex,texture_rgb(.25f,.5f,.75f));
  }
  if(!direct)for(int rank:ranks)array->AddEdge(rank+1);
  return array;
}
static void edge_case_triangles(const occ::handle<Graphic3d_Group>& group,const std::string& mode) {
  if(edge_case_has(mode,"projective")) {projective_surface_geometry(group);return;}
  const auto array=edge_triangle_array(mode);
  group->AddPrimitiveArray(array);
}
static void edge_case_view(const occ::handle<V3d_View>& view,const std::string& mode) {
  texture_camera(view);texture_lights(view);
  if(edge_case_has(mode,"resolution-144"))view->ChangeRenderingParams().Resolution=144;
  if(edge_case_has(mode,"feather-two"))view->ChangeRenderingParams().LineFeather=2.f;
  if(edge_case_has(mode,"feather-zero"))view->ChangeRenderingParams().LineFeather=0.f;
  if(edge_case_has(mode,"perspective"))view->Camera()->SetProjectionType(Graphic3d_Camera::Projection_Perspective);
  if(edge_case_has(mode,"clipped")) {
    const occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane;
    planes->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(1,0,0,0)));view->SetClipPlanes(planes);
  }
}
