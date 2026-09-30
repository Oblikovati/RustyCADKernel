#include <BRepMesh_ModelHealer.hxx>
static void healer_alter_curve(const IMeshData::IPCurveHandle& curve,int index,int mode){
 int count=curve->ParametersNb();if(!count)return;
 if(mode==1){auto point=curve->GetPoint(0);curve->GetPoint(0)=gp_Pnt2d(point.X()+(index+1)*.01,point.Y()-.02);}
 if(mode==2){std::vector<gp_Pnt2d> points;for(int i=0;i<count;++i)points.push_back(curve->GetPoint(i));for(int i=0;i<count;++i)curve->GetPoint(i)=points[count-i-1];}
}
static void healer_replace_vertex(TopoDS_Edge& edge,const TopoDS_Vertex& vertex,int index,int mode,bool first){
 BRep_Builder builder;builder.Remove(edge,vertex);if(mode==11||(mode==10&&first))return;auto point=BRep_Tool::Pnt(vertex);double offsets[]={0.,1.e-7,2.e-7,4.e-7};double offset=index%2==0||mode>=10?0.:offsets[mode-6];TopoDS_Vertex replacement;builder.MakeVertex(replacement,gp_Pnt(point.X()+offset,point.Y(),point.Z()),BRep_Tool::Tolerance(vertex));replacement.Orientation(vertex.Orientation());builder.Add(edge,replacement);
}
static void healer_separate_vertices(TopoDS_Edge edge,int index,int mode){
 TopoDS_Vertex first,last;TopExp::Vertices(edge,first,last);edge.Free(true);healer_replace_vertex(edge,first,index,mode,true);healer_replace_vertex(edge,last,index,mode,false);
}
static void healer_alter_model(const occ::handle<IMeshData_Model>& model,int mode){
 for(int i=0;i<model->EdgesNb();++i){auto edge=model->GetEdge(i);if(mode>=6)healer_separate_vertices(edge->GetEdge(),i,mode);if(mode==4)edge->SetStatus(IMeshData_Outdated);for(int j=0;j<edge->PCurvesNb();++j){auto curve=edge->GetPCurve(j);if(mode==3){curve->Clear(false);continue;}healer_alter_curve(curve,i,mode);}}
 if(mode==5&&model->FacesNb())model->GetFace(0)->SetStatus(IMeshData_Failure);
}
static void healer_report(const occ::handle<IMeshData_Model>& model){
 report_edge_discret(model);ga_int(model->FacesNb());for(int i=0;i<model->FacesNb();++i){auto face=model->GetFace(i);ga_int(face->GetStatusMask());ga_real(face->GetDeflection());ga_int(face->WiresNb());for(int j=0;j<face->WiresNb();++j){auto wire=face->GetWire(j);ga_int(wire->GetStatusMask());ga_real(wire->GetDeflection());}}
}
static occ::handle<IMeshData_Model> healer_model(int kind,int mode,const IMeshTools_Parameters& p){
 if(kind<10){BRepMesh_ModelBuilder builder;auto model=builder.Perform(model_shape(kind),p);BRepMesh_EdgeDiscret discret;discret.Perform(model,p,Message_ProgressRange());if(!model.IsNull())healer_alter_model(model,mode);return model;}
 auto[model,face]=checker_model({kind-10,mode%2,0,0,0,0,0});
 for(int i=0;i<model->EdgesNb();++i){auto edge=model->GetEdge(i);edge->SetDeflection(.01);edge->SetAngularDeflection(p.Angle);edge->SetSameParam(true);edge->SetSameRange(true);TopoDS_Vertex first,last;TopExp::Vertices(edge->GetEdge(),first,last);edge->GetCurve()->AddPoint(BRep_Tool::Pnt(first),0.);edge->GetCurve()->AddPoint(BRep_Tool::Pnt(last),2.);}
 return model;
}
void mesh_healer(std::istream& input){
 std::array<int,5>v;for(auto& x:v)input>>x;IMeshTools_Parameters p;p.InParallel=v[2];p.Relative=v[3];p.MinSize=v[4]?10.:.001;p.Deflection=.1;
 auto model=healer_model(v[0],v[1],p);
 occ::handle<BRepMesh_ModelHealer> healer=new BRepMesh_ModelHealer;ga_flag(healer->IsInstance(STANDARD_TYPE(BRepMesh_ModelHealer)));ga_flag(healer->IsKind(STANDARD_TYPE(IMeshTools_ModelAlgo)));
 for(int i=0;i<2;++i){ga_flag(healer->Perform(model,p,Message_ProgressRange()));if(!model.IsNull())healer_report(model);}ga_end();
}
