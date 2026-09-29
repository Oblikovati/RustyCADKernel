// TKGeomBase oracle: the surface-surface classes of the Extrema package
// (Extrema_FuncExtSS, Extrema_GenExtSS, Extrema_ExtSS). Twin:
// crates/tkgeombase/examples/gb_probe/extrema_ss.rs. Surfaces are read with
// gb_ex_surface (tkgeombase_probe_extrema_ps.cxx).
#include <Extrema_ExtSS.hxx>
#include <Extrema_FuncExtSS.hxx>
#include <Extrema_GenExtSS.hxx>
// ex_fess: <surface1> <surface2> nq (u1 v1 u2 v2)*: Extrema_FuncExtSS: Value,
// Derivatives, Values and a saved state at each point, the saved extrema; the
// default object raises.
void gb_ex_fess_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_surface(r);auto b=gb_ex_surface(r);int n=r.integer();
  Extrema_FuncExtSS d;math_Vector x0(1,4,0.5),f0(1,4,0.0);GB_TRY(gb_flag(d.Value(x0,f0)));GB_TRY(gb_int(d.GetStateNumber()));
  gb_int(d.NbExt());GB_TRY(gb_real(d.SquareDistance(1)));GB_TRY(gb_ponsurf(d.PointOnS1(1)));GB_TRY(gb_ponsurf(d.PointOnS2(1)));
  Extrema_FuncExtSS f(*a,*b);gb_int(f.NbVariables());gb_int(f.NbEquations());GB_TRY(gb_int(f.GetStateNumber()));
  for(int q=0;q<n;++q){math_Vector x(1,4),v(1,4,0.0),w(1,4,0.0);for(int i=1;i<=4;++i)x(i)=r.real();math_Matrix m(1,4,1,4,0.0);
   GB_TRY(gb_flag(f.Value(x,v));for(int i=1;i<=4;++i)gb_real(v(i)));
   GB_TRY(gb_flag(f.Derivatives(x,m));gb_ex_matrix(m));
   GB_TRY(gb_flag(f.Values(x,w,m));for(int i=1;i<=4;++i)gb_real(w(i));gb_ex_matrix(m));
   GB_TRY(gb_int(f.GetStateNumber()));}
  gb_int(f.NbExt());
  for(int i=0;i<=f.NbExt()+1;++i)GB_TRY(gb_real(f.SquareDistance(i));gb_ponsurf(f.PointOnS1(i));gb_ponsurf(f.PointOnS2(i)));
  f.Initialize(*b,*a);gb_int(f.NbExt());
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_genss_result(const Extrema_GenExtSS& e){
 gb_flag(e.IsDone());
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));gb_ponsurf(e.PointOnS1(i));gb_ponsurf(e.PointOnS2(i));});
 GB_TRY(gb_real(e.SquareDistance(0)));
}
// ex_genss: <surface1> <surface2> nbu nbv tol1 tol2 u1 u2 v1 v2 (S1 range) u1 u2 v1 v2
// (S2 range): Extrema_GenExtSS on the surface bounds and on the given ones, and one
// object initialized once and performed twice.
void gb_ex_genss_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_surface(r);auto b=gb_ex_surface(r);int nu=r.integer(),nv=r.integer();double t1=r.real(),t2=r.real();
  double p[8];for(double& x:p)x=r.real();
  GB_TRY(Extrema_GenExtSS e(*a,*b,nu,nv,t1,t2);gb_ex_genss_result(e));
  GB_TRY(Extrema_GenExtSS e(*a,*b,nu,nv,p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7],t1,t2);gb_ex_genss_result(e));
  Extrema_GenExtSS d;gb_flag(d.IsDone());GB_TRY(gb_int(d.NbExt()));
  GB_TRY(d.Initialize(*b,nu,nv,p[4],p[5],p[6],p[7],t2);d.Perform(*a,p[0],p[1],p[2],p[3],t1);gb_ex_genss_result(d);d.Perform(*a,t1);gb_ex_genss_result(d));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_ss_result(const Extrema_ExtSS& e){
 gb_flag(e.IsDone());GB_TRY(gb_flag(e.IsParallel()));
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));GB_TRY(Extrema_POnSurf p,q;e.Points(i,p,q);gb_ponsurf(p);gb_ponsurf(q));});
 GB_TRY(gb_real(e.SquareDistance(0)));
}
// ex_ss: <surface1> <surface2> tol1 tol2 u1 u2 v1 v2 (S1 range) u1 u2 v1 v2 (S2 range):
// Extrema_ExtSS on the surface bounds and on the given ones, and one object
// performed twice (the second time on the bounds of S1).
void gb_ex_ss_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_surface(r);auto b=gb_ex_surface(r);double t1=r.real(),t2=r.real();
  double p[8];for(double& x:p)x=r.real();
  GB_TRY(Extrema_ExtSS e(*a,*b,t1,t2);gb_ex_ss_result(e));
  GB_TRY(Extrema_ExtSS e(*a,*b,p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7],t1,t2);gb_ex_ss_result(e));
  Extrema_ExtSS d;gb_flag(d.IsDone());GB_TRY(gb_int(d.NbExt()));
  GB_TRY(d.Initialize(*b,p[4],p[5],p[6],p[7],t2);d.Perform(*a,p[0],p[1],p[2],p[3],t1);gb_ex_ss_result(d);
   d.Perform(*a,a->FirstUParameter(),a->LastUParameter(),a->FirstVParameter(),a->LastVParameter(),t1);gb_ex_ss_result(d));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
