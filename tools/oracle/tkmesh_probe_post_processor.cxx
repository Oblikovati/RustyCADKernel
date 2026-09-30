#include <BRepMesh_ModelPostProcessor.hxx>
#include <Poly_TriangulationParameters.hxx>
occ::handle<Poly_Triangulation> post_triangulation() {
  occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation(3,1,true);mesh->SetNode(1,gp_Pnt(0,0,0.3));mesh->SetNode(2,gp_Pnt(2,0,0.3));mesh->SetNode(3,gp_Pnt(0,3,0.3));mesh->SetUVNode(1,gp_Pnt2d(0,0));mesh->SetUVNode(2,gp_Pnt2d(2,0));mesh->SetUVNode(3,gp_Pnt2d(0,3));mesh->SetTriangle(1,Poly_Triangle(1,2,3));mesh->Deflection(7);return mesh;
}
occ::handle<IMeshData_Model> post_model(const std::array<int,7>& v) {
  if(v[0]<0)return {};auto [unused,unused2,ordered]=order_fixture(0,0,0);occ::handle<IMeshData_Model> model=new BRepMeshData_Model(TopoDS_Shape());auto edge=model->AddEdge(ordered->Edge(1));edge->SetDeflection(0.125);edge->SetStatus(IMeshData_Status(v[1]));
  for(int i=0;i<v[3];++i)edge->GetCurve()->AddPoint(gp_Pnt(i,0,0),i*0.5);if(v[0]==0)return model;
  for(int i=0;i<v[4];++i){auto [wire,face,ordered2]=order_fixture(0,0,0);auto discrete=model->AddFace(face);discrete->SetStatus(IMeshData_Status(v[2]));if(v[5]){auto mesh=post_triangulation();BRepMesh_ShapeTool::AddInFace(face,mesh);}
    for(int j=0;j<v[0];++j){auto curve=edge->AddPCurve(discrete.get(),j==1?TopAbs_REVERSED:TopAbs_FORWARD);for(int k=0;k<2;++k){curve->AddPoint(gp_Pnt2d(k,0),k+j*0.1);curve->GetIndex(k)=k+1+j%2;}}}return model;
}
void report_post_model(const occ::handle<IMeshData_Model>& model) {
  auto edge=model->GetEdge(0)->GetEdge();TopLoc_Location location;auto polygon=BRep_Tool::Polygon3D(edge,location);ga_flag(!polygon.IsNull());if(!polygon.IsNull()){ga_real(polygon->Deflection());ga_int(polygon->NbNodes());for(int i=1;i<=polygon->NbNodes();++i){ga_pnt(polygon->Nodes()(i));ga_real(polygon->Parameters()(i));}}
  for(int f=0;f<model->FacesNb();++f){auto mesh=BRep_Tool::Triangulation(model->GetFace(f)->GetFace(),location);ga_flag(!mesh.IsNull());if(mesh.IsNull())continue;ga_real(mesh->Deflection());auto parameters=mesh->Parameters();ga_flag(!parameters.IsNull());if(!parameters.IsNull()){ga_real(parameters->Deflection());ga_real(parameters->Angle());ga_real(parameters->MinSize());}
    for(auto orient:{TopAbs_FORWARD,TopAbs_REVERSED}){edge.Orientation(orient);auto poly=BRep_Tool::PolygonOnTriangulation(edge,mesh,location);ga_flag(!poly.IsNull());if(poly.IsNull())continue;ga_real(poly->Deflection());ga_int(poly->NbNodes());for(int i=1;i<=poly->NbNodes();++i){ga_int(poly->Node(i));ga_real(poly->Parameter(i));}}}
}
void mesh_post_processor(std::istream& input) {
  std::array<int,7> v;for(auto& x:v)input>>x;auto model=post_model(v);IMeshTools_Parameters p;p.InParallel=v[6];p.Deflection=0.25;p.Angle=0.75;p.MinSize=0.125;
  BRepMesh_ModelPostProcessor processor;ga_flag(processor.Perform(model,p,Message_ProgressRange()));if(!model.IsNull())report_post_model(model);ga_end();
}
