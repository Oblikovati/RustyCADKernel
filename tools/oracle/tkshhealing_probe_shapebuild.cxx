// ShapeBuild probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/shape_build.rs.
//
// `sh_vertex mode ...`: mode 0 `x1 y1 z1 x2 y2 z2 tol1 tol2 factor` is
// CombineVertex(pnt1, pnt2, tol1, tol2, factor), mode 1 the same through two vertices made
// with BRep_Builder; prints the point and tolerance of the new vertex.
//
// `sh_edge program op ...` over the shapes of a TKTopAlgo shape program (-1: a null
// shape), then the digest of the edges involved:
//  0 e v1 v2: CopyReplaceVertices; 1 to from alpha beta: CopyRanges; 2 e first last:
//  SetRange3d; 3 to from: CopyPCurves; 4 e share: Copy; 5 e f: RemovePCurve(face);
//  6 e s loc: RemovePCurve(surface, location); 7 e s: RemovePCurve(surface);
//  8 e pc f: ReplacePCurve; 9 e old sub: ReassignPCurve; 10 pc trsf ufact first last:
//  TransformPCurve (trsf 0 none, 1 `dx dy` a translation, 2 `x y angle` a rotation, 3
//  `x y s` a scale); 11 e: RemoveCurve3d; 12 e: BuildCurve3d; 13 c loc: MakeEdge(curve, L);
//  14 c loc p1 p2; 15 pc f: MakeEdge(pcurve, face); 16 pc f p1 p2; 17 pc s loc;
//  18 pc s loc p1 p2 (c and pc index the curve and p-curve tables, loc the locations).
#include <ShapeBuild_Edge.hxx>
#include <ShapeBuild_Vertex.hxx>
#include <gp_Trsf2d.hxx>
void sh_vertex_case(std::istream& input){
 GaReader r{input};int mode=r.integer();gp_Pnt p1=r.pnt(),p2=r.pnt();double t1=r.real(),t2=r.real(),f=r.real();
 ShapeBuild_Vertex sbv;TopoDS_Vertex v;
 if(mode==0)v=sbv.CombineVertex(p1,p2,t1,t2,f);
 else{BRep_Builder B;TopoDS_Vertex a,b;B.MakeVertex(a,p1,t1);B.MakeVertex(b,p2,t2);v=sbv.CombineVertex(a,b,f);}
 ga_pnt(BRep_Tool::Pnt(v));ga_real(BRep_Tool::Tolerance(v));ga_end();}
gp_Trsf2d sh_be_trsf(GaReader& r){
 gp_Trsf2d T;int kind=r.integer();
 if(kind==1){double x=r.real();T.SetTranslation(gp_Vec2d(x,r.real()));}
 else if(kind==2){gp_Pnt2d p=r.pnt2d();T.SetRotation(p,r.real());}
 else if(kind==3){gp_Pnt2d p=r.pnt2d();T.SetScale(p,r.real());}
 return T;}
void sh_be_pcurve(const occ::handle<Geom2d_Curve>& c,double f,double l){
 tp_type(c);ga_real(f);ga_real(l);if(!c.IsNull())tp_curve2d_points(c,f,l);}
void sh_be_make(GaReader& r,const TpShapes& t,int op,TopoDS_Edge& e){
 ShapeBuild_Edge sbe;
 if(op==13){occ::handle<Geom_Curve> c=t.curves.at(r.integer());sbe.MakeEdge(e,c,t.locations.at(r.integer()));}
 else if(op==14){occ::handle<Geom_Curve> c=t.curves.at(r.integer());TopLoc_Location L=t.locations.at(r.integer());
  double p1=r.real();sbe.MakeEdge(e,c,L,p1,r.real());}
 else if(op==15){occ::handle<Geom2d_Curve> c=t.pcurves.at(r.integer());sbe.MakeEdge(e,c,sh_face(t,r.integer()));}
 else if(op==16){occ::handle<Geom2d_Curve> c=t.pcurves.at(r.integer());TopoDS_Face f=sh_face(t,r.integer());
  double p1=r.real();sbe.MakeEdge(e,c,f,p1,r.real());}
 else if(op==17){occ::handle<Geom2d_Curve> c=t.pcurves.at(r.integer());occ::handle<Geom_Surface> s=t.surfaces.at(r.integer());
  sbe.MakeEdge(e,c,s,t.locations.at(r.integer()));}
 else{occ::handle<Geom2d_Curve> c=t.pcurves.at(r.integer());occ::handle<Geom_Surface> s=t.surfaces.at(r.integer());
  TopLoc_Location L=t.locations.at(r.integer());double p1=r.real();sbe.MakeEdge(e,c,s,L,p1,r.real());}}
// The ops that change or copy edges; prints the digests of the edges involved.
void sh_be_edit(GaReader& r,const TpShapes& t,int op){
 ShapeBuild_Edge sbe;
 if(op==0){TopoDS_Edge e=sh_edge(t,r.integer());TopoDS_Vertex v1=TopoDS::Vertex(sh_shape(t,r.integer()));
  TopoDS_Vertex v2=TopoDS::Vertex(sh_shape(t,r.integer()));tp_shape(sbe.CopyReplaceVertices(e,v1,v2));tp_shape(e);}
 else if(op==1){TopoDS_Edge to=sh_edge(t,r.integer()),from=sh_edge(t,r.integer());double a=r.real();
  sbe.CopyRanges(to,from,a,r.real());tp_shape(to);tp_shape(from);}
 else if(op==2){TopoDS_Edge e=sh_edge(t,r.integer());double f=r.real();sbe.SetRange3d(e,f,r.real());tp_shape(e);}
 else if(op==3){TopoDS_Edge to=sh_edge(t,r.integer()),from=sh_edge(t,r.integer());sbe.CopyPCurves(to,from);tp_shape(to);tp_shape(from);}
 else{TopoDS_Edge e=sh_edge(t,r.integer());TopoDS_Edge c=sbe.Copy(e,r.flag());tp_shape(c);ga_flag(c.IsSame(e));}}
void sh_be_pcurves(GaReader& r,const TpShapes& t,int op){
 ShapeBuild_Edge sbe;TopoDS_Edge e=sh_edge(t,r.integer());
 if(op==5)sbe.RemovePCurve(e,sh_face(t,r.integer()));
 else if(op==6){occ::handle<Geom_Surface> s=t.surfaces.at(r.integer());sbe.RemovePCurve(e,s,t.locations.at(r.integer()));}
 else if(op==7)sbe.RemovePCurve(e,t.surfaces.at(r.integer()));
 else if(op==8){occ::handle<Geom2d_Curve> c=t.pcurves.at(r.integer());sbe.ReplacePCurve(e,c,sh_face(t,r.integer()));}
 else if(op==9){TopoDS_Face old=sh_face(t,r.integer());ga_flag(sbe.ReassignPCurve(e,old,sh_face(t,r.integer())));}
 else if(op==11)sbe.RemoveCurve3d(e);
 else ga_flag(sbe.BuildCurve3d(e));
 tp_shape(e);}
void sh_edge_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);int op=r.integer();
 if(op<=4)GA_TRY(sh_be_edit(r,t,op))
 else if(op==10){occ::handle<Geom2d_Curve> c=t.pcurves.at(r.integer());gp_Trsf2d T=sh_be_trsf(r);double uf=r.real();
  double f=r.real(),l=r.real();GA_TRY(occ::handle<Geom2d_Curve> res=ShapeBuild_Edge().TransformPCurve(c,T,uf,f,l);sh_be_pcurve(res,f,l))}
 else if(op<=12)GA_TRY(sh_be_pcurves(r,t,op))
 else{TopoDS_Edge e;GA_TRY(sh_be_make(r,t,op,e);tp_shape(e))}
 ga_end();}
