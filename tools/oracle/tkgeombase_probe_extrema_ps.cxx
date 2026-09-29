// TKGeomBase oracle: the point-surface classes of the Extrema package
// (Extrema_FuncPSNorm, Extrema_FuncPSDist, Extrema_GenLocateExtPS, Extrema_GenExtPS,
// Extrema_ExtPExtS, Extrema_ExtPRevS, Extrema_ExtPS). Twin:
// crates/tkgeombase/examples/gb_probe/extrema_ps.rs. Surfaces are read with
// gb_bnd_surface (tkgeombase_probe_bndlib.cxx), then a ranged flag and the bounds of
// the adaptor.
#include <Extrema_ExtPExtS.hxx>
#include <Extrema_ExtPRevS.hxx>
#include <Extrema_ExtPS.hxx>
#include <Extrema_FuncPSDist.hxx>
#include <Extrema_FuncPSNorm.hxx>
#include <Extrema_GenExtPS.hxx>
#include <Extrema_GenLocateExtPS.hxx>
#include <GeomAdaptor_SurfaceOfLinearExtrusion.hxx>
#include <GeomAdaptor_SurfaceOfRevolution.hxx>
std::unique_ptr<GeomAdaptor_Surface> gb_ex_surface(GbReader& r){
 occ::handle<Geom_Surface> s=gb_bnd_surface(r);bool ranged=r.flag();
 if(ranged){double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();return std::unique_ptr<GeomAdaptor_Surface>(new GeomAdaptor_Surface(s,u1,u2,v1,v2));}
 return std::unique_ptr<GeomAdaptor_Surface>(new GeomAdaptor_Surface(s));
}
// The results of an algorithm with NbExt, SquareDistance(N) and Point(N).
template<class E> void gb_ex_ps_result(const E& e){
 gb_flag(e.IsDone());
 GB_TRY(int n=e.NbExt();gb_int(n);for(int i=1;i<=n;++i){gb_real(e.SquareDistance(i));gb_ponsurf(e.Point(i));});
 GB_TRY(gb_real(e.SquareDistance(0)));
}
// ex_fpsn: <surface> P nq (u v)*: Extrema_FuncPSNorm: Value, Derivatives, Values and a
// saved state at each pair, the saved extrema, SetPoint and Initialize; the default
// object raises.
void gb_ex_fpsn_case(std::istream& in){
 GbReader r{in};
 try{
  auto s=gb_ex_surface(r);gp_Pnt p=r.pnt();int n=r.integer();
  Extrema_FuncPSNorm d;math_Vector uv0(1,2,0.5),f0(1,2,0.0);GB_TRY(gb_flag(d.Value(uv0,f0)));GB_TRY(gb_int(d.GetStateNumber()));
  gb_int(d.NbExt());GB_TRY(gb_real(d.SquareDistance(1)));GB_TRY(gb_ponsurf(d.Point(1)));
  Extrema_FuncPSNorm f(p,*s);gb_int(f.NbVariables());gb_int(f.NbEquations());
  for(int q=0;q<n;++q){math_Vector uv(1,2),v(1,2,0.0),w(1,2,0.0);uv(1)=r.real();uv(2)=r.real();math_Matrix m(1,2,1,2,0.0);
   GB_TRY(gb_flag(f.Value(uv,v));gb_real(v(1));gb_real(v(2)));
   GB_TRY(gb_flag(f.Derivatives(uv,m));for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)gb_real(m(i,j)));
   GB_TRY(gb_flag(f.Values(uv,w,m));gb_real(w(1));gb_real(w(2));for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)gb_real(m(i,j)));
   GB_TRY(gb_int(f.GetStateNumber()));}
  gb_int(f.NbExt());
  for(int i=0;i<=f.NbExt()+1;++i)GB_TRY(gb_real(f.SquareDistance(i));gb_ponsurf(f.Point(i)));
  f.SetPoint(p.Translated(gp_Vec(1.0,0.0,0.0)));gb_int(f.NbExt());
  math_Vector uv(1,2,0.25),v(1,2,0.0);GB_TRY(gb_flag(f.Value(uv,v));gb_int(f.GetStateNumber());gb_int(f.NbExt()));
  f.Initialize(*s);gb_int(f.NbExt());
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_fpsd: <surface> P nq (u v)*: Extrema_FuncPSDist Value, Gradient and Values.
void gb_ex_fpsd_case(std::istream& in){
 GbReader r{in};
 try{
  auto s=gb_ex_surface(r);gp_Pnt p=r.pnt();int n=r.integer();
  Extrema_FuncPSDist f(*s,p);gb_int(f.NbVariables());
  for(int q=0;q<n;++q){math_Vector x(1,2),g(1,2,0.0);x(1)=r.real();x(2)=r.real();double v=0;
   GB_TRY(gb_flag(f.Value(x,v));gb_real(v));
   GB_TRY(gb_flag(f.Gradient(x,g));gb_real(g(1));gb_real(g(2)));
   GB_TRY(v=0;g(1)=0;g(2)=0;gb_flag(f.Values(x,v,g));gb_real(v);gb_real(g(1));gb_real(g(2)));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_glps: <surface> P tolu tolv nq (u0 v0)*: Extrema_GenLocateExtPS by the normal and
// the distance criteria, and IsMinDist, from each start.
void gb_ex_glps_case(std::istream& in){
 GbReader r{in};
 try{
  auto s=gb_ex_surface(r);gp_Pnt p=r.pnt();double tu=r.real(),tv=r.real();int n=r.integer();
  Extrema_GenLocateExtPS e(*s,tu,tv);Extrema_GenLocateExtPS d(*s);
  gb_flag(e.IsDone());GB_TRY(gb_real(e.SquareDistance()));GB_TRY(gb_ponsurf(e.Point()));
  for(int q=0;q<n;++q){double u0=r.real(),v0=r.real();
   for(int crit=0;crit<2;++crit)GB_TRY(e.Perform(p,u0,v0,crit==1);gb_flag(e.IsDone());GB_TRY(gb_real(e.SquareDistance());gb_ponsurf(e.Point())));
   GB_TRY(d.Perform(p,u0,v0);gb_flag(d.IsDone());GB_TRY(gb_real(d.SquareDistance())));
   GB_TRY(gb_flag(Extrema_GenLocateExtPS::IsMinDist(p,*s,u0,v0)));}
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_genps: <surface> P nbu nbv tolu tolv flag algo u1 u2 v1 v2 P2: Extrema_GenExtPS
// on the surface bounds and on the given ones, performed again (a second point, a
// changed flag and algorithm); Initialize with too few samples raises.
void gb_ex_genps_case(std::istream& in){
 GbReader r{in};
 try{
  auto s=gb_ex_surface(r);gp_Pnt p=r.pnt();int nu=r.integer(),nv=r.integer();double tu=r.real(),tv=r.real();
  Extrema_ExtFlag fl=Extrema_ExtFlag(r.integer());Extrema_ExtAlgo al=Extrema_ExtAlgo(r.integer());
  double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();gp_Pnt p2=r.pnt();
  GB_TRY(Extrema_GenExtPS e(p,*s,nu,nv,tu,tv,fl,al);gb_ex_ps_result(e);GB_TRY(e.Perform(p2);gb_ex_ps_result(e));
   e.SetFlag(Extrema_ExtFlag_MINMAX);GB_TRY(e.Perform(p);gb_ex_ps_result(e)));
  GB_TRY(Extrema_GenExtPS e(p,*s,nu,nv,u1,u2,v1,v2,tu,tv,fl,al);gb_ex_ps_result(e);
   e.SetAlgo(al==Extrema_ExtAlgo_Grad?Extrema_ExtAlgo_Tree:Extrema_ExtAlgo_Grad);GB_TRY(e.Perform(p2);gb_ex_ps_result(e));
   e.SetAlgo(al);GB_TRY(e.Perform(p);gb_ex_ps_result(e)));
  Extrema_GenExtPS d;gb_flag(d.IsDone());GB_TRY(gb_int(d.NbExt()));GB_TRY(d.Initialize(*s,1,nv,tu,tv));
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_ps: <surface> P tolu tolv flag algo u1 u2 v1 v2 P2: Extrema_ExtPS on the surface
// bounds and on the given ones (performed again after SetFlag/SetAlgo), and a
// default object initialized on the given bounds with its trimmed distances.
void gb_ex_ps_case(std::istream& in){
 GbReader r{in};
 try{
  auto s=gb_ex_surface(r);gp_Pnt p=r.pnt();double tu=r.real(),tv=r.real();
  Extrema_ExtFlag fl=Extrema_ExtFlag(r.integer());Extrema_ExtAlgo al=Extrema_ExtAlgo(r.integer());
  double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();gp_Pnt p2=r.pnt();
  GB_TRY(Extrema_ExtPS e(p,*s,tu,tv,fl,al);gb_ex_ps_result(e);GB_TRY(e.Perform(p2);gb_ex_ps_result(e)));
  GB_TRY(Extrema_ExtPS e(p,*s,u1,u2,v1,v2,tu,tv,fl,al);gb_ex_ps_result(e);e.SetFlag(Extrema_ExtFlag_MINMAX);
   e.SetAlgo(al==Extrema_ExtAlgo_Grad?Extrema_ExtAlgo_Tree:Extrema_ExtAlgo_Grad);GB_TRY(e.Perform(p2);gb_ex_ps_result(e)));
  Extrema_ExtPS d;gb_flag(d.IsDone());GB_TRY(gb_int(d.NbExt()));
  GB_TRY(d.Initialize(*s,u1,u2,v1,v2,tu,tv);d.Perform(p);gb_ex_ps_result(d);
   double t[4];gp_Pnt q[4];d.TrimmedSquareDistances(t[0],t[1],t[2],t[3],q[0],q[1],q[2],q[3]);for(int i=0;i<4;++i){gb_real(t[i]);gb_pnt(q[i]);});
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// The basis curve adaptor of a swept surface (gb_ex_adaptor, as a handle).
occ::handle<Adaptor3d_Curve> gb_ex_basis_curve(GbReader& r){
 auto a=gb_ex_adaptor<GbSpace3>(r);return new GeomAdaptor_Curve(*a);
}
// The runs of a swept-surface algorithm E on the handle `s`: on the surface bounds
// (performed again), on the given ones, and a default object initialized twice on
// the same handle.
template<class E,class H> void gb_ex_swept_runs(const H& s,const gp_Pnt& p,const gp_Pnt& p2,double u1,double u2,double v1,double v2,double tu,double tv){
 GB_TRY(E e(p,s,tu,tv);gb_ex_ps_result(e);GB_TRY(e.Perform(p2);gb_ex_ps_result(e)));
 GB_TRY(E e(p,s,u1,u2,v1,v2,tu,tv);gb_ex_ps_result(e));
 E d;gb_flag(d.IsDone());GB_TRY(gb_int(d.NbExt()));
 GB_TRY(d.Initialize(s,u1,u2,v1,v2,tu,tv);d.Perform(p);gb_ex_ps_result(d);
  d.Initialize(s,u1,u2,v1,v2,tu,tv);GB_TRY(d.Perform(p2);gb_ex_ps_result(d)));
}
// ex_pexts: <curve adaptor> dir P u1 u2 v1 v2 tolu tolv P2: Extrema_ExtPExtS.
void gb_ex_pexts_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Adaptor3d_Curve> c=gb_ex_basis_curve(r);gp_Dir dir=r.dir();gp_Pnt p=r.pnt();
  double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tu=r.real(),tv=r.real();gp_Pnt p2=r.pnt();
  occ::handle<GeomAdaptor_SurfaceOfLinearExtrusion> s=new GeomAdaptor_SurfaceOfLinearExtrusion(c,dir);
  gb_ex_swept_runs<Extrema_ExtPExtS>(s,p,p2,u1,u2,v1,v2,tu,tv);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
// ex_prevs: <curve adaptor> ax1 P u1 u2 v1 v2 tolu tolv P2: Extrema_ExtPRevS.
void gb_ex_prevs_case(std::istream& in){
 GbReader r{in};
 try{
  occ::handle<Adaptor3d_Curve> c=gb_ex_basis_curve(r);gp_Ax1 ax=r.ax1();gp_Pnt p=r.pnt();
  double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tu=r.real(),tv=r.real();gp_Pnt p2=r.pnt();
  occ::handle<GeomAdaptor_SurfaceOfRevolution> s=new GeomAdaptor_SurfaceOfRevolution(c,ax);
  gb_ex_swept_runs<Extrema_ExtPRevS>(s,p,p2,u1,u2,v1,v2,tu,tv);
 }catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
 gb_end();
}
