// LocalAnalysis probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/local_analysis.rs.
#include <LocalAnalysis.hxx>
#include <LocalAnalysis_CurveContinuity.hxx>
#include <LocalAnalysis_SurfaceContinuity.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <GeomLProp_SLProps.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir2d.hxx>
#include <sstream>
// `kind ...`: 0 circle (centre, normal, radius), 1 line (point, direction), 2 cubic Bezier.
occ::handle<Geom_Curve> ga_la_curve(GaReader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt c=r.pnt();gp_Dir n=r.dir();return new Geom_Circle(gp_Ax2(c,n),r.real());}
 if(kind==1){gp_Pnt p=r.pnt();return new Geom_Line(p,r.dir());}
 NCollection_Array1<gp_Pnt> poles(1,4);for(int i=1;i<=4;i++)poles(i)=r.pnt();return new Geom_BezierCurve(poles);
}
// `kind point axis r1 r2`: 0 plane, 1 cylinder, 2 sphere, 3 torus.
occ::handle<Geom_Surface> ga_la_surface(GaReader& r){
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir d=r.dir();double r1=r.real(),r2=r.real();gp_Ax3 ax(p,d);
 if(kind==0)return new Geom_Plane(ax);
 if(kind==1)return new Geom_CylindricalSurface(ax,r1);
 if(kind==2)return new Geom_SphericalSurface(ax,r1);
 return new Geom_ToroidalSurface(ax,r1,r2);
}
// The Dump text with its line breaks replaced by '|', as one token stream.
template<class T> void ga_la_dump(const T& c){
 std::ostringstream o;LocalAnalysis::Dump(c,o);std::string s=o.str();
 for(char& ch:s)if(ch=='\n')ch='|';for(char& ch:s)if(ch==' ')ch='_';std::cout<<'"'<<s<<"\" ";
}
// `curve1 u1 curve2 u2 order eps*8`: every query of LocalAnalysis_CurveContinuity.
void ga_la_curve_case(std::istream& input){
 GaReader r{input};occ::handle<Geom_Curve> c1=ga_la_curve(r);double u1=r.real();occ::handle<Geom_Curve> c2=ga_la_curve(r);double u2=r.real();
 int order=r.integer();double e[8];for(double& x:e)x=r.real();
 GA_TRY(LocalAnalysis_CurveContinuity c(c1,u1,c2,u2,GeomAbs_Shape(order),e[0],e[1],e[2],e[3],e[4],e[5],e[6],e[7]);
  ga_flag(c.IsDone());if(!c.IsDone())ga_int(int(c.StatusError()));
  GA_TRY(ga_int(int(c.ContinuityStatus())));GA_TRY(ga_real(c.C0Value()));GA_TRY(ga_real(c.C1Angle()));GA_TRY(ga_real(c.C1Ratio()));
  GA_TRY(ga_real(c.C2Angle()));GA_TRY(ga_real(c.C2Ratio()));GA_TRY(ga_real(c.G1Angle()));GA_TRY(ga_real(c.G2Angle()));
  GA_TRY(ga_real(c.G2CurvatureVariation()));GA_TRY(ga_flag(c.IsC0()));GA_TRY(ga_flag(c.IsC1()));GA_TRY(ga_flag(c.IsC2()));
  GA_TRY(ga_flag(c.IsG1()));GA_TRY(ga_flag(c.IsG2()));ga_la_dump(c));
 ga_end();
}
void ga_la_surface_report(const LocalAnalysis_SurfaceContinuity& c){
 ga_flag(c.IsDone());if(!c.IsDone())ga_int(int(c.StatusError()));
 GA_TRY(ga_int(int(c.ContinuityStatus())));GA_TRY(ga_real(c.C0Value()));GA_TRY(ga_real(c.C1UAngle()));GA_TRY(ga_real(c.C1URatio()));
 GA_TRY(ga_real(c.C1VAngle()));GA_TRY(ga_real(c.C1VRatio()));GA_TRY(ga_real(c.C2UAngle()));GA_TRY(ga_real(c.C2URatio()));
 GA_TRY(ga_real(c.C2VAngle()));GA_TRY(ga_real(c.C2VRatio()));GA_TRY(ga_real(c.G1Angle()));GA_TRY(ga_real(c.G2CurvatureGap()));
 GA_TRY(ga_flag(c.IsC0()));GA_TRY(ga_flag(c.IsC1()));GA_TRY(ga_flag(c.IsC2()));GA_TRY(ga_flag(c.IsG1()));GA_TRY(ga_flag(c.IsG2()));
 ga_la_dump(c);
}
// `mode surface1 surface2 order eps*7 params`: mode 0 points (u1 v1 u2 v2), 1 on two 2d
// lines (p1 d1 p2 d2 U), 2 ComputeAnalysis on local properties (u1 v1 u2 v2).
void ga_la_surface_case(std::istream& input){
 GaReader r{input};int mode=r.integer();occ::handle<Geom_Surface> s1=ga_la_surface(r),s2=ga_la_surface(r);
 int order=r.integer();double e[7];for(double& x:e)x=r.real();GeomAbs_Shape o=GeomAbs_Shape(order);
 GA_TRY(
  if(mode==0){double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();
   ga_la_surface_report(LocalAnalysis_SurfaceContinuity(s1,u1,v1,s2,u2,v2,o,e[0],e[1],e[2],e[3],e[4],e[5],e[6]));}
  else if(mode==1){gp_Pnt2d p1=r.pnt2d();double dx=r.real(),dy=r.real();gp_Pnt2d p2=r.pnt2d();double ex=r.real(),ey=r.real();double U=r.real();
   occ::handle<Geom2d_Curve> l1=new Geom2d_Line(p1,gp_Dir2d(dx,dy)),l2=new Geom2d_Line(p2,gp_Dir2d(ex,ey));
   ga_la_surface_report(LocalAnalysis_SurfaceContinuity(l1,l2,U,s1,s2,o,e[0],e[1],e[2],e[3],e[4],e[5],e[6]));}
  else{double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();
   LocalAnalysis_SurfaceContinuity c(e[0],e[1],e[2],e[3],e[4],e[5],e[6]);
   GeomLProp_SLProps a(s1,u1,v1,2,e[0]),b(s2,u2,v2,2,e[0]);c.ComputeAnalysis(a,b,o);ga_la_surface_report(c);});
 ga_end();
}
