// TKGeomBase oracle: the point-curve classes of the Extrema package (Extrema_ExtPC,
// Extrema_LocateExtPC, the EPC/PCF/LocEPC instantiations of the templates and
// Extrema_GCurveLocator), in 3D and 2D. Twin: crates/tkgeombase/examples/gb_probe/extrema_pc.rs.
//
// Curves are read with gb_curve3d (tkgeombase_probe_extrema_pc.cxx) and gb_curve2d
// (tkgeombase_probe_extrema_pc2d.cxx). An adaptor is `ranged [u1 u2]` over the curve.
#include <Extrema_CurveTool.hxx>
#include <Extrema_Curve2dTool.hxx>
#include <Extrema_EPCOfExtPC.hxx>
#include <Extrema_EPCOfExtPC2d.hxx>
#include <Extrema_ExtPC.hxx>
#include <Extrema_ExtPC2d.hxx>
#include <Extrema_GCurveLocator.hxx>
#include <Extrema_LocEPCOfLocateExtPC.hxx>
#include <Extrema_LocEPCOfLocateExtPC2d.hxx>
#include <Extrema_LocateExtPC.hxx>
#include <Extrema_LocateExtPC2d.hxx>
#include <Extrema_PCFOfEPCOfExtPC.hxx>
#include <Extrema_PCFOfEPCOfExtPC2d.hxx>
#include <Geom2dAdaptor_Curve.hxx>
// The 3D and 2D types of one request family.
struct GbSpace3{
 typedef GeomAdaptor_Curve Adaptor;typedef gp_Pnt Pnt;typedef Extrema_POnCurv POn;typedef Adaptor3d_Curve Curve;typedef Extrema_CurveTool Tool;
 static occ::handle<Geom_Curve> curve(GbReader& r){return gb_curve3d(r);}
 static Pnt point(GbReader& r){return r.pnt();}
 static void pnt(const Pnt& p){gb_pnt(p);}
 static void pon(const POn& p){gb_real(p.Parameter());gb_pnt(p.Value());}
};
struct GbSpace2{
 typedef Geom2dAdaptor_Curve Adaptor;typedef gp_Pnt2d Pnt;typedef Extrema_POnCurv2d POn;typedef Adaptor2d_Curve2d Curve;typedef Extrema_Curve2dTool Tool;
 static occ::handle<Geom2d_Curve> curve(GbReader& r){return gb_curve2d(r);}
 static Pnt point(GbReader& r){return r.pnt2d();}
 static void pnt(const Pnt& p){gb_pnt2d(p);}
 static void pon(const POn& p){gb_real(p.Parameter());gb_pnt2d(p.Value());}
};
template<class S> std::unique_ptr<typename S::Adaptor> gb_ex_adaptor(GbReader& r){
 auto c=S::curve(r);bool ranged=r.flag();
 if(ranged){double u1=r.real(),u2=r.real();return std::unique_ptr<typename S::Adaptor>(new typename S::Adaptor(c,u1,u2));}
 return std::unique_ptr<typename S::Adaptor>(new typename S::Adaptor(c));
}
// Extrema_ExtPC(2d) results.
template<class S,class E> void gb_ex_extpc_result(const E& e){
 gb_flag(e.IsDone());
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));gb_flag(e.IsMin(i));S::pon(e.Point(i));});
 double d1,d2;typename S::Pnt p1,p2;e.TrimmedSquareDistances(d1,d2,p1,p2);gb_real(d1);gb_real(d2);S::pnt(p1);S::pnt(p2);
 GB_TRY(gb_real(e.SquareDistance(0)));
}
// ex_pc / ex_pc2d: <curve> ranged [u1 u2] tolf s1 s2 nq points: the constructor on the
// adaptor range, then one object initialized on [s1, s2] performing every point.
template<class S,class E> void gb_ex_extpc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<S>(r);double tolf=r.real(),s1=r.real(),s2=r.real();int n=r.integer();
  E reused;GB_TRY(reused.Initialize(*a,s1,s2,tolf));
  for(int q=0;q<n;++q){typename S::Pnt p=S::point(r);
   GB_TRY(E e(p,*a,tolf);gb_ex_extpc_result<S>(e));
   GB_TRY(reused.Perform(p);gb_ex_extpc_result<S>(reused));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_pc_case(std::istream& in){gb_ex_extpc_case<GbSpace3,Extrema_ExtPC>(in);}
void gb_ex_pc2d_case(std::istream& in){gb_ex_extpc_case<GbSpace2,Extrema_ExtPC2d>(in);}
// ex_locpc / ex_locpc2d: <curve> ranged [u1 u2] tolf s1 s2 nq (point u0)*: the
// constructor on the adaptor range, then one object on [s1, s2].
template<class S,class E> void gb_ex_loc_result(const E& e){
 gb_flag(e.IsDone());GB_TRY(gb_real(e.SquareDistance()));GB_TRY(gb_flag(e.IsMin()));GB_TRY(S::pon(e.Point()));
}
template<class S,class E> void gb_ex_locpc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<S>(r);double tolf=r.real(),s1=r.real(),s2=r.real();int n=r.integer();
  E reused;GB_TRY(reused.Initialize(*a,s1,s2,tolf));
  for(int q=0;q<n;++q){typename S::Pnt p=S::point(r);double u0=r.real();
   GB_TRY(E e(p,*a,u0,tolf);gb_ex_loc_result<S>(e));
   GB_TRY(reused.Perform(p,u0);gb_ex_loc_result<S>(reused));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_locpc_case(std::istream& in){gb_ex_locpc_case<GbSpace3,Extrema_LocateExtPC>(in);}
void gb_ex_locpc2d_case(std::istream& in){gb_ex_locpc_case<GbSpace2,Extrema_LocateExtPC2d>(in);}
// ex_genpc / ex_genpc2d: <curve> ranged [u1 u2] nb umin usup tolu tolf nq points: the
// EPC template (math_FunctionRoots) with and without a range, and the generic locate
// (LocEPC) from the middle of [umin, usup].
template<class S,class E> void gb_ex_epc_result(const E& e){
 gb_flag(e.IsDone());
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));gb_flag(e.IsMin(i));S::pon(e.Point(i));});
 GB_TRY(gb_real(e.SquareDistance(0)));
}
template<class S,class E,class L> void gb_ex_genpc_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<S>(r);int nb=r.integer();double umin=r.real(),usup=r.real(),tolu=r.real(),tolf=r.real();int n=r.integer();
  for(int q=0;q<n;++q){typename S::Pnt p=S::point(r);
   GB_TRY(E e(p,*a,nb,tolu,tolf);gb_ex_epc_result<S>(e));
   GB_TRY(E e(p,*a,nb,umin,usup,tolu,tolf);gb_ex_epc_result<S>(e));
   GB_TRY(L l(p,*a,0.5*(umin+usup),umin,usup,tolu);gb_ex_loc_result<S>(l));
   GB_TRY(L l(p,*a,umin,tolu);gb_ex_loc_result<S>(l));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_genpc_case(std::istream& in){gb_ex_genpc_case<GbSpace3,Extrema_EPCOfExtPC,Extrema_LocEPCOfLocateExtPC>(in);}
void gb_ex_genpc2d_case(std::istream& in){gb_ex_genpc_case<GbSpace2,Extrema_EPCOfExtPC2d,Extrema_LocEPCOfLocateExtPC2d>(in);}
// ex_pcf / ex_pcf2d: <curve> ranged [u1 u2] point nu u...: the GFuncExtPC function:
// SearchOfTolerance, Value, Derivative, Values and a saved state at each parameter,
// then the saved extrema; a default function raises Standard_TypeMismatch.
template<class S,class F> void gb_ex_pcf_case(std::istream& in){
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<S>(r);typename S::Pnt p=S::point(r);std::vector<double> us=r.reals();
  F f(p,*a);GB_TRY(gb_real(f.SearchOfTolerance()));
  for(double u:us){double v=0,d=0;
   GB_TRY(gb_flag(f.Value(u,v));gb_real(v));GB_TRY(gb_flag(f.Derivative(u,d));gb_real(d));
   GB_TRY(v=0;d=0;gb_flag(f.Values(u,v,d));gb_real(v);gb_real(d));GB_TRY(gb_int(f.GetStateNumber()));}
  gb_int(f.NbExt());
  for(int i=0;i<=f.NbExt()+1;++i)GB_TRY(gb_real(f.SquareDistance(i));gb_flag(f.IsMin(i));S::pon(f.Point(i)));
  F g;double v=0;GB_TRY(gb_flag(g.Value(0.5,v)));g.SetPoint(p);GB_TRY(gb_flag(g.Value(0.5,v)));
  g.Initialize(*a);g.SubIntervalInitialize(0.0,1.0);GB_TRY(gb_flag(g.Value(0.5,v));gb_real(v));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_pcf_case(std::istream& in){gb_ex_pcf_case<GbSpace3,Extrema_PCFOfEPCOfExtPC>(in);}
void gb_ex_pcf2d_case(std::istream& in){gb_ex_pcf_case<GbSpace2,Extrema_PCFOfEPCOfExtPC2d>(in);}
// ex_cloc / ex_cloc2d: <curve> ranged [u1 u2] nq (point nbu umin usup)*: both
// Extrema_GCurveLocator::Locate overloads.
template<class S> void gb_ex_cloc_case(std::istream& in){
 typedef Extrema_GCurveLocator<typename S::Curve,typename S::Tool,typename S::POn,typename S::Pnt> Locator;
 GbReader r{in};
 try{
  auto a=gb_ex_adaptor<S>(r);int n=r.integer();
  for(int q=0;q<n;++q){typename S::Pnt p=S::point(r);int nb=r.integer();double umin=r.real(),usup=r.real();
   typename S::POn x;GB_TRY(Locator::Locate(p,*a,nb,x);S::pon(x));GB_TRY(Locator::Locate(p,*a,nb,umin,usup,x);S::pon(x));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_cloc_case(std::istream& in){gb_ex_cloc_case<GbSpace3>(in);}
void gb_ex_cloc2d_case(std::istream& in){gb_ex_cloc_case<GbSpace2>(in);}
// ex_ctool / ex_ctool2d: <curve> ranged [u1 u2]: the tool members that are not plain
// forwards (IsPeriodic, the 2D IsClosed) and Resolution.
void gb_ex_ctool_case(std::istream& in){
 GbReader r{in};
 try{auto a=gb_ex_adaptor<GbSpace3>(r);gb_flag(Extrema_CurveTool::IsPeriodic(*a));GB_TRY(gb_real(Extrema_CurveTool::Period(*a)));
  gb_real(Extrema_CurveTool::Resolution(*a,1.0e-7));gb_int(Extrema_CurveTool::GetType(*a));gb_int(Extrema_CurveTool::Continuity(*a));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
void gb_ex_ctool2d_case(std::istream& in){
 GbReader r{in};
 try{auto a=gb_ex_adaptor<GbSpace2>(r);gb_flag(Extrema_Curve2dTool::IsPeriodic(*a));gb_flag(Extrema_Curve2dTool::IsClosed(*a));
  GB_TRY(gb_real(Extrema_Curve2dTool::Period(*a)));gb_real(Extrema_Curve2dTool::Resolution(*a,1.0e-7));
  gb_int(Extrema_Curve2dTool::GetType(*a));gb_int(Extrema_Curve2dTool::Continuity(*a));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
