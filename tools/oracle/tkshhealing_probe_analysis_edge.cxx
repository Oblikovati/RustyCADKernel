// ShapeAnalysis_Edge probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/analysis_edge.rs.
//
// `sh_edgeana program op ...` over the shapes, surfaces and locations of a TKTopAlgo shape
// program; each op runs in GA_TRY and the status flags (sh_flags) follow the checks:
//  0 e orient: Curve3d (sh_curve3d), HasCurve3d, IsClosed3d; 1 e f orient: PCurve on the
//  face (sh_curve2d), HasPCurve, BoundUV (flag, points), IsSeam; 2 e s loc orient: the same
//  on a surface and location; 3 e: FirstVertex and LastVertex (IsSame index, orientation);
//  4 e f atend dparam: GetEndTangent2d on the face (flag, point, tangent); 5 e s loc atend
//  dparam: on a surface; 6 e preci vtx: CheckVerticesWithCurve3d; 7 e f preci vtx:
//  CheckVerticesWithPCurve on the face; 8 e s loc preci vtx: on a surface; 9 e f:
//  CheckVertexTolerance on the face (flag, then both tolerances unless FAIL1); 10 e: the same
//  with every p-curve; 11 e f: CheckCurve3dWithPCurve on the face; 12 e s loc: on a surface;
//  13 e nb: CheckSameParameter (flag, maxdev); 14 e f nb: on the face; 15 first last pc:
//  CheckPCurveRange; 16 e1 e2 tol domain: CheckOverlapping (flag, tolerance).
#include <ShapeAnalysis_Edge.hxx>
#include <gp_Vec2d.hxx>
void sh_ea_vertex(const TpShapes& t,const TopoDS_Vertex& v){ga_int(tp_same_index(t,v));ga_int(v.IsNull()?-1:int(v.Orientation()));}
void sh_ea_curves(GaReader& r,const TpShapes& t,ShapeAnalysis_Edge& sae,int op){
 TopoDS_Edge e=sh_edge(t,r.integer());
 if(op==0){bool o=r.flag();occ::handle<Geom_Curve> c;double f=0.,l=0.;ga_flag(sae.Curve3d(e,c,f,l,o));sh_curve3d(c,f,l);
  ga_flag(sae.HasCurve3d(e));ga_flag(sae.IsClosed3d(e));return;}
 if(op==3){sh_ea_vertex(t,sae.FirstVertex(e));sh_ea_vertex(t,sae.LastVertex(e));return;}
 occ::handle<Geom2d_Curve> c;double f=0.,l=0.;gp_Pnt2d a,b;bool ok;
 if(op==1){TopoDS_Face fc=sh_face(t,r.integer());bool o=r.flag();ga_flag(sae.PCurve(e,fc,c,f,l,o));sh_curve2d(c,f,l);
  ga_flag(sae.HasPCurve(e,fc));ok=sae.BoundUV(e,fc,a,b);ga_flag(ok);if(ok){ga_pnt2d(a);ga_pnt2d(b);}ga_flag(sae.IsSeam(e,fc));return;}
 occ::handle<Geom_Surface> s=t.surfaces.at(r.integer());TopLoc_Location L=t.locations.at(r.integer());bool o=r.flag();
 ga_flag(sae.PCurve(e,s,L,c,f,l,o));sh_curve2d(c,f,l);ga_flag(sae.HasPCurve(e,s,L));
 ok=sae.BoundUV(e,s,L,a,b);ga_flag(ok);if(ok){ga_pnt2d(a);ga_pnt2d(b);}ga_flag(sae.IsSeam(e,s,L));}
void sh_ea_tangent(GaReader& r,const TpShapes& t,ShapeAnalysis_Edge& sae,int op){
 TopoDS_Edge e=sh_edge(t,r.integer());gp_Pnt2d p(0.,0.);gp_Vec2d v(0.,0.);bool ok;
 if(op==4){TopoDS_Face f=sh_face(t,r.integer());bool atend=r.flag();ok=sae.GetEndTangent2d(e,f,atend,p,v,r.real());}
 else{occ::handle<Geom_Surface> s=t.surfaces.at(r.integer());TopLoc_Location L=t.locations.at(r.integer());bool atend=r.flag();
  ok=sae.GetEndTangent2d(e,s,L,atend,p,v,r.real());}
 ga_flag(ok);ga_pnt2d(p);sh_vec2d(v);}
void sh_ea_vertices(GaReader& r,const TpShapes& t,ShapeAnalysis_Edge& sae,int op){
 TopoDS_Edge e=sh_edge(t,r.integer());
 if(op==6){double p=r.real();ga_flag(sae.CheckVerticesWithCurve3d(e,p,r.integer()));}
 else if(op==7){TopoDS_Face f=sh_face(t,r.integer());double p=r.real();ga_flag(sae.CheckVerticesWithPCurve(e,f,p,r.integer()));}
 else if(op==8){occ::handle<Geom_Surface> s=t.surfaces.at(r.integer());TopLoc_Location L=t.locations.at(r.integer());
  double p=r.real();ga_flag(sae.CheckVerticesWithPCurve(e,s,L,p,r.integer()));}
 else{double t1=0.,t2=0.;bool ok=op==9?sae.CheckVertexTolerance(e,sh_face(t,r.integer()),t1,t2):sae.CheckVertexTolerance(e,t1,t2);
  ga_flag(ok);if(!sae.Status(ShapeExtend_FAIL1)){ga_real(t1);ga_real(t2);}}
 sh_flags(sae);}
void sh_ea_checks(GaReader& r,const TpShapes& t,ShapeAnalysis_Edge& sae,int op){
 if(op==15){double f=r.real(),l=r.real();ga_flag(sae.CheckPCurveRange(f,l,t.pcurves.at(r.integer())));return;}
 TopoDS_Edge e=sh_edge(t,r.integer());
 if(op==11)ga_flag(sae.CheckCurve3dWithPCurve(e,sh_face(t,r.integer())));
 else if(op==12){occ::handle<Geom_Surface> s=t.surfaces.at(r.integer());ga_flag(sae.CheckCurve3dWithPCurve(e,s,t.locations.at(r.integer())));}
 else if(op==13){double d=0.;ga_flag(sae.CheckSameParameter(e,d,r.integer()));ga_real(d);}
 else if(op==14){TopoDS_Face f=sh_face(t,r.integer());double d=0.;ga_flag(sae.CheckSameParameter(e,f,d,r.integer()));ga_real(d);}
 else{TopoDS_Edge e2=sh_edge(t,r.integer());double tol=r.real();ga_flag(sae.CheckOverlapping(e,e2,tol,r.real()));ga_real(tol);}
 sh_flags(sae);}
void sh_edgeana_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);int op=r.integer();ShapeAnalysis_Edge sae;
 if(op<=3)GA_TRY(sh_ea_curves(r,t,sae,op))
 else if(op<=5)GA_TRY(sh_ea_tangent(r,t,sae,op))
 else if(op<=10)GA_TRY(sh_ea_vertices(r,t,sae,op))
 else GA_TRY(sh_ea_checks(r,t,sae,op))
 ga_end();}
