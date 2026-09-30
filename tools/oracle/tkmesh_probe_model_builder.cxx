// Actual production topology, geometric bounds and native BRepMesh model construction.
#include <BRepMesh_ModelBuilder.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
TopoDS_Shape model_shape(int kind) {
  if(kind==0)return TopoDS_Shape();if(kind==5)return BRepPrimAPI_MakeBox(1,2,3).Shape();if(kind==6)return BRepPrimAPI_MakeSphere(1.5).Shape();
  BRep_Builder builder;if(kind==1){TopoDS_Vertex vertex;builder.MakeVertex(vertex,gp_Pnt(1,2,3),1.e-7);return vertex;}
  auto [wire,face,ordered]=order_fixture(0,0,kind==8?1:0);if(kind==2)return ordered->Edge(1);if(kind==3)return wire;
  if(kind==7){TopoDS_Compound compound;builder.MakeCompound(compound);builder.Add(compound,face);gp_Trsf transform;transform.SetTranslation(gp_Vec(7,11,13));builder.Add(compound,face.Located(TopLoc_Location(transform)));return compound;}
  if(kind==9){auto [inner,unused,unused2]=order_fixture(0,0,1);builder.Add(face,inner);}return face;
}
void report_model_face(const occ::handle<IMeshData_Model>& model,const IMeshData::IFaceHandle& face) {
  ga_int(face->GetStatusMask());ga_int(face->WiresNb());
  for(int index=0;index<face->WiresNb();++index){auto wire=face->GetWire(index);ga_int(wire->GetStatusMask());ga_int(wire->EdgesNb());
    for(int entry=0;entry<wire->EdgesNb();++entry){auto edge=wire->GetEdge(entry);for(int i=0;i<model->EdgesNb();++i)if(model->GetEdge(i).get()==edge){ga_int(i);break;}ga_int(wire->GetEdgeOrientation(entry));}}
}
void report_model_edges(const occ::handle<IMeshData_Model>& model) {
  for(int index=0;index<model->EdgesNb();++index){auto edge=model->GetEdge(index);ga_flag(edge->GetSameParam());ga_flag(edge->GetSameRange());ga_flag(edge->GetDegenerated());ga_int(edge->PCurvesNb());
    for(int entry=0;entry<edge->PCurvesNb();++entry){auto pcurve=edge->GetPCurve(entry);for(int i=0;i<model->FacesNb();++i)if(model->GetFace(i).get()==pcurve->GetFace()){ga_int(i);break;}ga_int(pcurve->GetOrientation());}}
}
void mesh_model_builder(std::istream& input) {
  std::array<int,5> v;for(auto& value:v)input>>value;auto shape=model_shape(v[0]);shape.Orientation(TopAbs_Orientation(v[4]));
  IMeshTools_Parameters p;p.Relative=v[1];p.Deflection=std::array<double,3>{0.001,-1,1}[v[2]];p.DeflectionInterior=std::array<double,3>{-1,0.2,4}[v[3]];
  BRepMesh_ModelBuilder builder;auto model=builder.Perform(shape,p);for(auto status:{Message_Done1,Message_Fail1,Message_Fail2})ga_flag(builder.GetStatus().IsSet(status));ga_flag(model.IsNull());
  if(model.IsNull()){ga_end();return;}ga_real(model->GetMaxSize());ga_int(model->FacesNb());ga_int(model->EdgesNb());
  for(int i=0;i<model->FacesNb();++i)report_model_face(model,model->GetFace(i));report_model_edges(model);ga_end();
}
