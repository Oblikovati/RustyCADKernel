// TKGeomBase oracle: the CPnts and GCPnts packages (tkgeombase_probe.cxx). A request reads
// the curve dimension (2 or 3), an adaptor range flag with its bounds, and the curve with
// the LProp builders (tkgeombase_probe_lprop_build.cxx); it then calls every overload of
// one class in a fixed order, each member printing its value or the raised exception
// type. Objects are reused across runs to expose the state kept between calls. The Rust
// twin is crates/tkgeombase/examples/gb_probe/gcpnts.rs.
#include <CPnts_AbscissaPoint.hxx>
#include <CPnts_UniformDeflection.hxx>
#include <GCPnts_AbscissaPoint.hxx>
#include <GCPnts_DistFunction.hxx>
#include <GCPnts_DistFunction2d.hxx>
#include <GCPnts_QuasiUniformAbscissa.hxx>
#include <GCPnts_QuasiUniformDeflection.hxx>
#include <GCPnts_TangentialDeflection.hxx>
#include <GCPnts_UniformAbscissa.hxx>
#include <GCPnts_UniformDeflection.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <math_Vector.hxx>
// Build the adaptor of a request and call run(adaptor, reader); a construction failure
// prints its exception type instead.
template<class Run> void gb_gcp_curve(std::istream& in,Run run){
 GbReader r{in};const int dim=r.integer();const bool ranged=r.flag();double u1=0.0,u2=0.0;
 if(ranged){u1=r.real();u2=r.real();}
 try{
  if(dim==2){occ::handle<Geom2d_Curve> c=gb_lp_curve2d(r);Geom2dAdaptor_Curve a;if(ranged)a.Load(c,u1,u2);else a.Load(c);run(a,r);}
  else{occ::handle<Geom_Curve> c=gb_lp_curve3d(r);GeomAdaptor_Curve a;if(ranged)a.Load(c,u1,u2);else a.Load(c);run(a,r);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// gcp_length U1 U2 Tol: the four Length overloads of CPnts_AbscissaPoint, then of
// GCPnts_AbscissaPoint.
void gb_gcp_length_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  const double u1=r.real(),u2=r.real(),tol=r.real();
  GB_TRY(gb_real(CPnts_AbscissaPoint::Length(a)));GB_TRY(gb_real(CPnts_AbscissaPoint::Length(a,tol)));
  GB_TRY(gb_real(CPnts_AbscissaPoint::Length(a,u1,u2)));GB_TRY(gb_real(CPnts_AbscissaPoint::Length(a,u1,u2,tol)));
  GB_TRY(gb_real(GCPnts_AbscissaPoint::Length(a)));GB_TRY(gb_real(GCPnts_AbscissaPoint::Length(a,tol)));
  GB_TRY(gb_real(GCPnts_AbscissaPoint::Length(a,u1,u2)));GB_TRY(gb_real(GCPnts_AbscissaPoint::Length(a,u1,u2,tol)));
 });
}
void gb_gcp_cpnts_point(const CPnts_AbscissaPoint& p){gb_flag(p.IsDone());GB_TRY(gb_real(p.Parameter()));}
// cpnts_abscissa Abscissa U0 Ui Resolution U1 U2 Tol: both constructors, then one object
// through every Init overload with Perform, Perform from Ui and AdvPerform, then SetParameter.
void gb_cpnts_abscissa_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  const double ab=r.real(),u0=r.real(),ui=r.real(),res=r.real(),u1=r.real(),u2=r.real(),tol=r.real();
  GB_TRY(CPnts_AbscissaPoint p(a,ab,u0,res);gb_gcp_cpnts_point(p));
  GB_TRY(CPnts_AbscissaPoint p(a,ab,u0,ui,res);gb_gcp_cpnts_point(p));
  CPnts_AbscissaPoint p;gb_gcp_cpnts_point(p);
  GB_TRY(p.Init(a);p.Perform(ab,u0,res);gb_gcp_cpnts_point(p));
  GB_TRY(p.Init(a,tol);p.AdvPerform(ab,u0,ui,res);gb_gcp_cpnts_point(p));
  GB_TRY(p.Init(a,u1,u2);p.Perform(ab,u0,ui,res);gb_gcp_cpnts_point(p));
  GB_TRY(p.Init(a,u1,u2,tol);p.AdvPerform(ab,u0,ui,res);gb_gcp_cpnts_point(p));
  p.SetParameter(u1);gb_gcp_cpnts_point(p);
 });
}
// The iteration of a CPnts_UniformDeflection, at most `cap` points, then their count.
void gb_gcp_cpnts_iterate(CPnts_UniformDeflection& it,int cap){
 gb_flag(it.IsAllDone());int n=0;
 GB_TRY(for(;it.More()&&n<cap;it.Next()){gb_real(it.Value());gb_pnt(it.Point());++n;});gb_int(n);
}
// cpnts_udefl Deflection Resolution WithControl U1 U2 Cap: both constructors, then the
// empty object (IsAllDone, More, Value, Next) and both Initialize overloads on it.
void gb_cpnts_udefl_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  const double d=r.real(),res=r.real();const bool ctl=r.flag();const double u1=r.real(),u2=r.real();const int cap=r.integer();
  GB_TRY(CPnts_UniformDeflection it(a,d,res,ctl);gb_gcp_cpnts_iterate(it,cap));
  GB_TRY(CPnts_UniformDeflection it(a,d,u1,u2,res,ctl);gb_gcp_cpnts_iterate(it,cap));
  CPnts_UniformDeflection it;gb_flag(it.IsAllDone());GB_TRY(gb_flag(it.More()));GB_TRY(gb_real(it.Value()));GB_TRY(it.Next());
  GB_TRY(it.Initialize(a,d,u1,u2,res,ctl);gb_gcp_cpnts_iterate(it,cap));
  GB_TRY(it.Initialize(a,d,res,ctl);gb_gcp_cpnts_iterate(it,cap));
 });
}
void gb_gcp_point(const GCPnts_AbscissaPoint& p){gb_flag(p.IsDone());GB_TRY(gb_real(p.Parameter()));}
// gcp_abscissa Abscissa U0 Ui Tol: the empty object and the four constructors.
void gb_gcp_abscissa_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  const double ab=r.real(),u0=r.real(),ui=r.real(),tol=r.real();
  GCPnts_AbscissaPoint e;gb_gcp_point(e);
  GB_TRY(GCPnts_AbscissaPoint p(a,ab,u0);gb_gcp_point(p));
  GB_TRY(GCPnts_AbscissaPoint p(tol,a,ab,u0);gb_gcp_point(p));
  GB_TRY(GCPnts_AbscissaPoint p(a,ab,u0,ui);gb_gcp_point(p));
  GB_TRY(GCPnts_AbscissaPoint p(a,ab,u0,ui,tol);gb_gcp_point(p));
 });
}
// The result of a GCPnts_UniformAbscissa: IsDone, then NbPoints, Abscissa and the
// parameters 1 .. NbPoints + extra (a reused array keeps older values past NbPoints).
void gb_gcp_uabs_dump(const GCPnts_UniformAbscissa& u,int extra){
 gb_flag(u.IsDone());
 GB_TRY(const int n=u.NbPoints();gb_int(n);GB_TRY(gb_real(u.Abscissa()));for(int i=0;i<=n+extra;++i){GB_TRY(gb_real(u.Parameter(i)));});
}
// gcp_uabs Runs, each Kind Value U1 U2 Toler Extra: kinds 0-3 Initialize the shared
// object (abscissa, abscissa on [U1, U2], NbPoints, NbPoints on [U1, U2]); 4-7 the
// matching constructors.
void gb_gcp_uabs_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  GCPnts_UniformAbscissa u;gb_gcp_uabs_dump(u,0);const int runs=r.integer();
  for(int k=0;k<runs;++k){
   const int kind=r.integer();const double v=r.real(),u1=r.real(),u2=r.real(),tol=r.real();const int extra=r.integer();const int nb=int(v);
   if(kind<4){GB_TRY(if(kind==0)u.Initialize(a,v,tol);else if(kind==1)u.Initialize(a,v,u1,u2,tol);else if(kind==2)u.Initialize(a,nb,tol);else u.Initialize(a,nb,u1,u2,tol));gb_gcp_uabs_dump(u,extra);continue;}
   try{
    std::unique_ptr<GCPnts_UniformAbscissa> c;
    if(kind==4)c.reset(new GCPnts_UniformAbscissa(a,v,tol));else if(kind==5)c.reset(new GCPnts_UniformAbscissa(a,v,u1,u2,tol));
    else if(kind==6)c.reset(new GCPnts_UniformAbscissa(a,nb,tol));else c.reset(new GCPnts_UniformAbscissa(a,nb,u1,u2,tol));
    gb_gcp_uabs_dump(*c,0);
   }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  }
 });
}
// gcp_quabs Runs, each Kind NbPoints U1 U2: kinds 0-1 Initialize the shared object
// (whole curve, [U1, U2]), 2-3 the constructors; IsDone, NbPoints and the parameters.
void gb_gcp_quabs_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  auto dump=[](const GCPnts_QuasiUniformAbscissa& q){gb_flag(q.IsDone());GB_TRY(const int n=q.NbPoints();gb_int(n);for(int i=0;i<=n;++i){GB_TRY(gb_real(q.Parameter(i)));});};
  GCPnts_QuasiUniformAbscissa q;dump(q);const int runs=r.integer();
  for(int k=0;k<runs;++k){
   const int kind=r.integer(),nb=r.integer();const double u1=r.real(),u2=r.real();
   if(kind<2){GB_TRY(if(kind==0)q.Initialize(a,nb);else q.Initialize(a,nb,u1,u2));dump(q);continue;}
   try{std::unique_ptr<GCPnts_QuasiUniformAbscissa> c(kind==2?new GCPnts_QuasiUniformAbscissa(a,nb):new GCPnts_QuasiUniformAbscissa(a,nb,u1,u2));dump(*c);}
   catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  }
 });
}
// The points of a deflection distribution: IsDone, then NbPoints, Deflection and each
// Parameter and Value (from index 0, which is out of range).
template<class Algo> void gb_gcp_defl_dump(const Algo& d){
 gb_flag(d.IsDone());GB_TRY(gb_real(d.Deflection()));GB_TRY(gb_pnt(d.Value(1)));
 GB_TRY(const int n=d.NbPoints();gb_int(n);for(int i=0;i<=n;++i){GB_TRY(gb_real(d.Parameter(i)));GB_TRY(gb_pnt(d.Value(i)));});
}
// gcp_udefl Runs, each Kind Deflection WithControl U1 U2: kinds 0-1 Initialize the
// shared object (whole curve, [U1, U2]), 2-3 the constructors.
void gb_gcp_udefl_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  GCPnts_UniformDeflection d;gb_gcp_defl_dump(d);const int runs=r.integer();
  for(int k=0;k<runs;++k){
   const int kind=r.integer();const double f=r.real();const bool ctl=r.flag();const double u1=r.real(),u2=r.real();
   if(kind<2){GB_TRY(if(kind==0)d.Initialize(a,f,ctl);else d.Initialize(a,f,u1,u2,ctl));gb_gcp_defl_dump(d);continue;}
   try{std::unique_ptr<GCPnts_UniformDeflection> c(kind==2?new GCPnts_UniformDeflection(a,f,ctl):new GCPnts_UniformDeflection(a,f,u1,u2,ctl));gb_gcp_defl_dump(*c);}
   catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  }
 });
}
// gcp_qudefl Runs, each Kind Deflection Continuity U1 U2: as gcp_udefl, with the
// GeomAbs_Shape continuity instead of the control flag.
void gb_gcp_qudefl_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  GCPnts_QuasiUniformDeflection d;gb_gcp_defl_dump(d);const int runs=r.integer();
  for(int k=0;k<runs;++k){
   const int kind=r.integer();const double f=r.real();const GeomAbs_Shape s=GeomAbs_Shape(r.integer());const double u1=r.real(),u2=r.real();
   if(kind<2){GB_TRY(if(kind==0)d.Initialize(a,f,s);else d.Initialize(a,f,u1,u2,s));gb_gcp_defl_dump(d);continue;}
   try{std::unique_ptr<GCPnts_QuasiUniformDeflection> c(kind==2?new GCPnts_QuasiUniformDeflection(a,f,s):new GCPnts_QuasiUniformDeflection(a,f,u1,u2,s));gb_gcp_defl_dump(*c);}
   catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  }
 });
}
void gb_gcp_tdefl_dump(const GCPnts_TangentialDeflection& t){
 const int n=t.NbPoints();gb_int(n);for(int i=0;i<=n+1;++i){GB_TRY(gb_real(t.Parameter(i)));GB_TRY(gb_pnt(t.Value(i)));}
}
// gcp_tdefl Runs, each Kind Angular Curvature MinPoints UTol MinLen U1 U2, then Adds,
// each X Y Z Param Replace: kinds 0-1 Initialize the shared object (whole curve,
// [U1, U2]), 2-3 the constructors; AddPoint then acts on the shared object.
void gb_gcp_tdefl_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){
  GCPnts_TangentialDeflection t;gb_gcp_tdefl_dump(t);const int runs=r.integer();
  for(int k=0;k<runs;++k){
   const int kind=r.integer();const double ang=r.real(),cur=r.real();const int mp=r.integer();const double ut=r.real(),ml=r.real(),u1=r.real(),u2=r.real();
   if(kind<2){GB_TRY(if(kind==0)t.Initialize(a,ang,cur,mp,ut,ml);else t.Initialize(a,u1,u2,ang,cur,mp,ut,ml));gb_gcp_tdefl_dump(t);continue;}
   try{std::unique_ptr<GCPnts_TangentialDeflection> c(kind==2?new GCPnts_TangentialDeflection(a,ang,cur,mp,ut,ml):new GCPnts_TangentialDeflection(a,u1,u2,ang,cur,mp,ut,ml));gb_gcp_tdefl_dump(*c);}
   catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  }
  const int adds=r.integer();
  for(int k=0;k<adds;++k){const gp_Pnt p=r.pnt();const double u=r.real();const bool rep=r.flag();gb_int(t.AddPoint(p,u,rep));}
  gb_gcp_tdefl_dump(t);
 });
}
// gcp_arc_step Radius LinearDeflection AngularDeflection MinLength.
void gb_gcp_arc_step_case(std::istream& in){
 GbReader r{in};const double rad=r.real(),lin=r.real(),ang=r.real(),ml=r.real();
 GB_TRY(gb_real(GCPnts_TangentialDeflection::ArcAngularStep(rad,lin,ang,ml)));gb_end();
}
// The DistFunction class of the curve dimension: Value at each X (flag, then the value
// when true), NbVariables and Value of the MV view at each X. The copy constructors are
// declared by the headers but not defined in TKGeomBase, so they cannot be called.
template<class F,class MV,class A> void gb_gcp_dist_run(const A& a,GbReader& r){
 const double u1=r.real(),u2=r.real();const int n=r.integer();std::vector<double> xs(n);for(int i=0;i<n;++i)xs[i]=r.real();
 F f(a,u1,u2);double v=0.0;
 for(double x:xs){const bool ok=f.Value(x,v);gb_flag(ok);if(ok)gb_real(v);}
 MV mv(f);gb_int(mv.NbVariables());math_Vector X(1,1);
 for(double x:xs){X(1)=x;const bool ok=mv.Value(X,v);gb_flag(ok);if(ok)gb_real(v);}
}
void gb_gcp_dist(const Adaptor3d_Curve& a,GbReader& r){gb_gcp_dist_run<GCPnts_DistFunction,GCPnts_DistFunctionMV>(a,r);}
void gb_gcp_dist(const Adaptor2d_Curve2d& a,GbReader& r){gb_gcp_dist_run<GCPnts_DistFunction2d,GCPnts_DistFunction2dMV>(a,r);}
// gcp_dist U1 U2 N X...: GCPnts_DistFunction (3D) or GCPnts_DistFunction2d (2D).
void gb_gcp_dist_case(std::istream& in){
 gb_gcp_curve(in,[](auto& a,GbReader& r){GB_TRY(gb_gcp_dist(a,r));});
}
