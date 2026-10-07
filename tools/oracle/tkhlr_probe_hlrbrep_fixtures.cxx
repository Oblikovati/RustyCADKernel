// HLRBRep fixtures, the Rust twin of crates/tkhlr/examples/hl_probe/hlrbrep_fixtures.rs:
// projectors and located edges read from a request, and the printers of the 2D conics.
#include <BRep_Builder.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Line.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Lin2d.hxx>
#include <HLRAlgo_Projector.hxx>
#include <NCollection_Array1.hxx>
#include <TopoDS_Edge.hxx>
// `pk [loc dir xdir [focus]]`: pk 0 the parallel projector of the frame, 1 the
// perspective one with `focus`, 2 the default projector (top view).
HLRAlgo_Projector hb_projector(GaReader& r){
 int kind=r.integer();if(kind==2)return HLRAlgo_Projector();
 gp_Pnt loc=r.pnt();gp_Dir dir=r.dir();gp_Dir xdir=r.dir();gp_Ax2 frame(loc,dir,xdir);
 if(kind==0)return HLRAlgo_Projector(frame);
 return HLRAlgo_Projector(frame,r.real());}
// The poles `n p1..pn` of a polynomial curve.
NCollection_Array1<gp_Pnt> hb_poles(GaReader& r){
 int n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);for(int i=1;i<=n;++i)poles(i)=r.pnt();return poles;}
// A degree 2 clamped B-spline on the uniform knots 0..n-2 of its `n` poles.
occ::handle<Geom_Curve> hb_bspline(GaReader& r){
 NCollection_Array1<gp_Pnt> poles=hb_poles(r);int nk=poles.Length()-1;
 NCollection_Array1<double> knots(1,nk);NCollection_Array1<int> mults(1,nk);
 for(int i=1;i<=nk;++i){knots(i)=i-1;mults(i)=(i==1||i==nk)?3:1;}
 return new Geom_BSplineCurve(poles,knots,mults,2,false);}
// `ck ...`: 0 line `loc dir`, 1 circle `loc n x r`, 2 ellipse `loc n x R r`,
// 3 Bezier `poles`, 4 B-spline `poles`.
occ::handle<Geom_Curve> hb_geometry(GaReader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
 if(kind==3)return new Geom_BezierCurve(hb_poles(r));
 if(kind==4)return hb_bspline(r);
 gp_Pnt p=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();gp_Ax2 frame(p,n,x);
 if(kind==1)return new Geom_Circle(frame,r.real());
 double major=r.real();return new Geom_Ellipse(frame,major,r.real());}
// `geometry u1 u2`: an edge without vertices on the range.
TopoDS_Edge hb_edge(GaReader& r){
 occ::handle<Geom_Curve> curve=hb_geometry(r);double u1=r.real(),u2=r.real();
 TopoDS_Edge edge;BRep_Builder b;b.MakeEdge(edge,curve,1e-7);b.Range(edge,u1,u2,false);return edge;}
void hb_lin2d(const gp_Lin2d& l){ga_pnt2d(l.Location());ga_xy(l.Direction().XY());}
void hb_circ2d(const gp_Circ2d& c){ga_pnt2d(c.Location());ga_xy(c.XAxis().Direction().XY());ga_xy(c.YAxis().Direction().XY());ga_real(c.Radius());}
void hb_elips2d(const gp_Elips2d& e){ga_pnt2d(e.Location());ga_xy(e.XAxis().Direction().XY());ga_xy(e.YAxis().Direction().XY());ga_real(e.MajorRadius());ga_real(e.MinorRadius());}
#include <IntRes2d_Domain.hxx>
void hb_domain(const IntRes2d_Domain& d){
 ga_flag(d.HasFirstPoint());if(d.HasFirstPoint()){ga_real(d.FirstParameter());ga_pnt2d(d.FirstPoint());ga_real(d.FirstTolerance());}
 ga_flag(d.HasLastPoint());if(d.HasLastPoint()){ga_real(d.LastParameter());ga_pnt2d(d.LastPoint());ga_real(d.LastTolerance());}
 ga_flag(d.IsClosed());if(d.IsClosed()){double a,b;d.EquivalentParameters(a,b);ga_real(a);ga_real(b);}}
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <IntCurveSurface_IntersectionPoint.hxx>
#include <NCollection_Array2.hxx>
#include <TopoDS_Face.hxx>
// The `nu nv` poles of a spline surface, row by row.
NCollection_Array2<gp_Pnt> hb_surface_poles(GaReader& r){
 int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> poles(1,nu,1,nv);
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)poles(i,j)=r.pnt();return poles;}
// Clamped degree 2 knots of `n` poles: 0..n-2 with end multiplicities 3.
void hb_clamped(int n,NCollection_Array1<double>& k,NCollection_Array1<int>& m){
 for(int i=1;i<=n-1;++i){k(i)=i-1;m(i)=(i==1||i==n-1)?3:1;}}
occ::handle<Geom_Surface> hb_bspline_surface(GaReader& r){
 NCollection_Array2<gp_Pnt> poles=hb_surface_poles(r);int nu=poles.ColLength(),nv=poles.RowLength();
 NCollection_Array1<double> uk(1,nu-1),vk(1,nv-1);NCollection_Array1<int> um(1,nu-1),vm(1,nv-1);
 hb_clamped(nu,uk,um);hb_clamped(nv,vk,vm);return new Geom_BSplineSurface(poles,uk,vk,um,vm,2,2);}
// `sk ...`: 0 plane `loc n x`, 1 cylinder `.. r`, 2 cone `.. angle r`, 3 sphere `.. r`,
// 4 torus `.. R r`, 5 Bezier `nu nv poles`, 6 B-spline `nu nv poles`, 7 revolution of the
// line `p d` around `loc dir`, 8 extrusion of the circle `loc n x r` along `dir`.
occ::handle<Geom_Surface> hb_geom_surface(GaReader& r){
 int kind=r.integer();
 if(kind==5)return new Geom_BezierSurface(hb_surface_poles(r));
 if(kind==6)return hb_bspline_surface(r);
 if(kind==7){gp_Pnt p=r.pnt();gp_Dir d=r.dir();gp_Pnt l=r.pnt();gp_Dir a=r.dir();
  return new Geom_SurfaceOfRevolution(new Geom_Line(p,d),gp_Ax1(l,a));}
 gp_Pnt l=r.pnt();gp_Dir n=r.dir();gp_Dir x=r.dir();gp_Ax3 frame(l,n,x);
 switch(kind){
  case 0:return new Geom_Plane(frame);
  case 1:return new Geom_CylindricalSurface(frame,r.real());
  case 2:{double a=r.real();return new Geom_ConicalSurface(frame,a,r.real());}
  case 3:return new Geom_SphericalSurface(frame,r.real());
  case 4:{double a=r.real();return new Geom_ToroidalSurface(frame,a,r.real());}
  default:{occ::handle<Geom_Curve> c=new Geom_Circle(gp_Ax2(l,n,x),r.real());return new Geom_SurfaceOfLinearExtrusion(c,r.dir());}}}
// `surface umin umax vmin vmax`: the face of the surface on the bounds.
TopoDS_Face hb_face(GaReader& r){
 occ::handle<Geom_Surface> s=hb_geom_surface(r);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 return BRepBuilderAPI_MakeFace(s,u1,u2,v1,v2,1e-7).Face();}
void hb_ics_point(const IntCurveSurface_IntersectionPoint& p){
 ga_pnt(p.Pnt());ga_real(p.U());ga_real(p.V());ga_real(p.W());ga_int(p.Transition());}
