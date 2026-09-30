// Context compute statuses, untouched outputs, endpoint refinement and pcurve preference.
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <IntTools_Tools.hxx>
TopoDS_Edge bo_context_edge(int kind) {
 if(kind==0)return BRepBuilderAPI_MakeEdge(gp_Pnt(),gp_Pnt(0.,0.,2.));
 TopoDS_Edge edge;BRep_Builder builder;builder.MakeEdge(edge);if(kind==2)builder.Degenerated(edge,true);return edge;
}
void bo_context_computations(IntTools_Context& c,const std::vector<double>& v,const gp_Pnt& p) {
 auto edge=bo_context_edge(int(v[0]));TopoDS_Vertex vertex=BRepBuilderAPI_MakeVertex(p);
 double t=31.,d=32.;bo_int(c.ComputePE(p,v[4],edge,t,d));bo_real(t);bo_real(d);
 t=31.;d=32.;bo_int(c.ComputeVE(vertex,edge,t,d,v[5]));bo_real(t);bo_real(d);
 auto f=bo_fclass_face(0,2.);double u=33.,w=34.;d=32.;bo_int(c.ComputeVF(vertex,f,u,w,d,v[5]));bo_real(u);bo_real(w);bo_real(d);
 if(v[0]==0.){t=31.;bo_int(c.ProjectPointOnEdge(p,edge,t));bo_real(t);}
}
void bo_context_curves(IntTools_Context& c,const std::vector<double>& v,const gp_Pnt& p) {
 auto geom=bo_curve_geometry(int(v[6]));IntTools_Curve curve;curve.SetCurve(geom);TopoDS_Vertex vertex=BRepBuilderAPI_MakeVertex(p);
 double t=31.;bo_int(c.IsVertexOnLine(vertex,curve,v[4],t));bo_real(t);t=31.;bo_int(c.IsVertexOnLine(vertex,v[5],curve,v[4],t));bo_real(t);
 auto f=bo_fclass_face(0,2.);bo_real(IntTools_Tools::IntermediatePoint(v[7],v[8]));bo_int(c.IsValidBlockForFace(v[7],v[8],curve,f,v[4]));
 occ::handle<Geom2d_Curve> pc=new Geom2d_Line(gp_Ax2d()),null;
 for(int mask=0;mask<4;++mask){curve.SetFirstCurve2d(mask&1?pc:null);curve.SetSecondCurve2d(mask&2?pc:null);bo_int(c.IsValidBlockForFaces(v[7],v[8],curve,f,f,v[4]));}
}
void bo_context_compute_case(const std::vector<double>& v) {
 try{IntTools_Context c;gp_Pnt p(v[1],v[2],v[3]);bo_context_computations(c,v,p);bo_context_curves(c,v,p);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
