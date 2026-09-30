#include <TopExp_Explorer.hxx>
TopoDS_Edge bo_tools_edge(double offset) {return BRepBuilderAPI_MakeEdge(gp_Pnt(offset,0,0),gp_Pnt(offset,0,2));}
void bo_tools_wire_face(const TopoDS_Edge& edge,const std::vector<double>& v) {
 BRep_Builder b;TopoDS_Wire w;b.MakeWire(w);b.Add(w,edge.Oriented(TopAbs_Orientation(int(v[3]))));bo_int(IntTools_Tools::HasInternalEdge(w));
 auto f=bo_fclass_face(0,2.);TopoDS_Face output,source=f;source.Orientation(TopAbs_REVERSED);IntTools_Tools::MakeFaceFromWireAndFace(w,source,output);
 bo_int(output.Orientation());bo_int(output.IsSame(source));int count=0;for(TopExp_Explorer e(output,TopAbs_EDGE);e.More();e.Next())++count;bo_int(count);
 bo_int(IntTools_Tools::ClassifyPointByFace(f,gp_Pnt2d(v[0],v[1])));
}
void bo_tools_topology_case(const std::vector<double>& v) {
 try{auto e=bo_tools_edge(0.),other=bo_tools_edge(v[1]);auto vertex=TopoDS::Vertex(TopExp_Explorer(e,TopAbs_VERTEX).Current());
 gp_Pnt p(v[1],0,v[0]);TopoDS_Vertex external=BRepBuilderAPI_MakeVertex(p);BRep_Builder().UpdateVertex(vertex,v[2]);
 bo_int(IntTools_Tools::ComputeVV(vertex,external));bo_int(IntTools_Tools::IsVertex(p,v[2],vertex));bo_int(IntTools_Tools::IsVertex(e,v[0]));bo_int(IntTools_Tools::IsVertex(e,vertex,v[0]));
 bo_int(IntTools_Tools::IsMiddlePointsEqual(e,other));IntTools_CommonPrt part;part.SetEdge1(e);part.SetEdge2(other);part.SetRange1(v[0],v[0]);part.AppendRange2(v[1],v[1]);bo_int(IntTools_Tools::IsVertex(part));
 bo_tools_wire_face(e,v);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
#include <Geom_Plane.hxx>
void bo_tools_surface_case(const std::vector<double>& v) {
 try{occ::handle<Geom_Curve> curve=new Geom_Line(gp_Ax1(gp_Pnt(0,v[0],v[1]),gp_Dir(1,0,0)));
 occ::handle<Geom2d_Curve> pc=new Geom2d_Line(gp_Ax2d());occ::handle<Geom_Surface> surface=new Geom_Plane(gp_Ax3());
 double d=31,t=32;bo_int(IntTools_Tools::ComputeTolerance(curve,pc,surface,v[2],v[3],d,t,v[4],v[5]!=0.));bo_real(d);bo_real(t);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
