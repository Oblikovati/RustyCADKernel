// HLRBRep_Curve, the Rust twin of crates/tkhlr/examples/hl_probe/hlrbrep_curve.rs.
#include <HLRAlgo.hxx>
#include <HLRBRep_BCurveTool.hxx>
#include <HLRBRep_Curve.hxx>
#include <HLRBRep_CurveTool.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Parab2d.hxx>
void hb_curve_box(HLRBRep_Curve& c){
 double mn[16],mx[16];HLRAlgo::InitMinMax(1e100,mn,mx);
 GA_TRY(ga_real(c.Update(mn,mx));for(double v:mn)ga_real(v);for(double v:mx)ga_real(v));
 ga_int(c.GetType());GA_TRY(ga_real(c.FirstParameter());ga_real(c.LastParameter()));}
void hb_curve_at(const HLRBRep_Curve& c,double t){
 GA_TRY(double p2=c.Parameter2d(t);ga_real(p2);ga_real(c.Parameter3d(p2)));GA_TRY(ga_real(c.Z(t)));
 GA_TRY(gp_Pnt2d p(7,8);c.D0(t,p);ga_pnt2d(p));GA_TRY(ga_pnt2d(c.Value(t)));GA_TRY(ga_pnt(c.Value3D(t)));
 GA_TRY(gp_Pnt2d p(7,8);gp_Vec2d v(1,2);c.D1(t,p,v);ga_pnt2d(p);ga_xy(v.XY()));
 GA_TRY(gp_Pnt2d p(7,8);gp_Vec2d v(1,2),w(3,4);c.D2(t,p,v,w);ga_pnt2d(p);ga_xy(v.XY());ga_xy(w.XY()));}
void hb_curve_conics(const HLRBRep_Curve& c){
 for(bool start:{true,false}){GA_TRY(gp_Pnt2d p;gp_Dir2d d;c.Tangent(start,p,d);ga_pnt2d(p);ga_xy(d.XY()));}
 GA_TRY(hb_lin2d(c.Line()));GA_TRY(hb_circ2d(c.Circle()));GA_TRY(hb_elips2d(c.Ellipse()));
 ga_real(c.Hyperbola().MajorRadius());ga_real(c.Parabola().Focal());ga_xy(c.DN(0.5,1).XY());}
void hb_curve_poles(const HLRBRep_Curve& c){
 GeomAbs_CurveType t=HLRBRep_BCurveTool::GetType(c.GetCurve());
 if(t!=GeomAbs_BezierCurve&&t!=GeomAbs_BSplineCurve)return;
 int n=c.NbPoles();NCollection_Array1<gp_Pnt2d> poles(1,n);NCollection_Array1<double> w(1,n);
 GA_TRY(c.Poles(poles);for(auto& p:poles)ga_pnt2d(p));
 GA_TRY(c.PolesAndWeights(poles,w);for(auto& p:poles)ga_pnt2d(p);for(double x:w)ga_real(x));
 if(t!=GeomAbs_BSplineCurve)return;
 int k=c.NbKnots();NCollection_Array1<double> kn(1,k);NCollection_Array1<int> mu(1,k);
 GA_TRY(c.Knots(kn);for(double x:kn)ga_real(x));GA_TRY(c.Multiplicities(mu);for(int x:mu)ga_int(x));
 occ::handle<Geom_BSplineCurve> b=HLRBRep_BCurveTool::BSpline(c.GetCurve());
 GA_TRY(c.Poles(b,poles);for(auto& p:poles)ga_pnt2d(p));
 GA_TRY(c.PolesAndWeights(b,poles,w);for(auto& p:poles)ga_pnt2d(p);for(double x:w)ga_real(x));}
// `projector edge t1 t2 t3`: Update and the box, then the members at three parameters.
void hb_curve_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);TopoDS_Edge edge=hb_edge(r);
 HLRBRep_Curve c;c.Projector(&projector);c.Curve(edge);hb_curve_box(c);
 for(int i=0;i<3;++i)hb_curve_at(c,r.real());
 hb_curve_conics(c);hb_curve_poles(c);ga_end();}
