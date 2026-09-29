// IntImpParGen probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_imp_par_gen.rs. The intersector template is
// probed through its IntCurve instantiation (tkgeomalgo_probe_intcurve.cxx).
#include <IntImpParGen.hxx>
#include <IntImpParGen_Tool.hxx>
#include <IntCurve_IntConicConic_Tool.hxx>
#include <gp_Vec2d.hxx>
gp_Vec2d ga_vec2d(GaReader& r){double x=r.real();return gp_Vec2d(x,r.real());}
void ga_v2(const gp_Vec2d& v){ga_real(v.X());ga_real(v.Y());}
// `pos1 tan1 norm1 pos2 tan2 norm2`: the transitions of the four DetermineTransition
// variants (IntImpParGen, its crossing-only overload, IntImpParGen_Tool and the line-conic
// one of IntCurve), each from default transitions, with the tangents they leave.
void ga_imppar_transition_case(std::istream& input){
 GaReader r{input};IntRes2d_Position p1=IntRes2d_Position(r.integer());gp_Vec2d t1=ga_vec2d(r),n1=ga_vec2d(r);
 IntRes2d_Position p2=IntRes2d_Position(r.integer());gp_Vec2d t2=ga_vec2d(r),n2=ga_vec2d(r);
 {gp_Vec2d a=t1,b=t2;IntRes2d_Transition x,y;IntImpParGen::DetermineTransition(p1,a,n1,x,p2,b,n2,y,1e-7);
  ga_trans(x);ga_trans(y);ga_v2(a);ga_v2(b);}
 {gp_Vec2d a=t1,b=t2;IntRes2d_Transition x,y;ga_flag(IntImpParGen::DetermineTransition(p1,a,x,p2,b,y,1e-7));
  ga_trans(x);ga_trans(y);}
 {gp_Vec2d a=t1,b=t2;IntRes2d_Transition x,y;Determine_Transition(p1,a,n1,x,p2,b,n2,y,1e-7);
  ga_trans(x);ga_trans(y);ga_v2(a);ga_v2(b);}
 {gp_Vec2d a=t1,b=t2;IntRes2d_Transition x,y;GA_TRY(Determine_Transition_LC(p1,a,n1,x,p2,b,n2,y,1e-7);
  ga_trans(x);ga_trans(y);ga_v2(a));}
 ga_end();
}
// `kind p1 t1 p2 t2 closed z zp x y param`: the position of a point on a domain (both
// variants) and the normalisation of a parameter (IntImpParGen, IntImpParGen_Tool and the
// circle one of IntCurve).
void ga_imppar_domain_case(std::istream& input){
 GaReader r{input};int kind=r.integer();double p1=r.real(),t1=r.real(),p2=r.real(),t2=r.real();
 bool closed=r.flag();double z=r.real(),zp=r.real();gp_Pnt2d point=r.pnt2d();double param=r.real();
 gp_Pnt2d a(p1,0.),b(p2,0.);IntRes2d_Domain d;
 if(kind==1)d.SetValues(a,p1,t1,b,p2,t2);else if(kind==2)d.SetValues(a,p1,t1,true);else if(kind==3)d.SetValues(b,p2,t2,false);
 GA_TRY(if(closed)d.SetEquivalentParameters(z,zp));
 {IntRes2d_Position pos=IntRes2d_Head;GA_TRY(IntImpParGen::DeterminePosition(pos,d,point,param);ga_int(pos));}
 {IntRes2d_Position pos=IntRes2d_Head;GA_TRY(Determine_Position(pos,d,point,param);ga_int(pos));}
 {double u=param;GA_TRY(ga_real(IntImpParGen::NormalizeOnDomain(u,d)));}
 {double u=param;GA_TRY(ga_real(NormalizeOnDomain(u,d)));}
 GA_TRY(ga_real(NormalizeOnCircleDomain(param,d)));
 ga_end();
}
