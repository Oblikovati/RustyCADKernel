// Copyright (c) 1995-1999 Matra Datavision
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

// The requests of the Approx multi-line drivers (tools/tkgeombase_corpus_approx.py)
// over the line and the ComputeLine instantiation of tkgeombase_probe_approx_line.cxx.
// `kind` 0 then an AppDef line (gb_apc_read_line), or kind 1 `nb3d nb2d n` then per
// point `t tangent` sampled on the source curves; then `mode cap` of the rules.
AppDef_MultiLine gb_ax_read_line(GbReader& r){
 gb_ax_rules.params.clear();
 const int kind=r.integer();gb_ax_rules.source=kind==1;
 AppDef_MultiLine line;
 if(kind==0)line=gb_apc_read_line(r);
 else{
  const int nb3d=r.integer(),nb2d=r.integer(),n=r.integer();line=AppDef_MultiLine(n);
  for(int i=1;i<=n;++i){const double t=r.real();line.SetValue(i,gb_ax_source_point(nb3d,nb2d,t,r.flag()));}
 }
 gb_ax_rules.mode=r.integer();gb_ax_rules.cap=r.integer();
 return line;
}
void gb_ax_array(const NCollection_Array1<double>& a){gb_int(a.Lower());gb_int(a.Upper());for(double v:a)gb_real(v);}
// A B-spline without poles has null knot handles (Knots() dereferences them).
void gb_ax_bspline(const AppParCurves_MultiBSpCurve& c){
 gb_apc_curve(c);gb_int(c.Degree());if(c.NbPoles()==0)return;
 gb_ax_array(c.Knots());for(int m:c.Multiplicities())gb_int(m);
}
// The options `degmin degmax tol3d tol2d iterations cutting parametrization squares`.
struct GbAxOptions{int degmin,degmax;double tol3d,tol2d;int iterations;bool cutting;Approx_ParametrizationType par;bool squares;};
GbAxOptions gb_ax_read_options(GbReader& r){
 GbAxOptions o;o.degmin=r.integer();o.degmax=r.integer();o.tol3d=r.real();o.tol2d=r.real();o.iterations=r.integer();
 o.cutting=r.flag();o.par=Approx_ParametrizationType(r.integer());o.squares=r.flag();return o;
}
// Also used for AppDef_Compute (tkgeombase_probe_appdef.cxx).
template<class C> void gb_ax_compute_results(C& c){
 gb_flag(c.IsAllApproximated());gb_flag(c.IsToleranceReached());gb_int(c.Parametrization());
 const int n=c.NbMultiCurves();gb_int(n);
 for(int i=1;i<=n;++i){
  gb_apc_curve(c.Value(i));double t3,t2;c.Error(i,t3,t2);gb_real(t3);gb_real(t2);gb_ax_array(c.Parameters(i));
 }
 GB_APC_TRY(gb_ax_bspline(c.SplineValue()));
 GB_APC_TRY(gb_apc_curve(c.Value(n+1)));
}
// `ax_compute line rules variant options firstC lastC params repeat`: variant 0
// (Line, ...), 1 (Line, Parameters, ...), 2 (...) then SetConstraints and Perform,
// 3 (Parameters, ...) then SetConstraints and Perform.
void gb_ax_compute_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_ax_read_line(r);const int variant=r.integer();const GbAxOptions o=gb_ax_read_options(r);
 const AppParCurves_Constraint fc=AppParCurves_Constraint(r.integer()),lc=AppParCurves_Constraint(r.integer());
 math_Vector u=gb_fem_read_vector(r);const bool repeat=r.flag();
 try{
  GbAxCompute c=variant==0?GbAxCompute(line,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.par,o.squares)
   :variant==1?GbAxCompute(line,u,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.squares)
   :variant==2?GbAxCompute(o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.par,o.squares)
   :GbAxCompute(u,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.squares);
  if(variant>=2){c.SetConstraints(fc,lc);c.Perform(line);}
  gb_ax_compute_results(c);
  if(repeat){c.SetDegrees(o.degmin,o.degmax+1);c.SetTolerances(o.tol3d*10.,o.tol2d*10.);c.Perform(line);gb_ax_compute_results(c);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
void gb_ax_bspl_results(AppDef_BSplineCompute& c){
 gb_flag(c.IsAllApproximated());gb_flag(c.IsToleranceReached());
 double t3,t2;c.Error(t3,t2);gb_real(t3);gb_real(t2);gb_ax_bspline(c.Value());gb_ax_array(c.Parameters());
}
// `ax_bspl line rules variant options firstC lastC continuity periodic knots-kind
// [knots] [mults] params setparams`: variant as ax_compute, 4 (...) then the setters
// and Interpol; continuity -2 skips SetContinuity; knots-kind 1 SetKnots, 2
// SetKnotsAndMultiplicities.
void gb_ax_bspl_case(std::istream& in){
 GbReader r{in};AppDef_MultiLine line=gb_ax_read_line(r);const int variant=r.integer();const GbAxOptions o=gb_ax_read_options(r);
 const AppParCurves_Constraint fc=AppParCurves_Constraint(r.integer()),lc=AppParCurves_Constraint(r.integer());
 const int cont=r.integer();const bool periodic=r.flag();const int knotkind=r.integer();
 NCollection_Array1<double> knots=knotkind>=1?gb_apc_read_knots(r):NCollection_Array1<double>(1,1);
 NCollection_Array1<int> mults=knotkind==2?gb_apc_read_mults(r):NCollection_Array1<int>(1,1);
 math_Vector u=gb_fem_read_vector(r);const bool setparams=r.flag();
 try{
  AppDef_BSplineCompute c=variant==0?AppDef_BSplineCompute(line,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.par,o.squares)
   :variant==1?AppDef_BSplineCompute(line,u,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.squares)
   :variant==3?AppDef_BSplineCompute(u,o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.squares)
   :AppDef_BSplineCompute(o.degmin,o.degmax,o.tol3d,o.tol2d,o.iterations,o.cutting,o.par,o.squares);
  if(variant>=2){
   c.SetConstraints(fc,lc);if(cont!=-2)c.SetContinuity(cont);c.SetPeriodic(periodic);
   if(knotkind==1)c.SetKnots(knots);else if(knotkind==2)c.SetKnotsAndMultiplicities(knots,mults);
   if(setparams)c.SetParameters(u);
   if(variant==4)c.Interpol(line);else c.Perform(line);
  }
  gb_ax_bspl_results(c);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// The continuous function of FitAndDivide: the source curves over [u0, u1]; D1 fails
// at the ends when asked; one output coordinate may be periodic.
class GbAxFunction:public AppCont_Function{
public:
 GbAxFunction(int nb3d,int nb2d,double u0,double u1,bool d1fail,int pdim,double period):myU0(u0),myU1(u1),myD1Fail(d1fail),myPDim(pdim),myPeriod(period){myNbPnt=nb3d;myNbPnt2d=nb2d;}
 double FirstParameter() const override{return myU0;}
 double LastParameter() const override{return myU1;}
 bool Value(const double u,NCollection_Array1<gp_Pnt2d>& p2,NCollection_Array1<gp_Pnt>& p3) const override{
  for(int k=1;k<=myNbPnt;++k)p3(k)=gb_ax_p3(k,u);
  for(int k=1;k<=myNbPnt2d;++k)p2(k)=gb_ax_p2(k,u);
  return true;
 }
 bool D1(const double u,NCollection_Array1<gp_Vec2d>& v2,NCollection_Array1<gp_Vec>& v3) const override{
  if(myD1Fail&&(u==myU0||u==myU1))return false;
  for(int k=1;k<=myNbPnt;++k)v3(k)=gb_ax_d3(k,u);
  for(int k=1;k<=myNbPnt2d;++k)v2(k)=gb_ax_d2(k,u);
  return true;
 }
 void PeriodInformation(const int dim,bool& isPeriodic,double& period) const override{isPeriodic=dim==myPDim;period=isPeriodic?myPeriod:0.;}
private:
 double myU0,myU1;bool myD1Fail;int myPDim;double myPeriod;
};
template<class F> void gb_ax_fit_results(const F& f){
 gb_flag(f.IsAllApproximated());gb_flag(f.IsToleranceReached());const int n=f.NbMultiCurves();gb_int(n);
 for(int i=1;i<=n;++i){
  gb_apc_curve(f.Value(i));double t3,t2,p1,p2;f.Error(i,t3,t2);gb_real(t3);gb_real(t2);f.Parameters(i,p1,p2);gb_real(p1);gb_real(p2);
 }
 GB_APC_TRY(gb_apc_curve(f.Value(n+1)));
}
template<class F> void gb_ax_fit_run(GbReader& r,const GbAxFunction& fn,int variant){
 const int degmin=r.integer(),degmax=r.integer();const double t3=r.real(),t2=r.real();const bool cutting=r.flag();
 const AppParCurves_Constraint fc=AppParCurves_Constraint(r.integer()),lc=AppParCurves_Constraint(r.integer());
 const int maxseg=r.integer();const bool inv=r.flag(),hang=r.flag();
 if(variant==0){F f(fn,degmin,degmax,t3,t2,cutting,fc,lc);gb_ax_fit_results(f);return;}
 F f(degmin,degmax,t3,t2,cutting,fc,lc);
 if(maxseg>0)f.SetMaxSegments(maxseg);
 f.SetInvOrder(inv);f.SetHangChecking(hang);f.Perform(fn);gb_ax_fit_results(f);
 f.SetDegrees(degmin+1,degmax);f.SetTolerances(t3*0.5,t2*0.5);f.SetConstraints(lc,fc);f.Perform(fn);gb_ax_fit_results(f);
}
// `ax_fit class nb3d nb2d u0 u1 d1fail pdim period variant degmin degmax tol3d tol2d
// cutting firstC lastC maxsegments invorder hangchecking`: class 0 FitAndDivide, 1
// FitAndDivide2d; variant 0 the performing constructor, 1 the setters then Perform
// twice (the second after SetDegrees, SetTolerances and swapped constraints).
void gb_ax_fit_case(std::istream& in){
 GbReader r{in};const int cls=r.integer(),nb3d=r.integer(),nb2d=r.integer();const double u0=r.real(),u1=r.real();
 const bool d1fail=r.flag();const int pdim=r.integer();const double period=r.real();const int variant=r.integer();
 try{
  GbAxFunction fn(nb3d,nb2d,u0,u1,d1fail,pdim,period);
  if(cls==0)gb_ax_fit_run<Approx_FitAndDivide>(r,fn,variant);else gb_ax_fit_run<Approx_FitAndDivide2d>(r,fn,variant);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
// `ax_mcurves count curves... variant`: each curve `n points...` (gb_apc_read_points);
// variant 0 Append + Perform(), 1 Perform(sequence), 2 Append, Reset, Append the
// last only, Perform().
void gb_ax_mcurves_case(std::istream& in){
 GbReader r{in};const int count=r.integer();NCollection_Sequence<AppParCurves_MultiCurve> seq;
 try{
  for(int i=0;i<count;++i)seq.Append(AppParCurves_MultiCurve(gb_apc_read_points(r)));
  const int variant=r.integer();Approx_MCurvesToBSpCurve join;
  if(variant==1)join.Perform(seq);
  else{
   for(const AppParCurves_MultiCurve& c:seq)join.Append(c);
   if(variant==2){join.Reset();if(count>0)join.Append(seq.Last());}
   join.Perform();
  }
  gb_ax_bspline(join.Value());gb_ax_bspline(join.ChangeValue());
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 gb_end();
}
