#include <BRepMesh_EdgeTessellationExtractor.hxx>
void mesh_tessellation_extractor(std::istream& input) {
  std::array<int,4> v;for(auto& x:v)input>>x;auto model=post_model({v[0],0,0,3,1,1,0});BRepMesh_ModelPostProcessor post;post.Perform(model,IMeshTools_Parameters(),Message_ProgressRange());auto edge=model->GetEdge(0);auto face=model->GetFace(0);edge->SetSameParam(v[1]);gp_Trsf transform;transform.SetTranslation(gp_Vec(v[2]*7.,0,0));TopLoc_Location location(transform);edge->SetShape(edge->GetShape().Located(location));face->SetShape(face->GetShape().Located(location));
  BRepMesh_EdgeTessellationExtractor extractor(edge,face);ga_int(extractor.PointsNb());ga_flag(extractor.IsKind(STANDARD_TYPE(IMeshTools_CurveTessellator)));gp_Pnt point;double parameter=0;
  std::array<int,4> order=v[3]==0?std::array<int,4>{1,2,1,2}:std::array<int,4>{2,1,2,1};for(int index:order){ga_flag(extractor.Value(index,point,parameter));ga_pnt(point);ga_real(parameter);}ga_end();
}
