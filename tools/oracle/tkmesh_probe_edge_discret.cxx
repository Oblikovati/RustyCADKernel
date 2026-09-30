#include <TopExp.hxx>
#include <BRepMesh_EdgeDiscret.hxx>
void seed_free_polygons(const occ::handle<IMeshData_Model>& model,int kind) {
  if(model.IsNull()||kind==0)return;for(int i=0;i<model->EdgesNb();++i){auto edge=model->GetEdge(i);if(!edge->IsFree())continue;NCollection_Array1<gp_Pnt> nodes(1,3);NCollection_Array1<double> parameters(1,3);for(int j=1;j<=3;++j){nodes(j)=gp_Pnt(j,0,0);parameters(j)=j-1.;}occ::handle<Poly_Polygon3D> polygon=kind==2?new Poly_Polygon3D(nodes):new Poly_Polygon3D(nodes,parameters);polygon->Deflection(kind==3?100.:0.01);BRepMesh_ShapeTool::UpdateEdge(edge->GetEdge(),polygon);}
}
occ::handle<IMeshData_Model> edge_discret_model(const std::array<int,6>& v,const IMeshTools_Parameters& parameters) {
  if(v[0]<10){BRepMesh_ModelBuilder builder;return builder.Perform(model_shape(v[0]),parameters);}auto model=post_model({v[0]-9,0,0,3,1,1,0});BRepMesh_ModelPostProcessor post;post.Perform(model,parameters,Message_ProgressRange());for(int i=0;i<model->EdgesNb();++i)model->GetEdge(i)->Clear(false);return model;
}
void report_edge_discret(const occ::handle<IMeshData_Model>& model) {
  ga_int(model->EdgesNb());for(int i=0;i<model->EdgesNb();++i){auto edge=model->GetEdge(i);ga_int(edge->GetStatusMask());ga_real(edge->GetDeflection());ga_real(edge->GetAngularDeflection());ga_flag(edge->GetSameParam());ga_flag(edge->GetSameRange());ga_flag(edge->GetDegenerated());auto curve=edge->GetCurve();ga_int(curve->ParametersNb());for(int j=0;j<curve->ParametersNb();++j){ga_pnt(curve->GetPoint(j));ga_real(curve->GetParameter(j));}ga_int(edge->PCurvesNb());for(int j=0;j<edge->PCurvesNb();++j){auto pcurve=edge->GetPCurve(j);ga_int(pcurve->ParametersNb());for(int k=0;k<pcurve->ParametersNb();++k){ga_pnt2d(pcurve->GetPoint(k));ga_real(pcurve->GetParameter(k));ga_int(pcurve->GetIndex(k));}}}
}
void mesh_edge_discret(std::istream& input) {
  std::array<int,6> v;for(auto& x:v)input>>x;IMeshTools_Parameters p;p.MinSize=v[1]?0.001:-1.;p.Relative=v[2];p.InParallel=v[3];p.AllowQualityDecrease=v[4];p.Deflection=0.1;auto model=edge_discret_model(v,p);seed_free_polygons(model,v[5]);BRepMesh_EdgeDiscret algorithm;ga_flag(algorithm.Perform(model,p,Message_ProgressRange()));if(!model.IsNull())report_edge_discret(model);ga_end();
}
void mesh_edge_tessellate(std::istream& input) {
  std::array<int,6> v;for(auto& x:v)input>>x;IMeshTools_Parameters p;p.MinSize=0.001;BRepMesh_ModelBuilder builder;auto model=builder.Perform(model_shape(v[0]),p);auto edge=model->GetEdge(v[1]%model->EdgesNb());edge->SetDeflection(0.1);edge->SetAngularDeflection(0.5);edge->SetDegenerated(v[3]);edge->SetSameParam(v[4]);auto pcurve=edge->GetPCurve(0);IMeshData::IFaceHandle face=pcurve->GetFace();auto tessellator=BRepMesh_EdgeDiscret::CreateEdgeTessellator(edge,pcurve->GetOrientation(),face,p,v[5]?5:2);
  if(v[2]==0){gp_Pnt point;double parameter;for(int index:{1,tessellator->PointsNb()}){tessellator->Value(index,point,parameter);edge->GetCurve()->AddPoint(point,parameter);}BRepMesh_EdgeDiscret::Tessellate2d(edge,true);}
  BRepMesh_EdgeDiscret::Tessellate3d(edge,tessellator,v[2]);BRepMesh_EdgeDiscret::Tessellate2d(edge,v[2]);report_edge_discret(model);ga_end();
}
class MutatingEndpoints : public IMeshTools_CurveTessellator {
public:TopoDS_Vertex first,last;int PointsNb() const override{return 2;}
  bool Value(int index,gp_Pnt&,double& parameter) const override {parameter=index;BRep_Builder builder;builder.UpdateVertex(index==1?first:last,gp_Pnt(index*7.,0,0),1.e-7);return false;}
};
void mesh_edge_callback(std::istream&) {
  BRepMesh_ModelBuilder builder;auto model=builder.Perform(model_shape(4),IMeshTools_Parameters());auto edge=model->GetEdge(0);occ::handle<MutatingEndpoints> callback=new MutatingEndpoints;TopExp::Vertices(edge->GetEdge(),callback->first,callback->last);BRepMesh_EdgeDiscret::Tessellate3d(edge,callback,true);report_edge_discret(model);ga_end();
}
