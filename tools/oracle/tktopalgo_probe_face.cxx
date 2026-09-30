// The face reader of the TKTopAlgo oracle: a face built with BRep_Builder from numbers
// (no BRepBuilderAPI, so the Rust twin builds the same shape with its BRep port), and
// the index of an edge among the edges of the face. The Rust twin is
// crates/tktopalgo/examples/tp_probe/face.rs.
//
// A face is `surface tol nverts (u v tol)... nedges (edge)... nwires (wire)... orient`:
// - surface: 0 plane P N; 1 cylinder P N R; 2 sphere P N R; 3 trimmed plane P N u1 u2 v1 v2
//   (gp_Ax3(P, N) frames, Geom_Plane(P, N) for the planes); 4 cone P N R semi-angle;
//   5 torus P N R r; 6 Bezier surface P N nu nv (x y z)... (U rows); 7 B-spline surface
//   P N udeg vdeg nu nv (x y z)... nuk (knot mult)... nvk (knot mult)... (P N unused by 6, 7);
// - a vertex is at the surface point of (u, v), with its tolerance;
// - an edge is `kind vfirst vlast degenerated tol pcurve...`, without a 3d curve: kind 0
//   the segment u1 v1 u2 v2 (a Geom2d_Line on [0, length]), 1 the arc cu cv r a1 a2 sense,
//   2 the Bezier curve n (u v)... on [0, 1], 3 the seam u1 v1 u2 v2 du dv (the segment and
//   its translation by (du, dv) as the pcurves of the FORWARD and REVERSED occurrences),
//   4 the line u v dx dy first last on an explicit range, 5 no pcurve at all (no fields),
//   6 the segment u1 v1 u2 v2 of kind 0 with the 3d line from S(u1, v1) towards S(u2, v2)
//   on the same range (a plane face, whose parametrization is isometric);
//   vfirst and vlast are vertex
//   indices (-1 for none), added FORWARD and REVERSED;
// - a wire is `orientation n (edge orientation)...`; the face orientation comes last.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <NCollection_Array2.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Line.hxx>
#include <NCollection_Array1.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
// A grid of nu x nv poles, U rows first.
NCollection_Array2<gp_Pnt> tp_poles(GaReader& r,int nu,int nv){
 NCollection_Array2<gp_Pnt> poles(1,nu,1,nv);
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)poles(i,j)=r.pnt();
 return poles;}
// `n (knot mult)...`: the knots and multiplicities of one direction.
void tp_knots(GaReader& r,NCollection_Array1<double>& knots,NCollection_Array1<int>& mults){
 int n=r.integer();knots.Resize(1,n,false);mults.Resize(1,n,false);
 for(int i=1;i<=n;++i){knots(i)=r.real();mults(i)=r.integer();}}
occ::handle<Geom_Surface> tp_bspline_surface(GaReader& r){
 int udeg=r.integer(),vdeg=r.integer();int nu=r.integer(),nv=r.integer();
 NCollection_Array2<gp_Pnt> poles=tp_poles(r,nu,nv);
 NCollection_Array1<double> uk(1,1),vk(1,1);NCollection_Array1<int> um(1,1),vm(1,1);
 tp_knots(r,uk,um);tp_knots(r,vk,vm);
 return new Geom_BSplineSurface(poles,uk,vk,um,vm,udeg,vdeg);}
occ::handle<Geom_Surface> tp_surface(GaReader& r){
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir n=r.dir();
 if(kind==0)return new Geom_Plane(p,n);
 if(kind==1)return new Geom_CylindricalSurface(gp_Ax3(p,n),r.real());
 if(kind==2)return new Geom_SphericalSurface(gp_Ax3(p,n),r.real());
 if(kind==4){double radius=r.real();return new Geom_ConicalSurface(gp_Ax3(p,n),r.real(),radius);}
 if(kind==5){double major=r.real();return new Geom_ToroidalSurface(gp_Ax3(p,n),major,r.real());}
 if(kind==6){NCollection_Array2<gp_Pnt> poles=tp_poles(r,r.integer(),r.integer());return new Geom_BezierSurface(poles);}
 if(kind==7)return tp_bspline_surface(r);
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 return new Geom_RectangularTrimmedSurface(new Geom_Plane(p,n),u1,u2,v1,v2);}
// The segment from (u1, v1) to (u2, v2): a line on [0, length].
occ::handle<Geom2d_Curve> tp_segment(const gp_Pnt2d& a,const gp_Pnt2d& b,double& last){
 last=a.Distance(b);return new Geom2d_Line(a,gp_Dir2d(gp_Vec2d(a,b)));}
// The pcurves of one edge on `face` and their range.
void tp_pcurves(GaReader& r,const TopoDS_Edge& e,const TopoDS_Face& face,int kind,double tol){
 BRep_Builder B;double first=0.,last=1.;
 if(kind==5)return;
 if(kind==3){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();double du=r.real(),dv=r.real();
  occ::handle<Geom2d_Curve> c1=tp_segment(a,b,last);
  occ::handle<Geom2d_Curve> c2=new Geom2d_Line(gp_Pnt2d(a.X()+du,a.Y()+dv),gp_Dir2d(gp_Vec2d(a,b)));
  B.UpdateEdge(e,c1,c2,face,tol);B.Range(e,face,first,last);return;}
 occ::handle<Geom2d_Curve> c;
 if(kind==6){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();c=tp_segment(a,b,last);
  occ::handle<Geom_Surface> s=BRep_Tool::Surface(face);gp_Pnt pa=s->Value(a.X(),a.Y()),pb=s->Value(b.X(),b.Y());
  B.UpdateEdge(e,new Geom_Line(pa,gp_Dir(gp_Vec(pa,pb))),tol);}
 else if(kind==0){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();c=tp_segment(a,b,last);}
 else if(kind==1){gp_Pnt2d center=r.pnt2d();double radius=r.real();first=r.real();last=r.real();
  c=new Geom2d_Circle(gp_Ax2d(center,gp_Dir2d(1.,0.)),radius,r.flag());}
 else if(kind==2){int n=r.integer();NCollection_Array1<gp_Pnt2d> poles(1,n);
  for(int i=1;i<=n;++i)poles(i)=r.pnt2d();c=new Geom2d_BezierCurve(poles);}
 else{gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();first=r.real();last=r.real();c=new Geom2d_Line(p,d);}
 B.UpdateEdge(e,c,face,tol);
 if(kind==6)B.Range(e,first,last);else B.Range(e,face,first,last);}
struct TpFace{TopoDS_Face face;std::vector<TopoDS_Edge> edges;};
TopoDS_Edge tp_edge(GaReader& r,const TopoDS_Face& face,const std::vector<TopoDS_Vertex>& vertices){
 BRep_Builder B;TopoDS_Edge e;B.MakeEdge(e);
 int kind=r.integer(),vf=r.integer(),vl=r.integer();bool degenerated=r.flag();double tol=r.real();
 tp_pcurves(r,e,face,kind,tol);
 if(vf>=0)B.Add(e,vertices.at(vf).Oriented(TopAbs_FORWARD));
 if(vl>=0)B.Add(e,vertices.at(vl).Oriented(TopAbs_REVERSED));
 B.Degenerated(e,degenerated);return e;}
TpFace tp_face(GaReader& r){
 BRep_Builder B;TpFace result;occ::handle<Geom_Surface> s=tp_surface(r);
 B.MakeFace(result.face,s,r.real());
 std::vector<TopoDS_Vertex> vertices(r.integer());
 for(TopoDS_Vertex& v:vertices){double u=r.real(),w=r.real();B.MakeVertex(v,s->Value(u,w),r.real());}
 result.edges.resize(r.integer());
 for(TopoDS_Edge& e:result.edges)e=tp_edge(r,result.face,vertices);
 int nwires=r.integer();
 for(int i=0;i<nwires;++i){TopoDS_Wire w;B.MakeWire(w);TopAbs_Orientation o=TopAbs_Orientation(r.integer());
  int n=r.integer();for(int j=0;j<n;++j){int k=r.integer();B.Add(w,result.edges.at(k).Oriented(TopAbs_Orientation(r.integer())));}
  w.Orientation(o);B.Add(result.face,w);}
 result.face.Orientation(TopAbs_Orientation(r.integer()));
 return result;}
// The index of `edge` (IsEqual) among the edges of the FORWARD face, -1 for a null edge,
// -2 when it is not there.
int tp_edge_index(const TopoDS_Face& face,const TopoDS_Shape& edge){
 if(edge.IsNull())return -1;int i=0;
 for(TopExp_Explorer x(face.Oriented(TopAbs_FORWARD),TopAbs_EDGE);x.More();x.Next(),++i)
  if(x.Current().IsEqual(edge))return i;
 return -2;}
