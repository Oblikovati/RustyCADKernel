// TKGeomBase oracle: the LProp and GeomLProp packages (tkgeombase_probe.cxx). Every
// request builds its curve or surface with tkgeombase_probe_lprop_build.cxx, then calls
// the property members in a fixed order selected by a mode, because the classes compute
// lazily and several native quirks depend on the call order. Each member prints its
// value or the raised exception type. The Rust twin is gb_probe/lprop/mod.rs.
#include <Adaptor3d_Curve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomLProp.hxx>
#include <GeomLProp_CLProps.hxx>
#include <GeomLProp_CurAndInf2d.hxx>
#include <GeomLProp_SLProps.hxx>
#include <LProp_CLProps3d.hxx>
#include <LProp_CurAndInf.hxx>
#include "tkgeombase_probe_lprop_build.cxx"
void gb_lp(const gp_Pnt& p){gb_pnt(p);}
void gb_lp(const gp_Pnt2d& p){gb_pnt2d(p);}
void gb_lp(const gp_Vec& v){gb_vec(v);}
void gb_lp(const gp_Vec2d& v){gb_vec2d(v);}
void gb_lp(const gp_Dir& d){gb_dir(d);}
void gb_lp(const gp_Dir2d& d){gb_dir2d(d);}
// The members of a curve property object, in the order of `mode`:
// 0 natural (Value, IsTangentDefined, Tangent, Curvature, Normal, Centre, D1, D2, D3, Value),
// 1 stale (Centre before Curvature, then Normal, Curvature, Centre, Tangent, D3, Value).
template<class Props,class Pnt,class Vec,class Dir>
void gb_lp_clprops_members(Props& p,int mode){
 Pnt c;Dir d;
 if(mode==1){
  GB_TRY(p.CentreOfCurvature(c);gb_lp(c));GB_TRY(p.Normal(d);gb_lp(d));GB_TRY(gb_real(p.Curvature()));
  GB_TRY(p.CentreOfCurvature(c);gb_lp(c));GB_TRY(p.Tangent(d);gb_lp(d));GB_TRY(gb_lp(p.D3()));gb_lp(p.Value());
  return;
 }
 gb_lp(p.Value());GB_TRY(gb_flag(p.IsTangentDefined()));GB_TRY(p.Tangent(d);gb_lp(d));
 GB_TRY(gb_real(p.Curvature()));GB_TRY(p.Normal(d);gb_lp(d));GB_TRY(p.CentreOfCurvature(c);gb_lp(c));
 GB_TRY(gb_lp(p.D1()));GB_TRY(gb_lp(p.D2()));GB_TRY(gb_lp(p.D3()));gb_lp(p.Value());
}
// Modes 0 and 1 build one object per parameter (C, U, N, Resolution); mode 2 shares one
// object (C, N, Resolution) moved by SetParameter; mode 3 starts without a curve
// (N, Resolution), tests the tangent, then SetCurve and SetParameter.
template<class Props,class Pnt,class Vec,class Dir,class Curve>
void gb_lp_clprops_run(const Curve& curve,GbReader& r){
 const int mode=r.integer(),n=r.integer();const double tol=r.real();const int np=r.integer();
 std::unique_ptr<Props> shared;
 if(mode==2){GB_TRY(shared.reset(new Props(curve,n,tol)));if(!shared){gb_end();return;}}
 for(int i=0;i<np;++i){
  const double u=r.real();
  if(mode==2){GB_TRY(shared->SetParameter(u));GB_TRY(gb_real(shared->Curvature()));Pnt c;Dir d;
   GB_TRY(shared->CentreOfCurvature(c);gb_lp(c));GB_TRY(shared->Tangent(d);gb_lp(d));GB_TRY(gb_lp(shared->D1()));continue;}
  if(mode==3){try{Props p(n,tol);gb_flag(p.IsTangentDefined());Dir d;GB_TRY(p.Tangent(d);gb_lp(d));p.SetCurve(curve);
   p.SetParameter(u);gb_lp_clprops_members<Props,Pnt,Vec,Dir>(p,0);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}continue;}
  try{Props p(curve,u,n,tol);gb_lp_clprops_members<Props,Pnt,Vec,Dir>(p,mode);}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 }
 gb_end();
}
// lprop_cl2d: GeomLProp_CLProps2d on a Geom2d curve.
void gb_lprop_cl2d_case(std::istream& in){
 GbReader r{in};occ::handle<Geom2d_Curve> c;
 try{c=gb_lp_curve2d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 gb_lp_clprops_run<GeomLProp_CLProps2d,gp_Pnt2d,gp_Vec2d,gp_Dir2d>(c,r);
}
// lprop_cl3d: GeomLProp_CLProps on a Geom curve.
void gb_lprop_cl3d_case(std::istream& in){
 GbReader r{in};occ::handle<Geom_Curve> c;
 try{c=gb_lp_curve3d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 gb_lp_clprops_run<GeomLProp_CLProps,gp_Pnt,gp_Vec,gp_Dir>(c,r);
}
// lprop_cl3d_adaptor: LProp_CLProps3d on a GeomAdaptor_Curve of a Geom curve.
void gb_lprop_cl3d_adaptor_case(std::istream& in){
 GbReader r{in};occ::handle<Adaptor3d_Curve> c;
 try{c=new GeomAdaptor_Curve(gb_lp_curve3d(r));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 gb_lp_clprops_run<LProp_CLProps3d,gp_Pnt,gp_Vec,gp_Dir>(c,r);
}
// The members of a surface property object, in the order of `mode`:
// 0 tangents first (the by-value significant order), 1 curvature getters first (the
// by-value curvature reads), then the rest.
void gb_lp_slprops_members(GeomLProp_SLProps& p,int mode){
 gp_Dir d,e;gb_pnt(p.Value());
 if(mode==1){
  GB_TRY(gb_real(p.MaxCurvature()));GB_TRY(gb_real(p.MinCurvature()));GB_TRY(gb_real(p.MeanCurvature()));
  GB_TRY(gb_real(p.GaussianCurvature()));GB_TRY(gb_flag(p.IsUmbilic()));
 }
 GB_TRY(p.TangentU(d);gb_dir(d));GB_TRY(p.TangentV(d);gb_dir(d));
 GB_TRY(gb_flag(p.IsTangentUDefined()));GB_TRY(gb_flag(p.IsTangentVDefined()));
 GB_TRY(gb_flag(p.IsNormalDefined()));GB_TRY(gb_dir(p.Normal()));GB_TRY(gb_flag(p.IsCurvatureDefined()));
 GB_TRY(gb_real(p.MaxCurvature()));GB_TRY(gb_real(p.MinCurvature()));GB_TRY(gb_real(p.MeanCurvature()));
 GB_TRY(gb_real(p.GaussianCurvature()));GB_TRY(gb_flag(p.IsUmbilic()));
 GB_TRY(p.CurvatureDirections(d,e);gb_dir(d);gb_dir(e));
 GB_TRY(p.TangentU(d);gb_dir(d));GB_TRY(p.TangentV(d);gb_dir(d));
 GB_TRY(gb_vec(p.D1U()));GB_TRY(gb_vec(p.D1V()));GB_TRY(gb_vec(p.D2U()));GB_TRY(gb_vec(p.D2V()));GB_TRY(gb_vec(p.DUV()));
}
// lprop_sl: GeomLProp_SLProps. Modes 0/1 build one object per (U, V); mode 2 shares one
// object moved by SetParameters; mode 3 starts without a surface, then SetSurface.
void gb_lprop_sl_case(std::istream& in){
 GbReader r{in};occ::handle<Geom_Surface> s;
 try{s=gb_lp_surface(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 const int mode=r.integer(),n=r.integer();const double tol=r.real();const int np=r.integer();
 std::unique_ptr<GeomLProp_SLProps> shared;
 if(mode==2){GB_TRY(shared.reset(new GeomLProp_SLProps(s,n,tol)));if(!shared){gb_end();return;}}
 for(int i=0;i<np;++i){
  const double u=r.real(),v=r.real();
  if(mode==2){GB_TRY(shared->SetParameters(u,v);gb_lp_slprops_members(*shared,1));continue;}
  if(mode==3){try{GeomLProp_SLProps p(n,tol);gb_flag(p.IsNormalDefined());gb_flag(p.IsCurvatureDefined());
   p.SetSurface(s);p.SetParameters(u,v);gb_lp_slprops_members(p,0);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}continue;}
  try{GeomLProp_SLProps p(s,u,v,n,tol);gb_lp_slprops_members(p,mode);}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 }
 gb_end();
}
void gb_lp_points(const LProp_CurAndInf& p){
 gb_flag(p.IsEmpty());gb_int(p.NbPoints());
 for(int i=1;i<=p.NbPoints();++i){gb_real(p.Parameter(i));gb_int(p.Type(i));}
}
// lprop_curinf2d: GeomLProp_CurAndInf2d; mode 0 Perform, 1 PerformCurExt, 2 PerformInf,
// 3 PerformCurExt then PerformInf on the same object.
void gb_lprop_curinf2d_case(std::istream& in){
 GbReader r{in};occ::handle<Geom2d_Curve> c;
 try{c=gb_lp_curve2d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 const int mode=r.integer();GeomLProp_CurAndInf2d a;gb_flag(a.IsDone());
 try{
  if(mode==0)a.Perform(c);
  if(mode==1||mode==3)a.PerformCurExt(c);
  if(mode==3){gb_flag(a.IsDone());gb_lp_points(a);a.PerformInf(c);}
  if(mode==2)a.PerformInf(c);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_flag(a.IsDone());gb_lp_points(a);gb_end();
}
// lprop_curandinf: LProp_CurAndInf filled by (op, parameter) pairs (0 inflection, 1 minimum,
// 2 maximum, 3 Clear), then Parameter and Type read at the given indices.
void gb_lprop_curandinf_case(std::istream& in){
 GbReader r{in};LProp_CurAndInf p;const int n=r.integer();
 for(int i=0;i<n;++i){const int op=r.integer();const double u=r.real();
  if(op==0)p.AddInflection(u);else if(op==3)p.Clear();else p.AddExtCur(u,op==1);}
 gb_lp_points(p);const int nq=r.integer();
 for(int i=0;i<nq;++i){const int q=r.integer();GB_TRY(gb_real(p.Parameter(q)));GB_TRY(gb_int(p.Type(q)));}
 gb_end();
}
// lprop_continuity: GeomLProp::Continuity of two Geom curves; with_tol selects the
// explicit-tolerance overload.
void gb_lprop_continuity_case(std::istream& in){
 GbReader r{in};occ::handle<Geom_Curve> c1,c2;
 try{c1=gb_lp_curve3d(r);c2=gb_lp_curve3d(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 const double u1=r.real(),u2=r.real();const bool r1=r.flag(),r2=r.flag(),with_tol=r.flag();
 const double tl=r.real(),ta=r.real();
 if(with_tol){GB_TRY(gb_int(GeomLProp::Continuity(c1,c2,u1,u2,r1,r2,tl,ta)));}
 else{GB_TRY(gb_int(GeomLProp::Continuity(c1,c2,u1,u2,r1,r2)));}
 gb_end();
}
