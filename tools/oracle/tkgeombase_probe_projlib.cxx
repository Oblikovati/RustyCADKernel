// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

// ProjLib elementary members (tools/tkgeombase_corpus_projlib.py): the projectors
// (ProjLib_Projector, _Plane, _Cylinder, _Cone, _Sphere, _Torus), the static
// ProjLib::Project overloads and IsAnaSurf, ProjLib_PrjFunc, ProjLib_PrjResolve and
// ProjLib_ProjectOnPlane, on the GeomTools builders (gt_curve, gt_surface).
#include <memory>
#include <ProjLib.hxx>
#include <ProjLib_Cone.hxx>
#include <ProjLib_Cylinder.hxx>
#include <ProjLib_Plane.hxx>
#include <ProjLib_PrjFunc.hxx>
#include <ProjLib_PrjResolve.hxx>
#include <ProjLib_ProjectOnPlane.hxx>
#include <ProjLib_Projector.hxx>
#include <ProjLib_Sphere.hxx>
#include <ProjLib_Torus.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>
// The state of a projector: done, type and periodic flags, each typed result (or
// the exception), and whether the spline handles are null.
void gb_pl_projector(const ProjLib_Projector& p){
 gb_flag(p.IsDone());gb_int(p.GetType());gb_flag(p.IsPeriodic());
 GB_TRY(gb_lin2d(p.Line()));GB_TRY(gb_circ2d(p.Circle()));GB_TRY(gb_elips2d(p.Ellipse()));
 GB_TRY(gb_hypr2d(p.Hyperbola()));GB_TRY(gb_parab2d(p.Parabola()));
 gb_flag(p.BSpline().IsNull());gb_flag(p.Bezier().IsNull());
}
// The projector of the surface kind (the base class for `which` 0 or another kind),
// Init with the surface unless `init` is 0 (the default constructor).
std::unique_ptr<ProjLib_Projector> gb_pl_make(int which,bool init,const GeomAdaptor_Surface& s){
 if(which==0)return std::make_unique<ProjLib_Projector>();
 switch(s.GetType()){
  case GeomAbs_Plane:{auto p=std::make_unique<ProjLib_Plane>();if(init)p->Init(s.Plane());return p;}
  case GeomAbs_Cylinder:{auto p=std::make_unique<ProjLib_Cylinder>();if(init)p->Init(s.Cylinder());return p;}
  case GeomAbs_Cone:{auto p=std::make_unique<ProjLib_Cone>();if(init)p->Init(s.Cone());return p;}
  case GeomAbs_Sphere:{auto p=std::make_unique<ProjLib_Sphere>();if(init)p->Init(s.Sphere());return p;}
  case GeomAbs_Torus:{auto p=std::make_unique<ProjLib_Torus>();if(init)p->Init(s.Torus());return p;}
  default:return std::make_unique<ProjLib_Projector>();
 }
}
// The virtual Project of the curve kind (other kinds are not projected).
void gb_pl_project(ProjLib_Projector& p,const GeomAdaptor_Curve& c){
 switch(c.GetType()){
  case GeomAbs_Line:p.Project(c.Line());break;
  case GeomAbs_Circle:p.Project(c.Circle());break;
  case GeomAbs_Ellipse:p.Project(c.Ellipse());break;
  case GeomAbs_Parabola:p.Project(c.Parabola());break;
  case GeomAbs_Hyperbola:p.Project(c.Hyperbola());break;
  default:break;
 }
}
// `pl_elem which init mode a b c d surface curve`: mode 1 then SetInBounds(a) on a
// sphere, 2 UFrame(a, b, c, d), 3 VFrame(a, b, c, d).
void gb_pl_elem_case(std::istream& in){
 GbReader r{in};const int which=r.integer();const bool init=r.flag();const int mode=r.integer();
 const double a=r.real(),b=r.real(),c=r.real(),d=r.real();
 GB_TRY(GeomAdaptor_Surface s(gt_surface(r));GeomAdaptor_Curve cu(gt_curve(r));
  std::unique_ptr<ProjLib_Projector> p=gb_pl_make(which,init,s);
  GB_TRY(gb_pl_project(*p,cu));
  if(mode==1){auto sp=dynamic_cast<ProjLib_Sphere*>(p.get());if(sp)GB_TRY(sp->SetInBounds(a));}
  else if(mode==2)p->UFrame(a,b,c,d);
  else if(mode==3)p->VFrame(a,b,c,d);
  gb_pl_projector(*p));
 gb_end();
}
// The static ProjLib::Project of a point on the elementary surface kinds.
void gb_pl_static_point(const GeomAdaptor_Surface& s,const gp_Pnt& p){
 switch(s.GetType()){
  case GeomAbs_Plane:gb_pnt2d(ProjLib::Project(s.Plane(),p));break;
  case GeomAbs_Cylinder:gb_pnt2d(ProjLib::Project(s.Cylinder(),p));break;
  case GeomAbs_Cone:gb_pnt2d(ProjLib::Project(s.Cone(),p));break;
  case GeomAbs_Sphere:gb_pnt2d(ProjLib::Project(s.Sphere(),p));break;
  case GeomAbs_Torus:gb_pnt2d(ProjLib::Project(s.Torus(),p));break;
  default:break;}
}
// The static ProjLib::Project overloads of a curve kind on a surface kind.
void gb_pl_static_curve(const GeomAdaptor_Surface& s,const GeomAdaptor_Curve& c){
 const GeomAbs_CurveType ct=c.GetType();
 switch(s.GetType()){
  case GeomAbs_Plane:
   if(ct==GeomAbs_Line)gb_lin2d(ProjLib::Project(s.Plane(),c.Line()));
   if(ct==GeomAbs_Circle)gb_circ2d(ProjLib::Project(s.Plane(),c.Circle()));
   if(ct==GeomAbs_Ellipse)gb_elips2d(ProjLib::Project(s.Plane(),c.Ellipse()));
   if(ct==GeomAbs_Parabola)gb_parab2d(ProjLib::Project(s.Plane(),c.Parabola()));
   if(ct==GeomAbs_Hyperbola)gb_hypr2d(ProjLib::Project(s.Plane(),c.Hyperbola()));
   break;
  case GeomAbs_Cylinder:
   if(ct==GeomAbs_Line)gb_lin2d(ProjLib::Project(s.Cylinder(),c.Line()));
   if(ct==GeomAbs_Circle)gb_lin2d(ProjLib::Project(s.Cylinder(),c.Circle()));
   break;
  case GeomAbs_Cone:
   if(ct==GeomAbs_Line)gb_lin2d(ProjLib::Project(s.Cone(),c.Line()));
   if(ct==GeomAbs_Circle)gb_lin2d(ProjLib::Project(s.Cone(),c.Circle()));
   break;
  case GeomAbs_Sphere:if(ct==GeomAbs_Circle)gb_lin2d(ProjLib::Project(s.Sphere(),c.Circle()));break;
  case GeomAbs_Torus:if(ct==GeomAbs_Circle)gb_lin2d(ProjLib::Project(s.Torus(),c.Circle()));break;
  default:break;}
}
// `pl_static mode surface (x y z | curve)`: IsAnaSurf, then the static Project of a
// point (mode 0) or of a curve (mode 1).
void gb_pl_static_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();
 GB_TRY(occ::handle<GeomAdaptor_Surface> hs=new GeomAdaptor_Surface(gt_surface(r));
  gb_flag(ProjLib::IsAnaSurf(hs));
  if(mode==0)gb_pl_static_point(*hs,r.pnt());
  else{GeomAdaptor_Curve c(gt_curve(r));gb_pl_static_curve(*hs,c);});
 gb_end();
}
// `pl_prjfunc fix fixval x1 x2 curve surface`: PrjFunc counts, Value, Derivatives,
// Values and Solution after each evaluation.
void gb_pl_prjfunc_case(std::istream& in){
 GbReader r{in};const int fix=r.integer();const double fv=r.real(),x1=r.real(),x2=r.real();
 GB_TRY(GeomAdaptor_Curve c(gt_curve(r));GeomAdaptor_Surface s(gt_surface(r));
  ProjLib_PrjFunc f(&c,fv,&s,fix);gb_int(f.NbVariables());gb_int(f.NbEquations());gb_pnt2d(f.Solution());
  math_Vector x(1,2),v(1,2);math_Matrix m(1,2,1,2);x(1)=x1;x(2)=x2;
  gb_flag(f.Value(x,v));gb_real(v(1));gb_real(v(2));gb_pnt2d(f.Solution());
  x(1)=x1+0.125;gb_flag(f.Derivatives(x,m));for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)gb_real(m(i,j));
  x(2)=x2-0.25;gb_flag(f.Values(x,v,m));gb_real(v(1));gb_real(v(2));
  for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)gb_real(m(i,j));gb_pnt2d(f.Solution()));
 gb_end();
}
// `pl_prjresolve fix t u v tolx toly infx infy supx supy ftol strict curve surface`.
void gb_pl_prjresolve_case(std::istream& in){
 GbReader r{in};const int fix=r.integer();const double t=r.real(),u=r.real(),v=r.real();
 const gp_Pnt2d tol=r.pnt2d(),inf=r.pnt2d(),sup=r.pnt2d();const double ftol=r.real();const bool strict=r.flag();
 GB_TRY(GeomAdaptor_Curve c(gt_curve(r));GeomAdaptor_Surface s(gt_surface(r));
  ProjLib_PrjResolve p(c,s,fix);gb_flag(p.IsDone());GB_TRY(gb_pnt2d(p.Solution()));
  p.Perform(t,u,v,tol,inf,sup,ftol,strict);gb_flag(p.IsDone());GB_TRY(gb_pnt2d(p.Solution())));
 gb_end();
}
// The adaptor members of a ProjectOnPlane: bounds, periodicity, continuity and
// intervals, evaluations at five parameters, the typed results, trim and copy.
void gb_pl_on_plane_eval(const Adaptor3d_Curve& p,double u){
 GB_TRY(gb_pnt(p.Value(u)));
 GB_TRY(gp_Pnt q;gp_Vec v1;p.D1(u,q,v1);gb_pnt(q);gb_vec(v1));
 GB_TRY(gp_Pnt q;gp_Vec v1,v2;p.D2(u,q,v1,v2);gb_pnt(q);gb_vec(v1);gb_vec(v2));
 GB_TRY(gp_Pnt q;gp_Vec v1,v2,v3;p.D3(u,q,v1,v2,v3);gb_pnt(q);gb_vec(v1);gb_vec(v2);gb_vec(v3));
 GB_TRY(gb_vec(p.DN(u,2)));
}
void gb_pl_on_plane_typed(const ProjLib_ProjectOnPlane& p){
 GB_TRY(gb_lin(p.Line()));GB_TRY(gb_circ(p.Circle()));GB_TRY(gb_elips(p.Ellipse()));
 GB_TRY(gb_hypr(p.Hyperbola()));GB_TRY(gb_parab(p.Parabola()));
 GB_TRY(gb_int(p.Degree()));GB_TRY(gb_flag(p.IsRational()));GB_TRY(gb_int(p.NbPoles()));GB_TRY(gb_int(p.NbKnots()));
 GB_TRY(gb_gcv_bezier(p.Bezier()));GB_TRY(gb_gcv_bspline(p.BSpline()));
}
void gb_pl_on_plane_results(const occ::handle<ProjLib_ProjectOnPlane>& p){
 gb_int(p->GetType());gb_ax3(p->GetPlane());gb_dir(p->GetDirection());
 gb_flag(p->GetResult().IsNull());gb_flag(p->GetCurve().IsNull());
 GB_TRY(gb_flag(p->IsPeriodic()));GB_TRY(gb_real(p->Period()));GB_TRY(gb_flag(p->IsClosed()));
 GB_TRY(gb_int(p->Continuity()));GB_TRY(gb_real(p->Resolution(1.0e-3)));
 GB_TRY(const int n=p->NbIntervals(GeomAbs_C2);gb_int(n);NCollection_Array1<double> t(1,n+1);p->Intervals(t,GeomAbs_C2);for(double v:t)gb_real(v));
 gb_pl_on_plane_typed(*p);
 GB_TRY(const double f=p->FirstParameter(),l=p->LastParameter();gb_real(f);gb_real(l);
  for(int i=0;i<=4;++i)gb_pl_on_plane_eval(*p,f+(l-f)*i/4.);
  GB_TRY(occ::handle<Adaptor3d_Curve> t=p->Trim(f+(l-f)/8.,l-(l-f)/8.,1.0e-7);gb_real(t->FirstParameter());gb_real(t->LastParameter());gb_pnt(t->Value(0.5*(f+l))));
  GB_TRY(occ::handle<Adaptor3d_Curve> c=p->ShallowCopy();gb_int(c->GetType());gb_pl_on_plane_eval(*c,0.5*(f+l))));
}
// `pl_on_plane variant keep tol ax3 dir curve`: variant 0 the default constructor,
// 1 (Pl), 2 (Pl, D); then Load(C, tol, keep); variant 3 loads twice (the curve,
// then the same curve without keeping the parametrization).
void gb_pl_on_plane_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const bool keep=r.flag();const double tol=r.real();
 const gp_Ax3 pl=gt_ax3(r);const gp_Dir d=r.dir();
 GB_TRY(occ::handle<Adaptor3d_Curve> c=gb_axa_curve(r);
  occ::handle<ProjLib_ProjectOnPlane> p=variant==0?new ProjLib_ProjectOnPlane():variant==1?new ProjLib_ProjectOnPlane(pl):new ProjLib_ProjectOnPlane(pl,d);
  gb_int(p->GetType());p->Load(c,tol,keep);if(variant==3)p->Load(c,tol,!keep);gb_pl_on_plane_results(p));
 gb_end();
}
