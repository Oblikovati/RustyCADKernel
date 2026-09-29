// Copyright (c) 1997-1999 Matra Datavision
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

// The AdvApprox-based Approx classes (tools/tkgeombase_corpus_approx_adaptors.py):
// Approx_Curve3d, Approx_Curve2d, Approx_CurveOnSurface, Approx_CurvlinFunc,
// Approx_CurvilinearParameter and Approx_SameParameter on the GeomTools builders
// (gt_curve, gt_curve2d, gt_surface), and Approx_SweepApproximation on the probe sweep
// function of tkgeombase_probe_approx_sweep.cxx.
#include <Adaptor3d_CurveOnSurface.hxx>
#include <Approx_Curve2d.hxx>
#include <Approx_Curve3d.hxx>
#include <Approx_CurveOnSurface.hxx>
#include <Approx_CurvilinearParameter.hxx>
#include <Approx_CurvlinFunc.hxx>
#include <Approx_SameParameter.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
// A 3D adaptor: kind 0 the whole curve, 1 `u1 u2` a GeomAdaptor range.
occ::handle<Adaptor3d_Curve> gb_axa_curve(GbReader& r){
 const int kind=r.integer();double u1=0.,u2=0.;if(kind==1){u1=r.real();u2=r.real();}
 occ::handle<Geom_Curve> c=gt_curve(r);
 if(kind==1)return new GeomAdaptor_Curve(c,u1,u2);
 return new GeomAdaptor_Curve(c);
}
occ::handle<Adaptor2d_Curve2d> gb_axa_curve2d(GbReader& r){return new Geom2dAdaptor_Curve(gt_curve2d(r));}
occ::handle<Adaptor3d_Surface> gb_axa_surface(GbReader& r){return new GeomAdaptor_Surface(gt_surface(r));}
template<class T> void gb_axa_dump(const T& object){
 std::ostringstream text;object.Dump(text);
 for(char c:text.str())std::cout<<(c=='\n'?'|':c);std::cout<<' ';
}
// `axa_curve3d tol order maxseg maxdeg curve`.
void gb_axa_curve3d_case(std::istream& in){
 GbReader r{in};const double tol=r.real();const GeomAbs_Shape order=GeomAbs_Shape(r.integer());const int seg=r.integer(),deg=r.integer();
 GB_TRY(occ::handle<Adaptor3d_Curve> c=gb_axa_curve(r);Approx_Curve3d a(c,tol,order,seg,deg);
  gb_flag(a.IsDone());gb_flag(a.HasResult());gb_gcv_bspline(a.Curve());gb_real(a.MaxError());if(a.HasResult())gb_axa_dump(a));
 gb_end();
}
// `axa_curve2d first last tolu tolv continuity maxdeg maxseg curve2d`.
void gb_axa_curve2d_case(std::istream& in){
 GbReader r{in};const double f=r.real(),l=r.real(),tu=r.real(),tv=r.real();const GeomAbs_Shape cont=GeomAbs_Shape(r.integer());
 const int deg=r.integer(),seg=r.integer();
 GB_TRY(occ::handle<Adaptor2d_Curve2d> c=gb_axa_curve2d(r);Approx_Curve2d a(c,f,l,tu,tv,cont,deg,seg);
  gb_flag(a.IsDone());gb_flag(a.HasResult());gb_g2c_bspline(a.Curve());gb_real(a.MaxError2dU());gb_real(a.MaxError2dV()));
 gb_end();
}
void gb_axa_cos_results(const Approx_CurveOnSurface& a){
 gb_flag(a.IsDone());gb_flag(a.HasResult());gb_gcv_bspline(a.Curve3d());gb_real(a.MaxError3d());
 gb_g2c_bspline(a.Curve2d());gb_real(a.MaxError2dU());gb_real(a.MaxError2dV());
}
// `axa_cos variant first last tol continuity maxdeg maxseg only3d only2d curve2d
// surface`: variant 0 the performing constructor, 1 the data constructor then Perform.
void gb_axa_cos_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const double f=r.real(),l=r.real(),tol=r.real();
 const GeomAbs_Shape cont=GeomAbs_Shape(r.integer());const int deg=r.integer(),seg=r.integer();const bool o3=r.flag(),o2=r.flag();
 GB_TRY(occ::handle<Adaptor2d_Curve2d> c=gb_axa_curve2d(r);occ::handle<Adaptor3d_Surface> s=gb_axa_surface(r);
  // The performing constructor is deprecated natively but still public.
  if(variant==0){
   _Pragma("GCC diagnostic push") _Pragma("GCC diagnostic ignored \"-Wdeprecated-declarations\"")
   Approx_CurveOnSurface a(c,s,f,l,tol,cont,deg,seg,o3,o2);
   _Pragma("GCC diagnostic pop")
   gb_axa_cos_results(a);}
  else{Approx_CurveOnSurface a(c,s,f,l,tol);gb_axa_cos_results(a);a.Perform(seg,deg,cont,o3,o2);gb_axa_cos_results(a);});
 gb_end();
}
occ::handle<Approx_CurvlinFunc> gb_axa_curvlin(GbReader& r,int kind,double tol){
 if(kind==1)return new Approx_CurvlinFunc(gb_axa_curve(r),tol);
 occ::handle<Adaptor2d_Curve2d> c1=gb_axa_curve2d(r);occ::handle<Adaptor3d_Surface> s1=gb_axa_surface(r);
 if(kind==2)return new Approx_CurvlinFunc(c1,s1,tol);
 occ::handle<Adaptor2d_Curve2d> c2=gb_axa_curve2d(r);occ::handle<Adaptor3d_Surface> s2=gb_axa_surface(r);
 return new Approx_CurvlinFunc(c1,c2,s1,s2,tol);
}
// CurvlinFunc commands: 1 FirstParameter/LastParameter/GetLength, 2 NbIntervals and
// Intervals of a shape, 3 GetSParameter(u), 4 EvalCase(s, order), 5 Trim(a, b, tol),
// 6 Length() then GetLength, 7 SetTol(t), 8 GetUParameter on a new adaptor of the
// kind-1 curve (s), 9 Length(C, u1, u2) on it.
void gb_axa_curvlin_command(GbReader& r,const occ::handle<Approx_CurvlinFunc>& f,int kind,const occ::handle<Adaptor3d_Curve>& c3d){
 const int code=r.integer();
 if(code==1){gb_real(f->FirstParameter());gb_real(f->LastParameter());gb_real(f->GetLength());}
 else if(code==2){const GeomAbs_Shape s=GeomAbs_Shape(r.integer());const int n=f->NbIntervals(s);gb_int(n);
  NCollection_Array1<double> t(1,n+1);f->Intervals(t,s);for(double v:t)gb_real(v);}
 else if(code==3)gb_real(f->GetSParameter(r.real()));
 else if(code==4){const double s=r.real();const int order=r.integer();NCollection_Array1<double> res(0,2*kind);res.Init(-7.);
  gb_flag(kind==1?f->EvalCase1(s,order,res):kind==2?f->EvalCase2(s,order,res):f->EvalCase3(s,order,res));for(double v:res)gb_real(v);}
 else if(code==5){const double a=r.real(),b=r.real(),t=r.real();f->Trim(a,b,t);gb_real(f->FirstParameter());gb_real(f->LastParameter());}
 else if(code==6){f->Length();gb_real(f->GetLength());}
 else if(code==7)f->SetTol(r.real());
 else if(code==8)gb_real(f->GetUParameter(*c3d,r.real(),1));
 else{const double a=r.real(),b=r.real();gb_real(f->Length(*c3d,a,b));}
}
// `axa_curvlin kind tol curves... count commands...`; commands 8 and 9 read a kind-1
// curve built before the commands (`curve3d` after the function curves).
void gb_axa_curvlin_case(std::istream& in){
 GbReader r{in};const int kind=r.integer();const double tol=r.real();
 GB_TRY(occ::handle<Approx_CurvlinFunc> f=gb_axa_curvlin(r,kind,tol);occ::handle<Adaptor3d_Curve> c3d=gb_axa_curve(r);
  const int count=r.integer();for(int k=0;k<count;++k){GB_TRY(gb_axa_curvlin_command(r,f,kind,c3d));});
 gb_end();
}
void gb_axa_curvilinear_results(const Approx_CurvilinearParameter& a,int kind){
 gb_flag(a.IsDone());gb_flag(a.HasResult());gb_gcv_bspline(a.Curve3d());gb_real(a.MaxError3d());
 gb_g2c_bspline(a.Curve2d1());gb_real(a.MaxError2d1());gb_g2c_bspline(a.Curve2d2());
 // The case 2 constructor leaves myMaxError2d2 uninitialized.
 if(kind!=2)gb_real(a.MaxError2d2());
 gb_axa_dump(a);
}
// `axa_curvilinear kind tol order maxdeg maxseg curves...`.
void gb_axa_curvilinear_case(std::istream& in){
 GbReader r{in};const int kind=r.integer();const double tol=r.real();const GeomAbs_Shape order=GeomAbs_Shape(r.integer());
 const int deg=r.integer(),seg=r.integer();
 GB_TRY(
  if(kind==1){occ::handle<Adaptor3d_Curve> c=gb_axa_curve(r);Approx_CurvilinearParameter a(c,tol,order,deg,seg);gb_axa_curvilinear_results(a,kind);}
  else if(kind==2){occ::handle<Adaptor2d_Curve2d> c=gb_axa_curve2d(r);occ::handle<Adaptor3d_Surface> s=gb_axa_surface(r);
   Approx_CurvilinearParameter a(c,s,tol,order,deg,seg);gb_axa_curvilinear_results(a,kind);}
  else{occ::handle<Adaptor2d_Curve2d> c1=gb_axa_curve2d(r);occ::handle<Adaptor3d_Surface> s1=gb_axa_surface(r);
   occ::handle<Adaptor2d_Curve2d> c2=gb_axa_curve2d(r);occ::handle<Adaptor3d_Surface> s2=gb_axa_surface(r);
   Approx_CurvilinearParameter a(c1,s1,c2,s2,tol,order,deg,seg);gb_axa_curvilinear_results(a,kind);});
 gb_end();
}
// The results of Approx_SameParameter, and the curve on surface at 5 parameters.
void gb_axa_same_results(const Approx_SameParameter& a){
 gb_flag(a.IsDone());gb_flag(a.IsSameParameter());gb_real(a.TolReached());gb_gcv_curve2d(a.Curve2d());
 const occ::handle<Adaptor3d_CurveOnSurface> cos=a.CurveOnSurface();gb_flag(cos.IsNull());if(cos.IsNull())return;
 const double f=cos->FirstParameter(),l=cos->LastParameter();gb_real(f);gb_real(l);
 for(int i=0;i<=4;++i)gb_pnt(cos->Value(f+(l-f)*i/4.));
}
// `axa_same variant tol curve3d curve2d surface`: variant 0 Geom objects, 1 an adapted
// 3D curve and surface with a Geom2d pcurve, 2 adaptors.
void gb_axa_same_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const double tol=r.real();
 GB_TRY(occ::handle<Geom_Curve> c3=gt_curve(r);occ::handle<Geom2d_Curve> c2=gt_curve2d(r);occ::handle<Geom_Surface> s=gt_surface(r);
  occ::handle<Adaptor3d_Curve> a3=new GeomAdaptor_Curve(c3);occ::handle<Adaptor3d_Surface> as=new GeomAdaptor_Surface(s);
  if(variant==0){Approx_SameParameter a(c3,c2,s,tol);gb_axa_same_results(a);}
  else if(variant==1){Approx_SameParameter a(a3,c2,as,tol);gb_axa_same_results(a);}
  else{occ::handle<Adaptor2d_Curve2d> a2=new Geom2dAdaptor_Curve(c2);Approx_SameParameter a(a3,a2,as,tol);gb_axa_same_results(a);});
 gb_end();
}
