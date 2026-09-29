// TKGeomBase oracle: the curve-surface classes of the Extrema package
// (Extrema_FuncExtCS, Extrema_GlobOptFuncCS, Extrema_GlobOptFuncConicS,
// Extrema_GlobOptFuncCQuadric, Extrema_GenExtCS, Extrema_ExtCS). Twin:
// crates/tkgeombase/examples/gb_probe/extrema_cs.rs. Curves are read with
// gb_ex_adaptor, surfaces with gb_ex_surface (tkgeombase_probe_extrema_ps.cxx).
#include <Extrema_ExtCS.hxx>
#include <Extrema_FuncExtCS.hxx>
#include <Extrema_GenExtCS.hxx>
#include <Extrema_GlobOptFuncCQuadric.hxx>
#include <Extrema_GlobOptFuncConicS.hxx>
#include <Extrema_GlobOptFuncCS.hxx>
void gb_ex_matrix(const math_Matrix& m){for(int i=m.LowerRow();i<=m.UpperRow();++i)for(int j=m.LowerCol();j<=m.UpperCol();++j)gb_real(m(i,j));}
// ex_fecs: <curve> <surface> nq (t u v)*: Extrema_FuncExtCS: Value, Derivatives, Values
// and a saved state at each triple, the saved extrema; the default object raises.
void gb_ex_fecs_case(std::istream& in){
 GbReader r{in};
 try{
  auto c=gb_ex_adaptor<GbSpace3>(r);auto s=gb_ex_surface(r);int n=r.integer();
  Extrema_FuncExtCS d;math_Vector x0(1,3,0.5),f0(1,3,0.0);GB_TRY(gb_flag(d.Value(x0,f0)));GB_TRY(gb_int(d.GetStateNumber()));
  gb_int(d.NbExt());GB_TRY(gb_real(d.SquareDistance(1)));GB_TRY(gb_poncurv(d.PointOnCurve(1)));GB_TRY(gb_ponsurf(d.PointOnSurface(1)));
  Extrema_FuncExtCS f(*c,*s);gb_int(f.NbVariables());gb_int(f.NbEquations());
  for(int q=0;q<n;++q){math_Vector x(1,3),v(1,3,0.0),w(1,3,0.0);x(1)=r.real();x(2)=r.real();x(3)=r.real();math_Matrix m(1,3,1,3,0.0);
   GB_TRY(gb_flag(f.Value(x,v));gb_real(v(1));gb_real(v(2));gb_real(v(3)));
   GB_TRY(gb_flag(f.Derivatives(x,m));gb_ex_matrix(m));
   GB_TRY(gb_flag(f.Values(x,w,m));gb_real(w(1));gb_real(w(2));gb_real(w(3));gb_ex_matrix(m));
   GB_TRY(gb_int(f.GetStateNumber()));}
  gb_int(f.NbExt());
  for(int i=0;i<=f.NbExt()+1;++i)GB_TRY(gb_real(f.SquareDistance(i));gb_poncurv(f.PointOnCurve(i));gb_ponsurf(f.PointOnSurface(i)));
  f.SquareDistances().Clear();gb_int(f.NbExt());gb_int(f.PointsOnCurve().Length());gb_int(f.PointsOnSurf().Length());
  f.Initialize(*c,*s);gb_int(f.NbExt());
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_gofcs: <curve> <surface> nq (t u v)*: Extrema_GlobOptFuncCS.
void gb_ex_gofcs_case(std::istream& in){
 GbReader r{in};
 try{
  auto c=gb_ex_adaptor<GbSpace3>(r);auto s=gb_ex_surface(r);int n=r.integer();
  Extrema_GlobOptFuncCS f(c.get(),s.get());gb_int(f.NbVariables());
  for(int q=0;q<n;++q){math_Vector x(1,3),g(1,3,0.0);x(1)=r.real();x(2)=r.real();x(3)=r.real();math_Matrix h(1,3,1,3,0.0);double v=0;
   GB_TRY(gb_flag(f.Value(x,v));gb_real(v));
   GB_TRY(gb_flag(f.Gradient(x,g));gb_real(g(1));gb_real(g(2));gb_real(g(3)));
   GB_TRY(v=0;gb_flag(f.Values(x,v,g));gb_real(v);gb_real(g(1));gb_real(g(2));gb_real(g(3)));
   GB_TRY(v=0;gb_flag(f.Values(x,v,g,h));gb_real(v);gb_ex_matrix(h));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_gofconics: <curve> <surface> tf tl uf ul vf vl nq (u v)*: Extrema_GlobOptFuncConicS
// on the curve and surface ranges and on the given ones.
void gb_ex_gofconics_case(std::istream& in){
 GbReader r{in};
 try{
  auto c=gb_ex_adaptor<GbSpace3>(r);auto s=gb_ex_surface(r);double tf=r.real(),tl=r.real(),uf=r.real(),ul=r.real(),vf=r.real(),vl=r.real();int n=r.integer();
  Extrema_GlobOptFuncConicS f(c.get(),s.get());Extrema_GlobOptFuncConicS g(s.get(),uf,ul,vf,vl);g.LoadConic(c.get(),tf,tl);gb_int(f.NbVariables());
  for(int q=0;q<n;++q){math_Vector x(1,2);x(1)=r.real();x(2)=r.real();
   for(Extrema_GlobOptFuncConicS* e:{&f,&g}){double v=0;GB_TRY(gb_flag(e->Value(x,v));gb_real(v));GB_TRY(gb_real(e->ConicParameter(x)));}}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_gofcquad: <curve> <surface> tf tl uf ul vf vl nq t*: Extrema_GlobOptFuncCQuadric on
// the curve and surface ranges and on the given ones.
void gb_ex_gofcquad_case(std::istream& in){
 GbReader r{in};
 try{
  auto c=gb_ex_adaptor<GbSpace3>(r);auto s=gb_ex_surface(r);double tf=r.real(),tl=r.real(),uf=r.real(),ul=r.real(),vf=r.real(),vl=r.real();int n=r.integer();
  Extrema_GlobOptFuncCQuadric f(c.get(),s.get());Extrema_GlobOptFuncCQuadric g(c.get(),tf,tl);g.LoadQuad(s.get(),uf,ul,vf,vl);gb_int(f.NbVariables());
  for(int q=0;q<n;++q){math_Vector x(1,1);x(1)=r.real();
   for(Extrema_GlobOptFuncCQuadric* e:{&f,&g}){double v=0;math_Vector uv(1,2,0.0);GB_TRY(gb_flag(e->Value(x,v));gb_real(v));GB_TRY(e->QuadricParameters(x,uv);gb_real(uv(1));gb_real(uv(2)));}}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_gencs_result(const Extrema_GenExtCS& e){
 gb_flag(e.IsDone());
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));gb_poncurv(e.PointOnCurve(i));gb_ponsurf(e.PointOnSurface(i));});
 GB_TRY(gb_real(e.SquareDistance(0)));
}
// ex_genecs: <curve> <surface> nbt nbu nbv tol1 tol2 tmin tsup umin usup vmin vsup:
// Extrema_GenExtCS on the ranges of the geometry and on the given ones, and one
// object initialized once and performed twice.
void gb_ex_genecs_case(std::istream& in){
 GbReader r{in};
 try{
  auto c=gb_ex_adaptor<GbSpace3>(r);auto s=gb_ex_surface(r);int nt=r.integer(),nu=r.integer(),nv=r.integer();double t1=r.real(),t2=r.real();
  double a=r.real(),b=r.real(),u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
  GB_TRY(Extrema_GenExtCS e(*c,*s,nt,nu,nv,t1,t2);gb_ex_gencs_result(e));
  GB_TRY(Extrema_GenExtCS e(*c,*s,nt,nu,nv,a,b,u1,u2,v1,v2,t1,t2);gb_ex_gencs_result(e));
  Extrema_GenExtCS d;gb_flag(d.IsDone());GB_TRY(gb_int(d.NbExt()));
  GB_TRY(d.Initialize(*s,nu,nv,u1,u2,v1,v2,t2);d.Perform(*c,nt,a,b,t1);gb_ex_gencs_result(d);d.Perform(*c,nt,t1);gb_ex_gencs_result(d));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_cs_result(const Extrema_ExtCS& e){
 gb_flag(e.IsDone());GB_TRY(gb_flag(e.IsParallel()));
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));GB_TRY(Extrema_POnCurv p;Extrema_POnSurf q;e.Points(i,p,q);gb_poncurv(p);gb_ponsurf(q));});
 GB_TRY(gb_real(e.SquareDistance(0)));
}
// ex_cs: <curve> <surface> tolc tols ucinf ucsup uinf usup vinf vsup: Extrema_ExtCS on the
// ranges of the geometry and on the given ones, and one object performed twice.
void gb_ex_cs_case(std::istream& in){
 GbReader r{in};
 try{
  auto c=gb_ex_adaptor<GbSpace3>(r);auto s=gb_ex_surface(r);double tc=r.real(),ts=r.real();
  double a=r.real(),b=r.real(),u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
  GB_TRY(Extrema_ExtCS e(*c,*s,tc,ts);gb_ex_cs_result(e));
  GB_TRY(Extrema_ExtCS e(*c,*s,a,b,u1,u2,v1,v2,tc,ts);gb_ex_cs_result(e));
  Extrema_ExtCS d;gb_flag(d.IsDone());GB_TRY(gb_int(d.NbExt()));
  GB_TRY(d.Initialize(*s,tc,ts);d.Perform(*c,a,b);gb_ex_cs_result(d);d.Perform(*c,c->FirstParameter(),c->LastParameter());gb_ex_cs_result(d));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
