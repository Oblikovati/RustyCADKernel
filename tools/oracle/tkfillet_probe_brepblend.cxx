// BRepBlend probes of tools/oracle/tkfillet_probe.cxx (included after the BlendFunc
// probes, whose readers they use); the Rust twin is
// crates/tkfillet/examples/fl_probe/brep_blend.rs.
#include <BRepBlend_BlendTool.hxx>
#include <BRepBlend_CurvPointRadInv.hxx>
#include <BRepBlend_HCurve2dTool.hxx>
#include <BRepBlend_HCurveTool.hxx>
#include <BRepBlend_SurfCurvConstRadInv.hxx>
#include <BRepBlend_SurfCurvEvolRadInv.hxx>
#include <BRepBlend_SurfPointConstRadInv.hxx>
#include <BRepBlend_SurfPointEvolRadInv.hxx>
#include <Geom2d_Circle.hxx>
#include <gp_Circ2d.hxx>

// A 2d curve: kind 0 the line through (a, b) along (c, d), 1 the circle of center (a, b)
// and radius c (d unused); 5 numbers.
occ::handle<Adaptor2d_Curve2d> fl_curve2d_kind(GaReader& r){
 int kind=r.integer();double a=r.real(),b=r.real(),c=r.real(),d=r.real();
 if(kind==1)return new Geom2dAdaptor_Curve(new Geom2d_Circle(gp_Circ2d(gp_Ax2d(gp_Pnt2d(a,b),gp_Dir2d(1,0)),c)));
 return new Geom2dAdaptor_Curve(new Geom2d_Line(gp_Pnt2d(a,b),gp_Dir2d(c,d)));}

// `fl_rad_inv kind S C C2 P2d D2d Law radius choix P X1..X3 tol`: kind 0
// SurfPointConstRadInv(S, C), 1 SurfPointEvolRadInv(S, C, Law), 2 SurfCurvConstRadInv(S, C2,
// C), 3 SurfCurvEvolRadInv(S, C2, C, Law), 4 CurvPointRadInv(C, C2) on (X1, X2).
void fl_rad_inv_case(std::istream& in){
 GaReader r{in};int kind=r.integer();
 occ::handle<Adaptor3d_Surface> s=fl_surface(r);occ::handle<Adaptor3d_Curve> c=fl_guide(r),c2=fl_guide(r);
 occ::handle<Adaptor2d_Curve2d> cs=fl_curve2d(r);occ::handle<Law_Function> law=fl_law(r);
 double radius=r.real();int choix=r.integer();gp_Pnt p=r.pnt();
 math_Vector x3(1,3);for(int i=1;i<=3;++i)x3(i)=r.real();double tol=r.real();
 if(kind==0){BRepBlend_SurfPointConstRadInv f(s,c);f.Set(radius,choix);f.Set(p);fl_inverse(f,x3,tol);}
 else if(kind==1){BRepBlend_SurfPointEvolRadInv f(s,c,law);f.Set(choix);f.Set(p);fl_inverse(f,x3,tol);}
 else if(kind==2){BRepBlend_SurfCurvConstRadInv f(s,c2,c);f.Set(radius,choix);f.Set(cs);fl_inverse(f,x3,tol);}
 else if(kind==3){BRepBlend_SurfCurvEvolRadInv f(s,c2,c,law);f.Set(choix);f.Set(cs);fl_inverse(f,x3,tol);}
 else{math_Vector x2(1,2);x2(1)=x3(1);x2(2)=x3(2);BRepBlend_CurvPointRadInv f(c,c2);f.Set(choix);f.Set(p);fl_inverse(f,x2,tol);}
 ga_end();}

// `fl_blend_tool C2d P1 P2 C U0 U1`: the projection of P1 on the 2d curve, the
// intersection of [P1, P2] with it, its bounds, and the sample counts of the tools.
void fl_blend_tool_case(std::istream& in){
 GaReader r{in};
 occ::handle<Adaptor2d_Curve2d> arc=fl_curve2d_kind(r);gp_Pnt2d p1=r.pnt2d(),p2=r.pnt2d();
 occ::handle<Adaptor3d_Curve> c=fl_guide(r);double u0=r.real(),u1=r.real();
 occ::handle<Adaptor3d_Surface> none;
 GA_TRY(double prm,dist;ga_flag(BRepBlend_BlendTool::Project(p1,none,arc,prm,dist));ga_real(prm);ga_real(dist));
 GA_TRY(double prm=0,dist=0;ga_flag(BRepBlend_BlendTool::Inters(p1,p2,none,arc,prm,dist));ga_real(prm);ga_real(dist));
 GA_TRY(double a,b;BRepBlend_BlendTool::Bounds(arc,a,b);ga_real(a);ga_real(b));
 GA_TRY(ga_int(BRepBlend_HCurve2dTool::NbSamples(arc,u0,u1)));
 GA_TRY(ga_int(BRepBlend_HCurveTool::NbSamples(c,u0,u1)));
 ga_end();}
