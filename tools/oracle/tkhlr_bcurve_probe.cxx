// HLRBRep_BCurveTool oracle against unchanged OCCT 3d097a0.
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <HLRBRep_BCurveTool.hxx>
#include <BRep_Builder.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <TopoDS_Edge.hxx>
#include <TopLoc_Location.hxx>
#include <Geom_Line.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_OffsetCurve.hxx>
#include "tkgeomalgo_probe_common.cxx"
using BCT=HLRBRep_BCurveTool;
occ::handle<Geom_Curve> bct_spline(int kind) {
 NCollection_Array1<gp_Pnt> poles(1,4);NCollection_Array1<double> weights(1,4);
 for(int i=1;i<=4;++i){poles(i)=gp_Pnt(i-1,i%2,i*i);weights(i)=i==2?2.:1.;}
 if(kind==5)return new Geom_BezierCurve(poles);if(kind==6)return new Geom_BezierCurve(poles,weights);
 NCollection_Array1<double> knots(1,3);NCollection_Array1<int> mults(1,3);
 for(int i=1;i<=3;++i){knots(i)=i-1;mults(i)=i==2?1:3;}
 if(kind==7)return new Geom_BSplineCurve(poles,knots,mults,2,false);
 return new Geom_BSplineCurve(poles,weights,knots,mults,2,false);
}
occ::handle<Geom_Curve> bct_geometry(int kind) {
 switch(kind){case 0:return new Geom_Line(gp_Ax1());case 1:return new Geom_Circle(gp_Ax2(),2.);
 case 2:return new Geom_Ellipse(gp_Ax2(),3.,2.);case 3:return new Geom_Hyperbola(gp_Ax2(),3.,2.);
 case 4:return new Geom_Parabola(gp_Ax2(),1.);case 5:case 6:case 7:case 8:return bct_spline(kind);
 case 10:return bct_spline(7);
 default:return new Geom_OffsetCurve(new Geom_Line(gp_Ax1()),.5,gp_Dir(0,1,0),false);}
}
BRepAdaptor_Curve bct_curve(int kind,double shift) {
 TopoDS_Edge edge;BRep_Builder b;b.MakeEdge(edge,bct_geometry(kind),1e-7);b.Range(edge,0.,kind==10?0.:1.,false);
 gp_Trsf transform;transform.SetTranslation(gp_Vec(shift,2*shift,-shift));edge.Location(TopLoc_Location(transform),true);
 return BRepAdaptor_Curve(edge);
}
void bct_metadata(const BRepAdaptor_Curve& c) {
 GA_TRY(ga_real(BCT::FirstParameter(c));ga_real(BCT::LastParameter(c));ga_int(BCT::GetType(c));ga_int(BCT::Continuity(c)));
 GA_TRY(ga_flag(BCT::IsClosed(c));ga_flag(BCT::IsPeriodic(c)));GA_TRY(ga_real(BCT::Period(c)));
 GA_TRY(ga_int(BCT::Degree(c)));GA_TRY(ga_int(BCT::NbPoles(c)));GA_TRY(ga_int(BCT::NbKnots(c)));GA_TRY(ga_flag(BCT::IsRational(c)));
 for(auto shape:{GeomAbs_C0,GeomAbs_C1,GeomAbs_C2,GeomAbs_C3,GeomAbs_CN}){GA_TRY(int n=BCT::NbIntervals(c,shape);ga_int(n);NCollection_Array1<double> bounds(-1,n-1);BCT::Intervals(c,bounds,shape);for(double v:bounds)ga_real(v));}
}
void bct_evaluation(const BRepAdaptor_Curve& c,double t) {
 GA_TRY(ga_pnt(BCT::Value(c,t));gp_Pnt p;BCT::D0(c,t,p);ga_pnt(p));
 GA_TRY(gp_Pnt p;gp_Vec v;BCT::D1(c,t,p,v);ga_pnt(p);ga_xyz(v.XYZ()));
 GA_TRY(gp_Pnt p;gp_Vec v,w;BCT::D2(c,t,p,v,w);ga_pnt(p);ga_xyz(v.XYZ());ga_xyz(w.XYZ()));
 GA_TRY(gp_Pnt p;gp_Vec v,w,z;BCT::D3(c,t,p,v,w,z);ga_pnt(p);ga_xyz(v.XYZ());ga_xyz(w.XYZ());ga_xyz(z.XYZ()));
 for(int n:{0,1,2,3,5}){GA_TRY(ga_xyz(BCT::DN(c,t,n).XYZ()));}
 for(double tolerance:{-1.,0.,1e-7,2.}){GA_TRY(ga_real(BCT::Resolution(c,tolerance)));}
}
void bct_conics(const BRepAdaptor_Curve& c) {
 GA_TRY(auto v=BCT::Line(c);ga_pnt(v.Location());ga_xyz(v.Direction().XYZ()));
 GA_TRY(auto v=BCT::Circle(c);ga_pnt(v.Location());ga_real(v.Radius()));
 GA_TRY(auto v=BCT::Ellipse(c);ga_pnt(v.Location());ga_real(v.MajorRadius());ga_real(v.MinorRadius()));
 GA_TRY(auto v=BCT::Hyperbola(c);ga_pnt(v.Location());ga_real(v.MajorRadius());ga_real(v.MinorRadius()));
 GA_TRY(auto v=BCT::Parabola(c);ga_pnt(v.Location());ga_real(v.Focal()));
 GA_TRY(ga_pnt(BCT::Bezier(c)->Value(.5)));GA_TRY(ga_pnt(BCT::BSpline(c)->Value(.5)));
}
void bct_poles(const BRepAdaptor_Curve& c,int pl,int pu,int wl,int wu) {
 NCollection_Array1<gp_Pnt> poles(pl,pu);NCollection_Array1<double> weights(wl,wu);poles.Init(gp_Pnt(7,8,9));weights.Init(17.);
 GA_TRY(BCT::Poles(c,poles));for(const auto& p:poles)ga_pnt(p);
 GA_TRY(BCT::PolesAndWeights(c,poles,weights));for(const auto& p:poles)ga_pnt(p);for(double w:weights)ga_real(w);
}
void bct_case(std::istream& input) {
 GaReader r{input};GA_TRY(int kind=r.integer();double shift=r.real(),t=r.real(),lo=r.real(),hi=r.real();int pl=r.integer(),pu=r.integer(),wl=r.integer(),wu=r.integer();
 auto c=bct_curve(kind,shift);bct_metadata(c);bct_evaluation(c,t);bct_conics(c);bct_poles(c,pl,pu,wl,wu);GA_TRY(ga_int(BCT::NbSamples(c,lo,hi))));ga_end();
}
int main() {
 std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);std::string operation;input>>operation;
 if(operation.empty())continue;if(operation!="bcurve"){std::cerr<<"Unknown probe operation: "<<operation<<std::endl;return 2;}bct_case(input);}
}
