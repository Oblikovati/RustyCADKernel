#include <BRepMesh_ModelPreProcessor.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
static occ::handle<IMeshData_Model> pre_model(const std::array<int,8>& v) {
  auto model=post_model({v[0],0,0,3,2,1,0});if(model.IsNull())return model;
  IMeshTools_Parameters p;p.MinSize=.001;p.Deflection=.1;
  BRepMesh_ModelPostProcessor().Perform(model,p,Message_ProgressRange());model->GetEdge(0)->SetStatus(IMeshData_Status(v[1]));
  for(int i=0;i<model->FacesNb();++i){auto face=model->GetFace(i);auto [wire,unused,unused2]=order_fixture(0,0,0);face->AddWire(wire,1)->AddEdge(model->GetEdge(0).get(),TopAbs_FORWARD);
    face->SetStatus(IMeshData_Status(v[2]));const double d[]={.01,.1,10.};face->SetDeflection(d[v[3]]);
    TopLoc_Location loc;auto mesh=BRep_Tool::Triangulation(face->GetFace(),loc);mesh->Deflection(.1);occ::handle<Poly_TriangulationParameters> source;
    if(v[4])source=new Poly_TriangulationParameters(v[4]==1?.01:-1.,.5,.001);mesh->Parameters(source);
    mesh->SetTriangle(1,Poly_Triangle(v[5]==0?1:v[5]==1?0:4,2,3));}
  return model;
}
static void pre_report_faces(const occ::handle<IMeshData_Model>& model) {
  ga_int(model->FacesNb());for(int i=0;i<model->FacesNb();++i){auto f=model->GetFace(i);ga_int(f->GetStatusMask());ga_real(f->GetDeflection());}
}
void mesh_pre(std::istream& input) {
  std::array<int,8>v;for(auto& x:v)input>>x;auto model=pre_model(v);IMeshTools_Parameters p;p.InParallel=v[7];p.AllowQualityDecrease=v[6];
  ga_flag(BRepMesh_ModelPreProcessor().Perform(model,p,Message_ProgressRange()));if(!model.IsNull()){pre_report_faces(model);report_post_model(model);}ga_end();
}
template<class Curve>static void reverse_pre_curve(const Curve& curve){
  for(int i=0;i<curve->ParametersNb()/2;++i){int j=curve->ParametersNb()-1-i;auto point=curve->GetPoint(i);double parameter=curve->GetParameter(i);curve->GetPoint(i)=curve->GetPoint(j);curve->GetParameter(i)=curve->GetParameter(j);curve->GetPoint(j)=point;curve->GetParameter(j)=parameter;}
}
static void alter_pre_cone(const occ::handle<IMeshData_Model>& model,int mode){
  for(int f=0;f<model->FacesNb();++f){auto face=model->GetFace(f);if(face->GetSurface()->GetType()!=GeomAbs_Cone)continue;if(mode==5)face->SetStatus(IMeshData_Failure);auto wire=face->GetWire(0);
    for(int i=0;i<wire->EdgesNb();++i){auto edge=wire->GetEdge(i);if(edge->GetPCurve(face.get(),TopAbs_FORWARD)==edge->GetPCurve(face.get(),TopAbs_REVERSED))continue;
      if(mode==1){reverse_pre_curve(edge->GetCurve());for(int p=0;p<edge->PCurvesNb();++p)reverse_pre_curve(edge->GetPCurve(p));}
      if(mode==2)BRep_Builder().UpdateEdge(edge->GetEdge(),occ::handle<Geom2d_Curve>(),face->GetFace(),1e-7);
      if(mode==3){auto p=edge->GetPCurve(0);p->GetPoint(p->ParametersNb()-1)=p->GetPoint(0);}
      if(mode==4){auto c=edge->GetCurve();c->InsertPoint(1,c->GetPoint(0),c->GetParameter(0));}return;
    }
  }
}
void mesh_pre_cone(std::istream& input) {
  int kind,def,angle,parallel,mode;input>>kind>>def>>angle>>parallel>>mode;
  IMeshTools_Parameters p;const double ds[]={.01,.1,1.};p.Deflection=ds[def];p.Angle=angle?.7:.3;p.MinSize=.001;p.InParallel=parallel;
  BRepPrimAPI_MakeCone builder(5.,kind==0?0.:2.,10.);auto model=BRepMesh_ModelBuilder().Perform(builder.Shape(),p);
  BRepMesh_EdgeDiscret().Perform(model,p,Message_ProgressRange());alter_pre_cone(model,mode);
  ga_flag(BRepMesh_ModelPreProcessor().Perform(model,p,Message_ProgressRange()));pre_report_faces(model);report_edge_discret(model);ga_end();
}
