// TKGeomBase oracle: the curve-curve classes of the Extrema package (Extrema_ExtCC,
// Extrema_ExtCC2d, Extrema_ECC(2d), Extrema_LocateExtCC, Extrema_LocECC,
// Extrema_CCLocFOfLocECC and the Extrema_GlobOptFuncCC functions). Twin:
// crates/tkgeombase/examples/gb_probe/extrema_cc.rs. Adaptors are read with
// gb_ex_adaptor (tkgeombase_probe_extrema_pc_classic.cxx).
#include <Extrema_CCLocFOfLocECC.hxx>
#include <Extrema_ECC.hxx>
#include <Extrema_ECC2d.hxx>
#include <Extrema_ExtCC.hxx>
#include <Extrema_ExtCC2d.hxx>
#include <Extrema_GlobOptFuncCC.hxx>
#include <Extrema_LocECC.hxx>
#include <Extrema_LocateExtCC.hxx>
#include <math_Matrix.hxx>
template<class S,class E> void gb_ex_cc_result(const E& e){
 gb_flag(e.IsDone());GB_TRY(gb_flag(e.IsParallel()));
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));GB_TRY(typename S::POn a,b;e.Points(i,a,b);S::pon(a);S::pon(b));});
 GB_TRY(gb_real(e.SquareDistance(0)));
}
template<class S,class E> void gb_ex_cc_trimmed(const E& e){
 double d[4];typename S::Pnt p[4];e.TrimmedSquareDistances(d[0],d[1],d[2],d[3],p[0],p[1],p[2],p[3]);
 for(int i=0;i<4;++i){gb_real(d[i]);S::pnt(p[i]);}
}
// ex_cc: <adaptor1> <adaptor2> u1 u2 v1 v2 single: the constructor on the adaptor
// ranges, the constructor on the given ranges, then one object initialized on the
// given ranges performing twice.
void gb_ex_cc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<GbSpace3>(r);auto b=gb_ex_adaptor<GbSpace3>(r);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();bool single=r.flag();
  GB_TRY(Extrema_ExtCC e(*a,*b);gb_ex_cc_result<GbSpace3>(e);if(e.IsDone())gb_ex_cc_trimmed<GbSpace3>(e));
  GB_TRY(Extrema_ExtCC e(*a,*b,u1,u2,v1,v2);gb_ex_cc_result<GbSpace3>(e);if(e.IsDone())gb_ex_cc_trimmed<GbSpace3>(e));
  Extrema_ExtCC e(1.0e-9,1.0e-8);e.SetSingleSolutionFlag(single);gb_flag(e.GetSingleSolutionFlag());
  GB_TRY(e.Initialize(*a,*b,u1,u2,v1,v2);e.Perform();gb_ex_cc_result<GbSpace3>(e);if(e.IsDone())gb_ex_cc_trimmed<GbSpace3>(e);
   e.Perform();gb_ex_cc_result<GbSpace3>(e));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_cc2d: the same for Extrema_ExtCC2d (Initialize(C2, V1, V2), Perform(C1, U1, U2)).
void gb_ex_cc2d_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<GbSpace2>(r);auto b=gb_ex_adaptor<GbSpace2>(r);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();bool single=r.flag();
  GB_TRY(Extrema_ExtCC2d e(*a,*b);gb_ex_cc_result<GbSpace2>(e);if(e.IsDone())gb_ex_cc_trimmed<GbSpace2>(e));
  GB_TRY(Extrema_ExtCC2d e(*a,*b,u1,u2,v1,v2);gb_ex_cc_result<GbSpace2>(e);if(e.IsDone())gb_ex_cc_trimmed<GbSpace2>(e));
  Extrema_ExtCC2d e;e.SetSingleSolutionFlag(single);gb_flag(e.GetSingleSolutionFlag());
  GB_TRY(e.Initialize(*b,v1,v2,1.0e-9,1.0e-8);e.Perform(*a,u1,u2);gb_ex_cc_result<GbSpace2>(e);if(e.IsDone())gb_ex_cc_trimmed<GbSpace2>(e);
   e.Perform(*a,u1,u2);gb_ex_cc_result<GbSpace2>(e));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_ecc / ex_ecc2d: <adaptor1> <adaptor2> u1 u2 v1 v2 single tol: the global search
// on the adaptor ranges and on the given ones, each performed twice.
template<class S,class E> void gb_ex_ecc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<S>(r);auto b=gb_ex_adaptor<S>(r);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();bool single=r.flag();double tol=r.real();
  GB_TRY(E e(*a,*b);e.SetSingleSolutionFlag(single);gb_flag(e.GetSingleSolutionFlag());e.SetTolerance(tol);e.Perform();gb_ex_cc_result<S>(e);e.Perform();gb_ex_cc_result<S>(e));
  GB_TRY(E e(*a,*b,u1,u2,v1,v2);e.Perform();gb_ex_cc_result<S>(e));
  E d;GB_TRY(gb_flag(d.IsParallel()));d.SetParams(*a,*b,u1,u2,v1,v2);GB_TRY(d.Perform();gb_ex_cc_result<S>(d));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_ecc_case(std::istream& in){gb_ex_ecc_case<GbSpace3,Extrema_ECC>(in);}
void gb_ex_ecc2d_case(std::istream& in){gb_ex_ecc_case<GbSpace2,Extrema_ECC2d>(in);}
// ex_locecc: <adaptor1> <adaptor2> nq (u0 v0 tolu tolv)*: Extrema_LocateExtCC and
// Extrema_LocECC.
void gb_ex_locecc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<GbSpace3>(r);auto b=gb_ex_adaptor<GbSpace3>(r);int n=r.integer();
  for(int q=0;q<n;++q){double u0=r.real(),v0=r.real(),tu=r.real(),tv=r.real();
   GB_TRY(Extrema_LocateExtCC e(*a,*b,u0,v0);gb_flag(e.IsDone());GB_TRY(gb_real(e.SquareDistance()));GB_TRY(Extrema_POnCurv p,s;e.Point(p,s);GbSpace3::pon(p);GbSpace3::pon(s)));
   GB_TRY(Extrema_LocECC e(*a,*b,u0,v0,tu,tv);gb_flag(e.IsDone());GB_TRY(gb_real(e.SquareDistance()));GB_TRY(Extrema_POnCurv p,s;e.Point(p,s);GbSpace3::pon(p);GbSpace3::pon(s)));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_fcc: <adaptor1> <adaptor2> tol nq (u v)*: Extrema_CCLocFOfLocECC: Value,
// Derivatives, Values and a saved state at each pair, the saved extrema, the tolerance
// search, CurvePtr and SetCurve.
void gb_ex_fcc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<GbSpace3>(r);auto b=gb_ex_adaptor<GbSpace3>(r);double tol=r.real();int n=r.integer();
  Extrema_CCLocFOfLocECC f(*a,*b,tol);gb_int(f.NbVariables());gb_int(f.NbEquations());gb_real(f.Tolerance());
  GB_TRY(gb_real(f.SearchOfTolerance((void*)a.get())));GB_TRY(gb_real(f.SearchOfTolerance((void*)b.get())));
  gb_flag(f.CurvePtr(1)==(void*)a.get());gb_flag(f.CurvePtr(2)==(void*)b.get());GB_TRY(f.CurvePtr(3));
  for(int q=0;q<n;++q){math_Vector uv(1,2),v(1,2),w(1,2);uv(1)=r.real();uv(2)=r.real();math_Matrix m(1,2,1,2,0.0);
   GB_TRY(v(1)=0;v(2)=0;gb_flag(f.Value(uv,v));gb_real(v(1));gb_real(v(2)));
   GB_TRY(gb_flag(f.Derivatives(uv,m));for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)gb_real(m(i,j)));
   GB_TRY(w(1)=0;w(2)=0;gb_flag(f.Values(uv,w,m));gb_real(w(1));gb_real(w(2));for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)gb_real(m(i,j)));
   gb_int(f.GetStateNumber());}
  gb_int(f.NbExt());
  for(int i=0;i<=f.NbExt()+1;++i)GB_TRY(gb_real(f.SquareDistance(i));Extrema_POnCurv p,s;f.Points(i,p,s);GbSpace3::pon(p);GbSpace3::pon(s));
  f.SetTolerance(2.0*tol);gb_real(f.Tolerance());GB_TRY(f.SetCurve(3,*a));
  f.SetCurve(1,*b);gb_flag(f.CurvePtr(1)==(void*)b.get());f.SetCurve(2,*b);
  math_Vector uv(1,2),v(1,2);uv(1)=0.25;uv(2)=0.5;GB_TRY(gb_flag(f.Value(uv,v));gb_real(v(1));gb_real(v(2)));
  Extrema_CCLocFOfLocECC g;gb_real(g.Tolerance());GB_TRY(gb_real(g.SearchOfTolerance((void*)a.get())));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_gofcc / ex_gofcc2d: <adaptor1> <adaptor2> nq (u v)*: the three Extrema_GlobOptFuncCC
// functions.
template<class S> void gb_ex_gofcc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<S>(r);auto b=gb_ex_adaptor<S>(r);int n=r.integer();
  Extrema_GlobOptFuncCCC0 f0(*a,*b);Extrema_GlobOptFuncCCC1 f1(*a,*b);Extrema_GlobOptFuncCCC2 f2(*a,*b);
  gb_int(f0.NbVariables());gb_int(f1.NbVariables());gb_int(f2.NbVariables());
  for(int q=0;q<n;++q){math_Vector x(1,2),g(1,2,0.0);x(1)=r.real();x(2)=r.real();math_Matrix h(1,2,1,2,0.0);double v=0;
   GB_TRY(gb_flag(f0.Value(x,v));gb_real(v));GB_TRY(v=0;gb_flag(f1.Value(x,v));gb_real(v));
   GB_TRY(gb_flag(f1.Gradient(x,g));gb_real(g(1));gb_real(g(2)));GB_TRY(v=0;gb_flag(f1.Values(x,v,g));gb_real(v));
   GB_TRY(v=0;gb_flag(f2.Values(x,v,g));gb_real(v);gb_real(g(1));gb_real(g(2)));
   GB_TRY(v=0;gb_flag(f2.Values(x,v,g,h));gb_real(v);for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)gb_real(h(i,j)));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_gofcc_case(std::istream& in){gb_ex_gofcc_case<GbSpace3>(in);}
void gb_ex_gofcc2d_case(std::istream& in){gb_ex_gofcc_case<GbSpace2>(in);}
