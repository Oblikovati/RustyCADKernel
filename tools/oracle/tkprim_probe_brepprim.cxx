// BRepPrim probe operations; the Rust twin is crates/tkprim/examples/pr_probe/brep_prim.rs.
//
// Records (every number read through GaReader):
// - frame `P N X`: gp_Ax2(P, N) when X is the null vector, else gp_Ax2(P, N, X);
// - wedge call `op a b c` (a, b, c are BRepPrim_Direction values, unused ones 0): 0 Shell,
//   1 Face(a), 2 Wire(a), 3 Edge(a,b), 4 Vertex(a,b,c), 5 Open(a), 6 Close(a), 7 Plane(a),
//   8 Line(a,b), 9 Point(a,b,c), 10 HasFace(a), 11 HasWire(a), 12 HasEdge(a,b),
//   13 HasVertex(a,b,c), 14 IsInfinite(a), 15 stop (ends the line before the accessors
//   and the shell, which would crash natively after some build orders);
// - one-axis call `op arg`: 0..29 the topology members from Shell to BottomEndVertex in
//   header order, 30 MakeEmptyLateralFace, 31 MakeEmptyMeridianEdge(arg),
//   32 SetMeridianPCurve on a new meridian edge (angle arg) and a new lateral face,
//   40 Angle(arg), 41 VMin(arg), 42 VMax(arg), 43 SetMeridianOffset(arg), 44 Axes(the
//   current axes rotated by arg around gp::OX()), 50 MeridianValue(arg),
//   51 MeridianOnAxis(arg), 52 MeridianClosed, 53 VMaxInfinite, 54 VMinInfinite, 55 HasTop,
//   56 HasBottom, 57 HasSides, 58 Angle, 59 VMin, 60 VMax, 61 Axes, 99 stop (ends the line
//   before the shell, which would crash natively after some call orders).
// Shapes print through pr_shape (tkprim_probe_shape.cxx).
#include <BRepPrim_Builder.hxx>
#include <BRepPrim_Cone.hxx>
#include <BRepPrim_Cylinder.hxx>
#include <BRepPrim_GWedge.hxx>
#include <BRepPrim_Revolution.hxx>
#include <BRepPrim_Sphere.hxx>
#include <BRepPrim_Torus.hxx>
#include <BRepPrim_Wedge.hxx>
#include <ElCLib.hxx>
#include <gp.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Lin.hxx>
#include <gp_Pln.hxx>
gp_Ax2 pr_ax2(GaReader& r){
 gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_XYZ x=r.xyz();
 if(x.Modulus()==0.)return gp_Ax2(p,n);
 return gp_Ax2(p,n,gp_Dir(x));}
void pr_print_ax2(const gp_Ax2& a){ga_pnt(a.Location());ga_dir(a.Direction());ga_dir(a.XDirection());}
BRepPrim_Direction pr_dir(double v){return BRepPrim_Direction(int(v));}
// One wedge call.
void pr_wedge_call(BRepPrim_GWedge& w,int op,BRepPrim_Direction a,BRepPrim_Direction b,BRepPrim_Direction c){
 switch(op){
  case 0:pr_shape(w.Shell());break;
  case 1:pr_shape(w.Face(a));break;
  case 2:pr_shape(w.Wire(a));break;
  case 3:pr_shape(w.Edge(a,b));break;
  case 4:pr_shape(w.Vertex(a,b,c));break;
  case 5:w.Open(a);break;
  case 6:w.Close(a);break;
  case 7:{gp_Ax3 p=w.Plane(a).Position();ga_pnt(p.Location());ga_dir(p.Direction());ga_dir(p.XDirection());ga_dir(p.YDirection());break;}
  case 8:{gp_Lin l=w.Line(a,b);ga_pnt(l.Location());ga_dir(l.Direction());break;}
  case 9:ga_pnt(w.Point(a,b,c));break;
  case 10:ga_flag(w.HasFace(a));break;
  case 11:ga_flag(w.HasWire(a));break;
  case 12:ga_flag(w.HasEdge(a,b));break;
  case 13:ga_flag(w.HasVertex(a,b,c));break;
  default:ga_flag(w.IsInfinite(a));break;}}
// The accessors of a wedge.
void pr_wedge_state(BRepPrim_GWedge& w){
 pr_print_ax2(w.Axes());
 ga_real(w.GetXMin());ga_real(w.GetYMin());ga_real(w.GetZMin());ga_real(w.GetZ2Min());ga_real(w.GetX2Min());
 ga_real(w.GetXMax());ga_real(w.GetYMax());ga_real(w.GetZMax());ga_real(w.GetZ2Max());ga_real(w.GetX2Max());
 ga_flag(w.IsDegeneratedShape());}
// `kind [frame] values... ncalls (op a b c)...`: kind 0 the default wedge, 1 Wedge(Axes,
// dx, dy, dz), 2 Wedge(Axes, dx, dy, dz, ltx), 3 Wedge(Axes, xmin, ymin, zmin, z2min,
// x2min, xmax, ymax, zmax, z2max, x2max), 4 GWedge(Builder, Axes, dx, dy, dz); then the
// calls, each printed or raising, then the accessors and the shell.
void pr_bp_wedge_case(std::istream& input){
 GaReader r{input};int kind=r.integer();std::unique_ptr<BRepPrim_GWedge> w;
 if(kind==0)w.reset(new BRepPrim_Wedge());
 else{gp_Ax2 a=pr_ax2(r);
  if(kind==1){double dx=r.real(),dy=r.real();w.reset(new BRepPrim_Wedge(a,dx,dy,r.real()));}
  else if(kind==2){double dx=r.real(),dy=r.real(),dz=r.real();w.reset(new BRepPrim_Wedge(a,dx,dy,dz,r.real()));}
  else if(kind==3){double v[10];for(double& x:v)x=r.real();w.reset(new BRepPrim_Wedge(a,v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8],v[9]));}
  else{double dx=r.real(),dy=r.real();w.reset(new BRepPrim_GWedge(BRepPrim_Builder(BRep_Builder()),a,dx,dy,r.real()));}}
 int n=r.integer();
 for(int i=0;i<n;++i){int op=r.integer();BRepPrim_Direction a=pr_dir(r.real()),b=pr_dir(r.real()),c=pr_dir(r.real());
  if(op==15){ga_end();return;}
  GA_TRY(pr_wedge_call(*w,op,a,b,c))}
 pr_wedge_state(*w);GA_TRY(pr_shape(w->Shell()))
 ga_end();}
// `frame P1 P2 radius first last`: every member of BRepPrim_Builder on a face of the
// plane of `frame`: a degenerated edge, the line edge P1-P2 with its two vertices and a
// line p-curve, the circle edge of `radius` with one closed vertex, the range
// first..last and a circle p-curve, a seam edge with two line p-curves, the reversed
// face with the wire of the circle, and the completed shell; then the digests of the
// shell, the degenerated edge, the line edge and the seam edge.
void pr_builder_edges(const BRepPrim_Builder& B,const TopoDS_Face& F,const gp_Ax2& a,const gp_Pnt& p1,const gp_Pnt& p2,TopoDS_Edge& line,TopoDS_Edge& seam){
 B.MakeEdge(line,gp_Lin(p1,gp_Dir(gp_Vec(p1,p2))));TopoDS_Vertex v1,v2;B.MakeVertex(v1,p1);B.MakeVertex(v2,p2);
 B.AddEdgeVertex(line,v1,0.,true);B.AddEdgeVertex(line,v2,p1.Distance(p2),false);
 B.SetPCurve(line,F,gp_Lin2d(gp_Pnt2d(0.,0.),gp_Dir2d(1.,0.)));B.CompleteEdge(line);
 B.MakeEdge(seam,gp_Lin(a.Location(),a.XDirection()));
 B.SetPCurve(seam,F,gp_Lin2d(gp_Pnt2d(0.,1.),gp_Dir2d(0.,1.)),gp_Lin2d(gp_Pnt2d(1.,0.),gp_Dir2d(0.,1.)));}
void pr_bp_builder_case(std::istream& input){
 GaReader r{input};gp_Ax2 a=pr_ax2(r);gp_Pnt p1=r.pnt(),p2=r.pnt();double radius=r.real(),first=r.real(),last=r.real();
 BRepPrim_Builder B(BRep_Builder{});TopoDS_Shell sh;TopoDS_Face F;TopoDS_Wire W;TopoDS_Edge degenerated,line,circle,seam;
 GA_TRY(
  B.MakeShell(sh);ga_flag(sh.Closed());B.MakeFace(F,gp_Pln(a));B.MakeWire(W);B.MakeDegeneratedEdge(degenerated);
  pr_builder_edges(B,F,a,p1,p2,line,seam);
  B.MakeEdge(circle,gp_Circ(a,radius));TopoDS_Vertex v;B.MakeVertex(v,ElCLib::Value(0.,gp_Circ(a,radius)));
  B.AddEdgeVertex(circle,v,0.,2.*M_PI);B.SetParameters(circle,v,first,last);
  B.SetPCurve(circle,F,gp_Circ2d(gp_Ax2d(gp_Pnt2d(0.,0.),gp_Dir2d(1.,0.)),radius));B.CompleteEdge(circle);
  B.AddWireEdge(W,circle,true);B.CompleteWire(W);B.ReverseFace(F);B.AddFaceWire(F,W);B.CompleteFace(F);
  B.AddShellFace(sh,F);B.CompleteShell(sh);
  pr_shape(sh);pr_shape(degenerated);pr_shape(line);pr_shape(seam))
 ga_end();}
