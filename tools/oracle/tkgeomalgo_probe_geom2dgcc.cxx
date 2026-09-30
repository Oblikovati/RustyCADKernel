// Geom2dGcc probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/geom2d_gcc.rs (the qualified curve, curve tool and
// functions), geom2d_gcc_lines.rs (the line solvers) and geom2d_gcc_circles.rs (the circle
// solvers). A qualified curve is `position curve u0 u1` (the curve as ga_hatch_adaptor reads
// it), a point `x y`, a GccEnt argument as ga_read_qcirc / ga_read_qlin read it. Every solver
// request is `kind detail arguments...`: `kind` selects the constructor overload, `detail` the
// consultations printed per solution (bit 1 qualifiers, bit 2 first tangency, bit 4 second,
// bit 8 third tangency or centre); the corpus clears the bits of the slots native leaves
// uninitialized (tools/tkgeomalgo_corpus_geom2dgcc.py). The computations run under GaMute:
// the Geom2dInt polygon intersector may write on std::cout, which the port does not.
#include <Geom2dGcc.hxx>
#include <Geom2dGcc_Circ2d2TanOn.hxx>
#include <Geom2dGcc_Circ2d2TanOnGeo.hxx>
#include <Geom2dGcc_Circ2d2TanOnIter.hxx>
#include <Geom2dGcc_Circ2d2TanRad.hxx>
#include <Geom2dGcc_Circ2d2TanRadGeo.hxx>
#include <Geom2dGcc_Circ2d3Tan.hxx>
#include <Geom2dGcc_Circ2d3TanIter.hxx>
#include <Geom2dGcc_Circ2dTanCen.hxx>
#include <Geom2dGcc_Circ2dTanCenGeo.hxx>
#include <Geom2dGcc_CurveTool.hxx>
#include <Geom2dGcc_FunctionTanCirCu.hxx>
#include <Geom2dGcc_FunctionTanCuCu.hxx>
#include <Geom2dGcc_FunctionTanCuCuCu.hxx>
#include <Geom2dGcc_FunctionTanCuCuOnCu.hxx>
#include <Geom2dGcc_FunctionTanCuPnt.hxx>
#include <Geom2dGcc_FunctionTanObl.hxx>
#include <Geom2dGcc_Lin2d2Tan.hxx>
#include <Geom2dGcc_Lin2d2TanIter.hxx>
#include <Geom2dGcc_Lin2dTanObl.hxx>
#include <Geom2dGcc_Lin2dTanOblIter.hxx>
#include <Geom2dGcc_QCurve.hxx>
#include <Geom2dGcc_QualifiedCurve.hxx>
#include <Geom2d_CartesianPoint.hxx>
Geom2dGcc_QualifiedCurve ga_g2gcc_q(GaReader& r){GccEnt_Position p=GccEnt_Position(r.integer());return Geom2dGcc_QualifiedCurve(ga_hatch_adaptor(r),p);}
Geom2dGcc_QCurve ga_g2gcc_qc(GaReader& r){GccEnt_Position p=GccEnt_Position(r.integer());return Geom2dGcc_QCurve(ga_hatch_adaptor(r),p);}
occ::handle<Geom2d_Point> ga_g2gcc_point(GaReader& r){double x=r.real();return new Geom2d_CartesianPoint(x,r.real());}
template<class Q> void ga_g2gcc_qflags(const Q& q){
 ga_int(q.Qualifier());ga_flag(q.IsUnqualified());ga_flag(q.IsEnclosing());ga_flag(q.IsEnclosed());ga_flag(q.IsOutside());
 Geom2dAdaptor_Curve a=q.Qualified();ga_adaptor_out(a);ga_int(a.GetType());
}
// `position curve u0 u1`: the four Geom2dGcc makers, then Geom2dGcc_QualifiedCurve and
// Geom2dGcc_QCurve of the position.
void ga_g2gcc_qualified_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int p=r.integer();Geom2dAdaptor_Curve a=ga_hatch_adaptor(r);
  ga_int(Geom2dGcc::Unqualified(a).Qualifier());ga_int(Geom2dGcc::Enclosing(a).Qualifier());
  ga_int(Geom2dGcc::Enclosed(a).Qualifier());ga_int(Geom2dGcc::Outside(a).Qualifier());
  ga_g2gcc_qflags(Geom2dGcc_QualifiedCurve(a,GccEnt_Position(p)));ga_g2gcc_qflags(Geom2dGcc_QCurve(a,GccEnt_Position(p))));
 ga_end();
}
// `curve u0 u1 tol n u...`: every member of Geom2dGcc_CurveTool.
void ga_g2gcc_tool_case(std::istream& input){
 GaReader r{input};
 GA_TRY(Geom2dAdaptor_Curve a=ga_hatch_adaptor(r);double tol=r.real();std::vector<double> us=r.reals();
  GA_TRY(ga_real(Geom2dGcc_CurveTool::FirstParameter(a)));GA_TRY(ga_real(Geom2dGcc_CurveTool::LastParameter(a)));
  GA_TRY(ga_real(Geom2dGcc_CurveTool::EpsX(a,tol)));ga_int(Geom2dGcc_CurveTool::NbSamples(a));
  for(double u:us){gp_Pnt2d p;gp_Vec2d t,n,dn;
   GA_TRY(ga_pnt2d(Geom2dGcc_CurveTool::Value(a,u)));GA_TRY(Geom2dGcc_CurveTool::D1(a,u,p,t);ga_pnt2d(p);ga_v2d(t));
   GA_TRY(Geom2dGcc_CurveTool::D2(a,u,p,t,n);ga_pnt2d(p);ga_v2d(t);ga_v2d(n));
   GA_TRY(Geom2dGcc_CurveTool::D3(a,u,p,t,n,dn);ga_pnt2d(p);ga_v2d(t);ga_v2d(n);ga_v2d(dn));});
 ga_end();
}
// Value, Derivative and Values of a scalar function at x, each on a fresh function.
template<class F,class M> void ga_g2gcc_scalar(M make,double x){
 GA_TRY(F f=make();double v=0.;bool ok=f.Value(x,v);ga_flag(ok);ga_real(v));
 GA_TRY(F f=make();double d=0.;bool ok=f.Derivative(x,d);ga_flag(ok);ga_real(d));
 GA_TRY(F f=make();double v=0.,d=0.;bool ok=f.Values(x,v,d);ga_flag(ok);ga_real(v);ga_real(d));
}
// `kind curve u0 u1 args n x...`: kind 0 FunctionTanObl (`dx dy`), 1 FunctionTanCuPnt
// (`x y`), 2 FunctionTanCirCu (`x y dx dy r`, the circle first).
void ga_g2gcc_func1_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int kind=r.integer();Geom2dAdaptor_Curve a=ga_hatch_adaptor(r);
  if(kind==0){gp_Dir2d d=r.dir2d();for(double x:r.reals())ga_g2gcc_scalar<Geom2dGcc_FunctionTanObl>([&]{return Geom2dGcc_FunctionTanObl(a,d);},x);}
  else if(kind==1){gp_Pnt2d p=r.pnt2d();for(double x:r.reals())ga_g2gcc_scalar<Geom2dGcc_FunctionTanCuPnt>([&]{return Geom2dGcc_FunctionTanCuPnt(a,p);},x);}
  else{gp_Circ2d c=ga_read_circ2d(r);for(double x:r.reals())ga_g2gcc_scalar<Geom2dGcc_FunctionTanCirCu>([&]{return Geom2dGcc_FunctionTanCirCu(c,a);},x);});
 ga_end();
}
void ga_g2gcc_matrix(const math_Matrix& m){for(int i=m.LowerRow();i<=m.UpperRow();i++)for(int j=m.LowerCol();j<=m.UpperCol();j++)ga_real(m(i,j));}
void ga_g2gcc_vector(const math_Vector& v){for(int i=v.Lower();i<=v.Upper();i++)ga_real(v(i));}
// NbVariables, NbEquations, then at each point Value, Derivatives and Values into
// outputs filled with 7 (to show what a member leaves untouched), each on a fresh function.
template<class F,class M,class I> void ga_g2gcc_set(M make,I init,GaReader& r){
 {F f=make();ga_int(f.NbVariables());ga_int(f.NbEquations());}
 int n=r.integer(),nv=r.integer();
 for(int k=0;k<n;k++){math_Vector x(1,nv);for(int i=1;i<=nv;i++)x(i)=r.real();
  GA_TRY(F f=make();init(f,x));
  GA_TRY(F f=make();math_Vector v(1,nv,7.);ga_flag(f.Value(x,v));ga_g2gcc_vector(v));
  GA_TRY(F f=make();math_Matrix m(1,nv,1,nv,7.);ga_flag(f.Derivatives(x,m));ga_g2gcc_matrix(m));
  GA_TRY(F f=make();math_Vector v(1,nv,7.);math_Matrix m(1,nv,1,nv,7.);ga_flag(f.Values(x,v,m));ga_g2gcc_vector(v);ga_g2gcc_matrix(m));}
}
// `kind curve1 u0 u1 | circle, curve2 u0 u1, n nv x...`: FunctionTanCuCu (kind 0 two
// curves, 1 a circle and a curve).
void ga_g2gcc_func2_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int kind=r.integer();Geom2dAdaptor_Curve a1;gp_Circ2d c1;if(kind==0)a1=ga_hatch_adaptor(r);else c1=ga_read_circ2d(r);
  Geom2dAdaptor_Curve a2=ga_hatch_adaptor(r);
  auto make=[&]{return kind==0?Geom2dGcc_FunctionTanCuCu(a1,a2):Geom2dGcc_FunctionTanCuCu(c1,a2);};
  auto init=[](Geom2dGcc_FunctionTanCuCu& f,const math_Vector& x){gp_Pnt2d p1,p2;gp_Vec2d t1,t2,d1,d2;f.InitDerivative(x,p1,p2,t1,t2,d1,d2);
   ga_pnt2d(p1);ga_pnt2d(p2);ga_v2d(t1);ga_v2d(t2);ga_v2d(d1);ga_v2d(d2);};
  ga_g2gcc_set<Geom2dGcc_FunctionTanCuCu>(make,init,r));
 ga_end();
}
// The nine InitDerivative outputs of the three-argument functions.
template<class F> void ga_g2gcc_init3(F& f,const math_Vector& x){
 gp_Pnt2d p1,p2,p3;gp_Vec2d t1,t2,t3,d1,d2,d3;f.InitDerivative(x,p1,p2,p3,t1,t2,t3,d1,d2,d3);
 ga_pnt2d(p1);ga_pnt2d(p2);ga_pnt2d(p3);ga_v2d(t1);ga_v2d(t2);ga_v2d(t3);ga_v2d(d1);ga_v2d(d2);ga_v2d(d3);
}
// `kind args... n nv x...`: FunctionTanCuCuCu; kind 0 (curve, curve, curve), 1 (circle,
// curve, curve), 2 (circle, circle, curve), 3 (circle, line, curve), 4 (line, line, curve),
// 5 (line, curve, curve).
void ga_g2gcc_func3_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int kind=r.integer();Geom2dAdaptor_Curve a1,a2,a3;gp_Circ2d c1,c2;gp_Lin2d l1,l2;
  if(kind==0)a1=ga_hatch_adaptor(r);else if(kind<=3)c1=ga_read_circ2d(r);else l1=ga_read_lin2d(r);
  if(kind==0||kind==1||kind==5)a2=ga_hatch_adaptor(r);else if(kind==2)c2=ga_read_circ2d(r);else l2=ga_read_lin2d(r);
  a3=ga_hatch_adaptor(r);
  auto make=[&]{switch(kind){case 0:return Geom2dGcc_FunctionTanCuCuCu(a1,a2,a3);case 1:return Geom2dGcc_FunctionTanCuCuCu(c1,a2,a3);
   case 2:return Geom2dGcc_FunctionTanCuCuCu(c1,c2,a3);case 3:return Geom2dGcc_FunctionTanCuCuCu(c1,l2,a3);
   case 4:return Geom2dGcc_FunctionTanCuCuCu(l1,l2,a3);default:return Geom2dGcc_FunctionTanCuCuCu(l1,a2,a3);}};
  ga_g2gcc_set<Geom2dGcc_FunctionTanCuCuCu>(make,[](Geom2dGcc_FunctionTanCuCuCu& f,const math_Vector& x){ga_g2gcc_init3(f,x);},r));
 ga_end();
}
// `kind first second locus rad n nv x...`: FunctionTanCuCuOnCu; the first argument is a
// curve, a circle or a line by kind % 4 (0 curve, 1 circle, 2 line, 3 curve with a point
// second), the locus a circle (kind < 4), a line (kind < 8) or a curve.
void ga_g2gcc_func4_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int kind=r.integer();int first=kind%4,locus=kind/4;Geom2dAdaptor_Curve a1,a2,on;gp_Circ2d c1,oc;gp_Lin2d l1,ol;gp_Pnt2d p2;
  if(first==0||first==3)a1=ga_hatch_adaptor(r);else if(first==1)c1=ga_read_circ2d(r);else l1=ga_read_lin2d(r);
  if(first==3)p2=r.pnt2d();else a2=ga_hatch_adaptor(r);
  if(locus==0)oc=ga_read_circ2d(r);else if(locus==1)ol=ga_read_lin2d(r);else on=ga_hatch_adaptor(r);
  double rad=r.real();
  auto make=[&]{switch(kind){
   case 0:return Geom2dGcc_FunctionTanCuCuOnCu(a1,a2,oc,rad);case 1:return Geom2dGcc_FunctionTanCuCuOnCu(c1,a2,oc,rad);
   case 2:return Geom2dGcc_FunctionTanCuCuOnCu(l1,a2,oc,rad);case 3:return Geom2dGcc_FunctionTanCuCuOnCu(a1,p2,oc,rad);
   case 4:return Geom2dGcc_FunctionTanCuCuOnCu(a1,a2,ol,rad);case 5:return Geom2dGcc_FunctionTanCuCuOnCu(c1,a2,ol,rad);
   case 6:return Geom2dGcc_FunctionTanCuCuOnCu(l1,a2,ol,rad);case 7:return Geom2dGcc_FunctionTanCuCuOnCu(a1,p2,ol,rad);
   case 8:return Geom2dGcc_FunctionTanCuCuOnCu(a1,a2,on,rad);case 9:return Geom2dGcc_FunctionTanCuCuOnCu(c1,a2,on,rad);
   case 10:return Geom2dGcc_FunctionTanCuCuOnCu(l1,a2,on,rad);default:return Geom2dGcc_FunctionTanCuCuOnCu(a1,p2,on,rad);}};
  ga_g2gcc_set<Geom2dGcc_FunctionTanCuCuOnCu>(make,[](Geom2dGcc_FunctionTanCuCuOnCu& f,const math_Vector& x){ga_g2gcc_init3(f,x);},r));
 ga_end();
}
#define GA_G2_TANGENCY(CALL) GA_TRY(double s_=0.,a_=0.;gp_Pnt2d p_;CALL(s_,a_,p_);ga_tangency(s_,a_,p_))
#define GA_G2_CENTRE(CALL) GA_TRY(double a_=0.;gp_Pnt2d p_;CALL(a_,p_);ga_real(a_);ga_pnt2d(p_))
#define GA_G2_CENTRE_I(CALL) GA_TRY(double a_=0.;gp_Pnt2d p_;CALL(i,a_,p_);ga_real(a_);ga_pnt2d(p_))
// Builds a solver with std::cout muted (the report prints unmuted).
// Zeroes the stack the solver is about to use. Geom2dGcc_FunctionTanCuCuCu::Values leaves
// the residual untouched, so math_FunctionSetRoot inside Geom2dGcc_Circ2d3TanIter reads its
// uninitialized residual buffer (a math_Vector on the stack): the search result depends on
// what earlier requests left there. Zeroed, native finds what the port does, which starts
// the buffer at 0.
__attribute__((noinline)) void ga_g2gcc_scrub(){volatile char b[1<<17];for(size_t i=0;i<sizeof b;i++)b[i]=0;}
template<class F> auto ga_g2gcc_build(F f){GaMute m;ga_g2gcc_scrub();return std::unique_ptr<std::remove_pointer_t<decltype(f())>>(f());}
void ga_g2gcc_lin2tan_report(const Geom2dGcc_Lin2d2Tan& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_lin2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a,b;s.WhichQualifier(i,a,b);ga_int(a);ga_int(b));
  if(detail&2)GA_TANGENCY(s.Tangency1);
  if(detail&4)GA_TANGENCY(s.Tangency2);)
}
// `kind detail args`: Geom2dGcc_Lin2d2Tan 0 (q1, q2, tol), 1 (q1, point, tol), 2 (q1, q2, tol,
// p1, p2), 3 (q1, point, tol, p1).
void ga_g2gcc_lin2tan_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r);
  if(kind==0||kind==2){Geom2dGcc_QualifiedCurve q2=ga_g2gcc_q(r);double tol=r.real();
   if(kind==0)ga_g2gcc_lin2tan_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Lin2d2Tan(q1,q2,tol);}),detail);
   else{double p1=r.real(),p2=r.real();ga_g2gcc_lin2tan_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Lin2d2Tan(q1,q2,tol,p1,p2);}),detail);}}
  else{gp_Pnt2d p=r.pnt2d();double tol=r.real();
   if(kind==1)ga_g2gcc_lin2tan_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Lin2d2Tan(q1,p,tol);}),detail);
   else{double p1=r.real();ga_g2gcc_lin2tan_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Lin2d2Tan(q1,p,tol,p1);}),detail);}});
 ga_end();
}
// `kind args`: Geom2dGcc_Lin2d2TanIter 0 (q1, point, p1, tol), 1 (qcirc, q2, p2, tol),
// 2 (q1, q2, p1, p2, tol); then every consultation.
void ga_g2gcc_lin2tan_iter_case(std::istream& input){
 GaReader r{input};int kind=r.integer();
 GA_TRY(std::unique_ptr<Geom2dGcc_Lin2d2TanIter> s;
  if(kind==0){Geom2dGcc_QCurve q1=ga_g2gcc_qc(r);gp_Pnt2d p=r.pnt2d();double p1=r.real(),tol=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Lin2d2TanIter(q1,p,p1,tol);});}
  else if(kind==1){GccEnt_QualifiedCirc q1=ga_read_qcirc(r);Geom2dGcc_QCurve q2=ga_g2gcc_qc(r);double p2=r.real(),tol=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Lin2d2TanIter(q1,q2,p2,tol);});}
  else{Geom2dGcc_QCurve q1=ga_g2gcc_qc(r),q2=ga_g2gcc_qc(r);double p1=r.real(),p2=r.real(),tol=r.real();s=ga_g2gcc_build([&]{return new Geom2dGcc_Lin2d2TanIter(q1,q2,p1,p2,tol);});}
  ga_flag(s->IsDone());GA_TRY(ga_lin2d(s->ThisSolution()));GA_TRY(GccEnt_Position a,b;s->WhichQualifier(a,b);ga_int(a);ga_int(b));
  GA_G2_TANGENCY(s->Tangency1);GA_G2_TANGENCY(s->Tangency2));
 ga_end();
}
void ga_g2gcc_lintanobl_report(const Geom2dGcc_Lin2dTanObl& s,int detail){
 GA_SOLUTIONS(s,
  GA_TRY(ga_lin2d(s.ThisSolution(i)));
  if(detail&1)GA_TRY(GccEnt_Position a;s.WhichQualifier(i,a);ga_int(a));
  if(detail&2)GA_TANGENCY(s.Tangency1);
  if(detail&4)GA_TANGENCY(s.Intersection2);)
}
// `kind detail q1 line tolang [param1] angle`: Geom2dGcc_Lin2dTanObl 0 (q1, line, tolang,
// angle), 1 (q1, line, tolang, param1, angle).
void ga_g2gcc_lintanobl_case(std::istream& input){
 GaReader r{input};int kind=r.integer(),detail=r.integer();
 GA_TRY(Geom2dGcc_QualifiedCurve q1=ga_g2gcc_q(r);gp_Lin2d l=ga_read_lin2d(r);double tol=r.real();
  if(kind==0){double a=r.real();ga_g2gcc_lintanobl_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Lin2dTanObl(q1,l,tol,a);}),detail);}
  else{double p1=r.real(),a=r.real();ga_g2gcc_lintanobl_report(*ga_g2gcc_build([&]{return new Geom2dGcc_Lin2dTanObl(q1,l,tol,p1,a);}),detail);});
 ga_end();
}
// `q1 line param1 tolang angle`: Geom2dGcc_Lin2dTanOblIter and every consultation.
void ga_g2gcc_lintanobl_iter_case(std::istream& input){
 GaReader r{input};
 GA_TRY(Geom2dGcc_QCurve q1=ga_g2gcc_qc(r);gp_Lin2d l=ga_read_lin2d(r);double p1=r.real(),tol=r.real(),a=r.real();
  auto s=ga_g2gcc_build([&]{return new Geom2dGcc_Lin2dTanOblIter(q1,l,p1,tol,a);});
  ga_flag(s->IsDone());ga_flag(s->IsParallel2());GA_TRY(ga_lin2d(s->ThisSolution()));GA_TRY(GccEnt_Position a;s->WhichQualifier(a);ga_int(a));
  GA_G2_TANGENCY(s->Tangency1);GA_G2_TANGENCY(s->Intersection2));
 ga_end();
}
