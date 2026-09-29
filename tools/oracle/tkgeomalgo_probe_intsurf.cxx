// IntSurf probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_surf.rs.
#include <IntSurf.hxx>
#include <IntSurf_PntOn2S.hxx>
#include <IntSurf_Quadric.hxx>
#include <IntSurf_QuadricTool.hxx>
#include <IntSurf_Transition.hxx>
#include <gp_Ax3.hxx>
#include <gp_Vec.hxx>
// Prints a transition: type, then the tangent flag, situation and opposite flag or the
// exception each accessor raises.
void ga_intsurf_transition(const IntSurf_Transition& t){
 ga_int(int(t.TransitionType()));
 GA_TRY(ga_flag(t.IsTangent()));GA_TRY(ga_int(int(t.Situation())));GA_TRY(ga_flag(t.IsOpposite()));
}
// `kind loc dir xdir r1 r2 p u v`: kind 0..4 is a plane, cylinder, sphere, cone (r1 the
// reference radius, r2 the semi-angle) or torus; 5 is the default (undefined) quadric.
IntSurf_Quadric ga_intsurf_quadric(GaReader& r){
 int kind=r.integer();gp_Pnt loc=r.pnt();gp_Dir dir=r.dir(),xdir=r.dir();double r1=r.real(),r2=r.real();
 gp_Ax3 ax3(loc,dir,xdir);
 switch(kind){
  case 0: return IntSurf_Quadric(gp_Pln(ax3));
  case 1: return IntSurf_Quadric(gp_Cylinder(ax3,r1));
  case 2: return IntSurf_Quadric(gp_Sphere(ax3,r1));
  case 3: return IntSurf_Quadric(gp_Cone(ax3,r2,r1));
  case 4: return IntSurf_Quadric(gp_Torus(ax3,r1,r2));
  default: return IntSurf_Quadric();
 }
}
// Implicit members at a point: distance, gradient, ValAndGrad, Normale(P), tolerance.
void ga_intsurf_quadric_point_case(std::istream& input){
 GaReader r{input};IntSurf_Quadric q=ga_intsurf_quadric(r);gp_Pnt p=r.pnt();
 ga_real(q.Distance(p));ga_xyz(q.Gradient(p).XYZ());
 double dist=-7.;gp_Vec grad(-1.,-2.,-3.);q.ValAndGrad(p,dist,grad);ga_real(dist);ga_xyz(grad.XYZ());
 GA_TRY(gp_Vec n=q.Normale(p);ga_xyz(n.XYZ()));
 ga_real(IntSurf_QuadricTool::Tolerance(q));ga_int(int(q.TypeQuadric()));ga_end();
}
// Parametric members at (u, v): Value, D1, DN (1,0) (0,1) (1,1) (2,0), Normale(U, V), Parameters.
void ga_intsurf_quadric_uv_case(std::istream& input){
 GaReader r{input};IntSurf_Quadric q=ga_intsurf_quadric(r);double u=r.real(),v=r.real();
 ga_pnt(q.Value(u,v));
 gp_Pnt p(-1.,-2.,-3.);gp_Vec d1u(-4.,-5.,-6.),d1v(-7.,-8.,-9.);q.D1(u,v,p,d1u,d1v);
 ga_pnt(p);ga_xyz(d1u.XYZ());ga_xyz(d1v.XYZ());
 ga_xyz(q.DN(u,v,1,0).XYZ());ga_xyz(q.DN(u,v,0,1).XYZ());ga_xyz(q.DN(u,v,1,1).XYZ());ga_xyz(q.DN(u,v,2,0).XYZ());
 GA_TRY(gp_Vec n=q.Normale(u,v);ga_xyz(n.XYZ()));
 double pu=-11.,pv=-12.;q.Parameters(q.Value(u,v),pu,pv);ga_real(pu);ga_real(pv);ga_end();
}
// `tg1 tg2 normal`: IntSurf::MakeTransition on two fresh transitions.
void ga_intsurf_transition_case(std::istream& input){
 GaReader r{input};gp_XYZ t1=r.xyz(),t2=r.xyz();gp_Dir n=r.dir();
 IntSurf_Transition first,second;IntSurf::MakeTransition(gp_Vec(t1),gp_Vec(t2),n,first,second);
 ga_intsurf_transition(first);ga_intsurf_transition(second);ga_end();
}
// `p1 u1 v1 u2 v2 p2 u1 v1 u2 v2 tol3d tol2d`: IntSurf_PntOn2S::IsSame.
void ga_intsurf_same_case(std::istream& input){
 GaReader r{input};IntSurf_PntOn2S a,b;
 gp_Pnt pa=r.pnt();double a1=r.real(),a2=r.real(),a3=r.real(),a4=r.real();a.SetValue(pa,a1,a2,a3,a4);
 gp_Pnt pb=r.pnt();double b1=r.real(),b2=r.real(),b3=r.real(),b4=r.real();b.SetValue(pb,b1,b2,b3,b4);
 double tol3=r.real(),tol2=r.real();ga_flag(a.IsSame(b,tol3,tol2));ga_end();
}
