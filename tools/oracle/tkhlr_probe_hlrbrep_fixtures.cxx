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
