// Law probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/law.rs.
#include <Law.hxx>
#include <Law_BSpFunc.hxx>
#include <Law_BSpline.hxx>
#include <Law_BSplineKnotSplitting.hxx>
#include <Law_Composite.hxx>
#include <Law_Constant.hxx>
#include <Law_Interpol.hxx>
#include <Law_Interpolate.hxx>
#include <Law_Linear.hxx>
#include <Law_S.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Circle.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
// `n v1..vn` as a one-based array.
NCollection_Array1<double> ga_law_reals(GaReader& r){std::vector<double> v=r.reals();NCollection_Array1<double> a(1,int(v.size()));for(int i=1;i<=a.Length();i++)a(i)=v[i-1];return a;}
NCollection_Array1<int> ga_law_ints(GaReader& r){std::vector<double> v=r.reals();NCollection_Array1<int> a(1,int(v.size()));for(int i=1;i<=a.Length();i++)a(i)=int(v[i-1]);return a;}
// `degree periodic rational poles [weights] knots mults`: a Law_BSpline.
occ::handle<Law_BSpline> ga_law_bspline(GaReader& r){
 int deg=r.integer();bool per=r.flag(),rat=r.flag();
 NCollection_Array1<double> poles=ga_law_reals(r);NCollection_Array1<double> w(1,1);if(rat)w=ga_law_reals(r);
 NCollection_Array1<double> knots=ga_law_reals(r);NCollection_Array1<int> mults=ga_law_ints(r);
 if(rat)return new Law_BSpline(poles,w,knots,mults,deg,per);
 return new Law_BSpline(poles,knots,mults,deg,per);
}
// The whole state of a law: degree, flags, poles, weights, knots, multiplicities, forms.
void ga_law_state(const occ::handle<Law_BSpline>& b){
 ga_int(b->Degree());ga_flag(b->IsPeriodic());ga_flag(b->IsRational());ga_int(b->NbPoles());
 NCollection_Array1<double> p(1,b->NbPoles()),w(1,b->NbPoles());b->Poles(p);b->Weights(w);
 for(int i=1;i<=p.Length();i++){ga_real(p(i));ga_real(w(i));}
 ga_int(b->NbKnots());NCollection_Array1<double> k(1,b->NbKnots());NCollection_Array1<int> m(1,b->NbKnots());b->Knots(k);b->Multiplicities(m);
 for(int i=1;i<=k.Length();i++){ga_real(k(i));ga_int(m(i));}
 ga_int(int(b->Continuity()));ga_int(int(b->KnotDistribution()));ga_real(b->FirstParameter());ga_real(b->LastParameter());
}
// Evaluators at one parameter, each block printing its values or its exception.
void ga_law_eval(const occ::handle<Law_BSpline>& b,double u){
 GA_TRY(double v=b->Value(u);ga_real(v));
 GA_TRY(double p,d;b->D1(u,p,d);ga_real(p);ga_real(d));
 GA_TRY(double p,d1,d2;b->D2(u,p,d1,d2);ga_real(p);ga_real(d1);ga_real(d2));
 GA_TRY(double p,d1,d2,d3;b->D3(u,p,d1,d2,d3);ga_real(p);ga_real(d1);ga_real(d2);ga_real(d3));
 GA_TRY(double v=b->DN(u,2);ga_real(v));GA_TRY(double v=b->DN(u,5);ga_real(v));
 int i1=-9,i2=-9;GA_TRY(b->LocateU(u,1e-9,i1,i2,false);ga_int(i1);ga_int(i2));
 GA_TRY(b->LocateU(u,1e-9,i1,i2,true);ga_int(i1);ga_int(i2));
 GA_TRY(double v=b->LocalValue(u,1,2);ga_real(v));
 GA_TRY(double p,d1,d2,d3;b->LocalD3(u,b->NbKnots()-1,b->NbKnots(),p,d1,d2,d3);ga_real(p);ga_real(d1);ga_real(d2);ga_real(d3));
 GA_TRY(double v=b->LocalDN(u,b->NbKnots()-1,b->NbKnots(),3);ga_real(v));
}
// `law params...`: the state, the queries and the evaluators at each parameter.
void ga_law_bspline_case(std::istream& input){
 GaReader r{input};occ::handle<Law_BSpline> b;
 GA_TRY(b=ga_law_bspline(r));
 if(b.IsNull()){ga_end();return;}
 ga_law_state(b);
 GA_TRY(bool c=b->IsClosed();ga_flag(c));GA_TRY(double s=b->StartPoint();ga_real(s));GA_TRY(double e=b->EndPoint();ga_real(e));
 for(int n=-1;n<=5;n++){GA_TRY(bool c=b->IsCN(n);ga_flag(c));}
 GA_TRY(double t;b->Resolution(1e-3,t);ga_real(t));ga_real(b->ReversedParameter(0.3));
 double pn=7.25;b->PeriodicNormalization(pn);ga_real(pn);
 std::vector<double> us=r.reals();for(double u:us)ga_law_eval(b,u);
 ga_end();
}
// `law op args... u`: one edit, then the state and the evaluators at u.
void ga_law_edit_case(std::istream& input){
 GaReader r{input};occ::handle<Law_BSpline> b;
 GA_TRY(b=ga_law_bspline(r));
 if(b.IsNull()){ga_end();return;}
 int op=r.integer();double a=r.real(),c=r.real(),d=r.real(),u=r.real();
 GA_TRY(switch(op){
  case 0:b->IncreaseDegree(int(a));break;
  case 1:b->InsertKnot(a,int(c),d,true);break;
  case 2:ga_flag(b->RemoveKnot(int(a),int(c),d));break;
  case 3:b->Reverse();break;
  case 4:b->Segment(a,c);break;
  case 5:b->SetPeriodic();break;
  case 6:b->SetNotPeriodic();break;
  case 7:b->SetOrigin(int(a));break;
  case 8:b->SetWeight(int(a),c);break;
  case 9:b->SetKnot(int(a),c);break;
  case 10:{int e=-1;b->MovePointAndTangent(a,c,d,1e-7,1,1,e);ga_int(e);}break;
  case 11:b->IncreaseMultiplicity(int(a),int(c));break;
  case 12:b->IncrementMultiplicity(int(a),int(c),int(d));break;
  case 13:b->SetPole(int(a),c,d);break;
  case 14:b->SetKnot(int(a),c,int(d));break;
  case 15:b->IncreaseMultiplicity(int(a),int(c),int(d));break;
  default:{occ::handle<Law_BSpline> cp=b->Copy();b=cp;}break;
 });
 ga_law_state(b);ga_law_eval(b,u);ga_end();
}
// A Law_Function at one parameter: bounds, continuity, intervals, D2 and a trimmed copy.
void ga_law_function(const occ::handle<Law_Function>& f,double u){
 GA_TRY(double a,b;f->Bounds(a,b);ga_real(a);ga_real(b));
 GA_TRY(int c=int(f->Continuity());ga_int(c));
 for(int s=0;s<=6;s++){
  GA_TRY(int n=f->NbIntervals(GeomAbs_Shape(s));NCollection_Array1<double> t(1,n+1);f->Intervals(t,GeomAbs_Shape(s));ga_int(n);for(int i=1;i<=n+1;i++)ga_real(t(i)));
 }
 GA_TRY(double v=f->Value(u);ga_real(v));
 GA_TRY(double p,d;f->D1(u,p,d);ga_real(p);ga_real(d));
 GA_TRY(double p,d1,d2;f->D2(u,p,d1,d2);ga_real(p);ga_real(d1);ga_real(d2));
 GA_TRY(occ::handle<Law_Function> t=f->Trim(u-0.25,u+0.25,1e-9);double a,b;t->Bounds(a,b);double v=t->Value(u);ga_real(a);ga_real(b);ga_real(v));
}
// `kind args... u`: 0 constant, 1 linear, 2 S, 3 S with derivatives, 4 BSpFunc on a law,
// 5 Interpol, 6 Interpol with tangents, 7 Interpol in relative, 8 composite of linears.
void ga_law_function_case(std::istream& input){
 GaReader r{input};int kind=r.integer();occ::handle<Law_Function> f;
 GA_TRY(switch(kind){
  case 0:{occ::handle<Law_Constant> l=new Law_Constant();double v=r.real(),a=r.real(),b=r.real();l->Set(v,a,b);f=l;}break;
  case 1:{occ::handle<Law_Linear> l=new Law_Linear();double a=r.real(),va=r.real(),b=r.real(),vb=r.real();l->Set(a,va,b,vb);f=l;}break;
  case 2:{occ::handle<Law_S> l=new Law_S();double a=r.real(),va=r.real(),b=r.real(),vb=r.real();l->Set(a,va,b,vb);f=l;}break;
  case 3:{occ::handle<Law_S> l=new Law_S();double a=r.real(),va=r.real(),da=r.real(),b=r.real(),vb=r.real(),db=r.real();l->Set(a,va,da,b,vb,db);f=l;}break;
  case 4:{occ::handle<Law_BSpline> b=ga_law_bspline(r);double a=r.real(),c=r.real();f=new Law_BSpFunc(b,a,c);}break;
  case 5:case 6:case 7:{
   bool per=r.flag();std::vector<double> v=r.reals();NCollection_Array1<gp_Pnt2d> pts(1,int(v.size())/2);
   for(int i=1;i<=pts.Length();i++)pts(i)=gp_Pnt2d(v[2*i-2],v[2*i-1]);
   occ::handle<Law_Interpol> l=new Law_Interpol();double x=r.real(),y=r.real();
   if(kind==5)l->Set(pts,per);else if(kind==6)l->Set(pts,x,y,per);else l->SetInRelative(pts,x,y,per);
   f=l;}break;
  default:{
   double a=r.real(),b=r.real(),tol=r.real();
   occ::handle<Law_Composite> l=new Law_Composite(a,b,tol);if(r.flag())l->SetPeriodic();
   std::vector<double> v=r.reals();for(size_t i=0;i+3<v.size();i+=4){occ::handle<Law_Linear> p=new Law_Linear();p->Set(v[i],v[i+1],v[i+2],v[i+3]);l->ChangeLaws().Append(p);}
   f=l;}break;
 });
 double u=r.real();
 if(!f.IsNull())ga_law_function(f,u);
 ga_end();
}
// `kind args...`: 0 MixBnd law, 1 MixBnd poles, 2 MixTgt poles, 3 Scale, 4 ScaleCub,
// 5 Reparametrize a circle, 6 knot splitting.
void ga_law_mix_case(std::istream& input){
 GaReader r{input};int kind=r.integer();
 GA_TRY(switch(kind){
  case 0:{occ::handle<Law_Linear> l=new Law_Linear();double a=r.real(),va=r.real(),b=r.real(),vb=r.real();l->Set(a,va,b,vb);
   occ::handle<Law_BSpFunc> f=Law::MixBnd(l);ga_law_state(f->Curve());double u=r.real();ga_real(f->Value(u));}break;
  case 1:case 2:{int deg=r.integer();NCollection_Array1<double> k=ga_law_reals(r);NCollection_Array1<int> m=ga_law_ints(r);
   occ::handle<NCollection_HArray1<double>> p;
   if(kind==1){occ::handle<Law_Linear> l=new Law_Linear();double a=r.real(),va=r.real(),b=r.real(),vb=r.real();l->Set(a,va,b,vb);p=Law::MixBnd(deg,k,m,l);}
   else{bool nr=r.flag();int idx=r.integer();p=Law::MixTgt(deg,k,m,nr,idx);}
   ga_int(p->Length());for(int i=1;i<=p->Length();i++)ga_real(p->Value(i));}break;
  case 3:case 4:{double a=r.real(),b=r.real();bool hf=r.flag(),hl=r.flag();double va=r.real(),vb=r.real();
   occ::handle<Law_BSpline> s=kind==3?Law::Scale(a,b,hf,hl,va,vb):Law::ScaleCub(a,b,hf,hl,va,vb);ga_law_state(s);}break;
  case 5:{double rad=r.real(),f=r.real(),l=r.real();bool hdf=r.flag(),hdl=r.flag();double df=r.real(),dl=r.real();bool rev=r.flag();int nb=r.integer();
   occ::handle<Geom_Circle> c=new Geom_Circle(gp_Ax2(),rad);GeomAdaptor_Curve ac(c,0.5,4.0);
   occ::handle<Law_BSpline> s=Law::Reparametrize(ac,f,l,hdf,hdl,df,dl,rev,nb);ga_law_state(s);}break;
  default:{occ::handle<Law_BSpline> b=ga_law_bspline(r);int cont=r.integer();Law_BSplineKnotSplitting ks(b,cont);
   ga_int(ks.NbSplits());NCollection_Array1<int> sv(1,ks.NbSplits());ks.Splitting(sv);for(int i=1;i<=sv.Length();i++)ga_int(sv(i));
   GA_TRY(ga_int(ks.SplitValue(ks.NbSplits()+1)));}break;
 });
 ga_end();
}
// `periodic points [params] tangent-mode ...`: Law_Interpolate, then its law or status.
void ga_law_interpolate_case(std::istream& input){
 GaReader r{input};bool per=r.flag();
 occ::handle<NCollection_HArray1<double>> pts=new NCollection_HArray1<double>(ga_law_reals(r));
 bool withParams=r.flag();occ::handle<NCollection_HArray1<double>> par=new NCollection_HArray1<double>(ga_law_reals(r));
 int mode=r.integer();double t0=r.real(),t1=r.real();
 NCollection_Array1<double> tg=ga_law_reals(r);std::vector<double> fl=r.reals();
 GA_TRY(
  Law_Interpolate inter=withParams?Law_Interpolate(pts,par,per,1e-9):Law_Interpolate(pts,per,1e-9);
  if(mode==1)inter.Load(t0,t1);
  if(mode==2){occ::handle<NCollection_HArray1<bool>> f=new NCollection_HArray1<bool>(1,int(fl.size()));for(int i=1;i<=f->Length();i++)f->SetValue(i,fl[i-1]!=0);inter.Load(tg,f);}
  inter.Perform();ga_flag(inter.IsDone());
  if(inter.IsDone())ga_law_state(inter.Curve()););
 ga_end();
}
