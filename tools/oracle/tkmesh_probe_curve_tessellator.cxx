#include <BRepMesh_CurveTessellator.hxx>
void mesh_curve_tessellator(std::istream& input) {
  std::array<int,8> v;for(auto& x:v)input>>x;try {BRepMesh_ModelBuilder builder;auto model=builder.Perform(model_shape(v[0]),IMeshTools_Parameters());auto edge=model->GetEdge(v[1]%model->EdgesNb());edge->SetShape(edge->GetShape().Oriented(TopAbs_Orientation(v[3])));edge->SetDeflection(v[4]?0.5:0.01);edge->SetAngularDeflection(0.5);IMeshTools_Parameters p;p.MinSize=v[5]?0.001:-1.;p.AdjustMinSize=v[6];
  occ::handle<BRepMesh_CurveTessellator> tessellator;if(v[2]==0)tessellator=new BRepMesh_CurveTessellator(edge,p,v[7]?5:2);else{IMeshData::IFaceHandle face=edge->GetPCurve(0)->GetFace();tessellator=new BRepMesh_CurveTessellator(edge,TopAbs_Orientation(v[3]),face,p,v[7]?5:2);}std::cout<<"ok ";ga_int(tessellator->PointsNb());gp_Pnt point;double parameter=0.;for(int index=1;index<=tessellator->PointsNb();++index){ga_flag(tessellator->Value(index,point,parameter));ga_pnt(point);ga_real(parameter);}}
  catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}ga_end();
}
#include <Geom_BezierSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom2d_Line.hxx>
std::pair<IMeshData::IEdgeHandle,IMeshData::IFaceHandle> refinement_fixture(int kind,bool located,bool internal) {
  BRep_Builder builder;TopoDS_Face face;occ::handle<Geom_Surface> surface;
  if(kind==0)surface=new Geom_CylindricalSurface(gp_Ax3(),1.);else{NCollection_Array2<gp_Pnt> poles(1,2,1,2);for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)poles(i,j)=gp_Pnt(i-1,j-1,0);surface=new Geom_BezierSurface(poles);}
  builder.MakeFace(face,surface,1.e-7);TopoDS_Edge edge;builder.MakeEdge(edge,new Geom_Line(gp_Ax1(gp_Pnt(),gp_Dir(1,0,0))),1.e-7);builder.UpdateEdge(edge,new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0,0.5),gp_Dir2d(1,0))),face,1.e-7);builder.Range(edge,kind==0?0.:-1.,2.,false);
  if(internal){TopoDS_Vertex vertex;builder.MakeVertex(vertex,gp_Pnt(0.37,0,0),1.e-7);vertex.Orientation(TopAbs_INTERNAL);builder.Add(edge,vertex);builder.UpdateVertex(vertex,0.37,edge,1.e-7);}
  gp_Trsf transform;transform.SetTranslation(gp_Vec(located?7.:0.,0,0));TopLoc_Location location(transform);BRepMeshData_Model model{TopoDS_Shape()};auto discreteFace=model.AddFace(TopoDS::Face(face.Located(location)));auto discreteEdge=model.AddEdge(TopoDS::Edge(edge.Located(location)));discreteEdge->AddPCurve(discreteFace.get(),TopAbs_FORWARD);return {discreteEdge,discreteFace};
}
void mesh_curve_refinement(std::istream& input) {
  std::array<int,7> v;for(auto& x:v)input>>x;try {auto [edge,face]=refinement_fixture(v[0],v[1],v[2]);edge->SetDeflection(v[3]?0.5:0.01);edge->SetAngularDeflection(0.5);edge->SetSameParam(v[4]&1);edge->SetSameRange(v[4]&2);IMeshTools_Parameters p;p.MinSize=v[5]?1.:0.001;
    occ::handle<BRepMesh_CurveTessellator> tessellator;if(v[6]==0)tessellator=new BRepMesh_CurveTessellator(edge,p,2);else tessellator=new BRepMesh_CurveTessellator(edge,TopAbs_FORWARD,face,p,2);std::cout<<"ok ";ga_int(tessellator->PointsNb());gp_Pnt point;double parameter=0.;for(int i=1;i<=tessellator->PointsNb();++i){ga_flag(tessellator->Value(i,point,parameter));ga_pnt(point);ga_real(parameter);}}
  catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}ga_end();
}
