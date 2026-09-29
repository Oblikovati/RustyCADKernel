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

// Approx_SweepApproximation on a probe sweep function
// (tools/tkgeombase_corpus_approx_adaptors.py): a Bezier section of NbPol poles moving
// on polynomial paths, weights 1 + 0.1 k t when rational, NbCurves2d pcurves (t, 0.5 j
// + t^2); D1, D2 and Resolution either implemented or left to the native
// Standard_NotImplemented bodies.
#include <Approx_SweepApproximation.hxx>
#include <Approx_SweepFunction.hxx>
class GbAxSweep : public Approx_SweepFunction{
public:
 int nbpol=0,nb2d=0,d1mode=0,d2mode=0,nbc2=1,nbc3=1;bool rational=false,resolution=false;double pfirst=0.,plast=1.;
 double first=0.,last=0.,tol3d=0.,tol2d=0.;int nbinterval=0;
 static gp_XYZ pole(int k,double t){return gp_XYZ(t+0.3*k,0.5*t*t+0.1*k*t,0.2*k-0.1*t*t*t);}
 static gp_XYZ dpole(int k,double t){return gp_XYZ(1.0,t+0.1*k,-0.3*t*t);}
 static gp_XYZ d2pole(int,double t){return gp_XYZ(0.0,1.0,-0.6*t);}
 double weight(int k,double t) const{return rational?1.0+0.1*k*t:1.0;}
 double dweight(int k) const{return rational?0.1*k:0.0;}
 void values(double t,NCollection_Array1<gp_Pnt>& p,NCollection_Array1<gp_Pnt2d>& p2,NCollection_Array1<double>& w) const{
  for(int k=1;k<=nbpol;++k){p(k)=gp_Pnt(pole(k,t));w(k)=weight(k,t);}
  for(int j=1;j<=nb2d;++j)p2(j)=gp_Pnt2d(t,0.5*j+t*t);
 }
 void firsts(double t,NCollection_Array1<gp_Vec>& dp,NCollection_Array1<gp_Vec2d>& dp2,NCollection_Array1<double>& dw) const{
  for(int k=1;k<=nbpol;++k){dp(k)=gp_Vec(dpole(k,t));dw(k)=dweight(k);}
  for(int j=1;j<=nb2d;++j)dp2(j)=gp_Vec2d(1.0,2.0*t);
 }
 bool D0(const double t,const double,const double,NCollection_Array1<gp_Pnt>& p,NCollection_Array1<gp_Pnt2d>& p2,NCollection_Array1<double>& w) override{
  values(t,p,p2,w);return true;
 }
 bool D1(const double t,const double f,const double l,NCollection_Array1<gp_Pnt>& p,NCollection_Array1<gp_Vec>& dp,NCollection_Array1<gp_Pnt2d>& p2,
         NCollection_Array1<gp_Vec2d>& dp2,NCollection_Array1<double>& w,NCollection_Array1<double>& dw) override{
  if(d1mode==0)return Approx_SweepFunction::D1(t,f,l,p,dp,p2,dp2,w,dw);
  values(t,p,p2,w);firsts(t,dp,dp2,dw);return d1mode==1;
 }
 bool D2(const double t,const double f,const double l,NCollection_Array1<gp_Pnt>& p,NCollection_Array1<gp_Vec>& dp,NCollection_Array1<gp_Vec>& d2p,
         NCollection_Array1<gp_Pnt2d>& p2,NCollection_Array1<gp_Vec2d>& dp2,NCollection_Array1<gp_Vec2d>& d2p2,
         NCollection_Array1<double>& w,NCollection_Array1<double>& dw,NCollection_Array1<double>& d2w) override{
  if(d2mode==0)return Approx_SweepFunction::D2(t,f,l,p,dp,d2p,p2,dp2,d2p2,w,dw,d2w);
  values(t,p,p2,w);firsts(t,dp,dp2,dw);
  for(int k=1;k<=nbpol;++k){d2p(k)=gp_Vec(d2pole(k,t));d2w(k)=0.0;}
  for(int j=1;j<=nb2d;++j)d2p2(j)=gp_Vec2d(0.0,2.0);
  return d2mode==1;
 }
 int Nb2dCurves() const override{return nb2d;}
 void SectionShape(int& nbpoles,int& nbknots,int& degree) const override{nbpoles=nbpol;nbknots=2;degree=nbpol-1;}
 void Knots(NCollection_Array1<double>& k) const override{k(k.Lower())=0.0;k(k.Upper())=1.0;}
 void Mults(NCollection_Array1<int>& m) const override{m(m.Lower())=nbpol;m(m.Upper())=nbpol;}
 bool IsRational() const override{return rational;}
 int NbIntervals(const GeomAbs_Shape s) const override{return s>=GeomAbs_C3?nbc3:nbc2;}
 void Intervals(NCollection_Array1<double>& t,const GeomAbs_Shape s) const override{
  const int n=NbIntervals(s);for(int i=1;i<=n+1;++i)t(i)=pfirst+(plast-pfirst)*(i-1)/n;
 }
 void SetInterval(const double f,const double l) override{first=f;last=l;++nbinterval;}
 void Resolution(const int index,const double tol,double& tu,double& tv) const override{
  if(!resolution){Approx_SweepFunction::Resolution(index,tol,tu,tv);return;}
  tu=tol*index*0.75;tv=tol;
 }
 void GetTolerance(const double,const double surf,const double,NCollection_Array1<double>& t) const override{
  for(int i=t.Lower();i<=t.Upper();++i)t(i)=surf/(1.0+0.25*(i-1));
 }
 void SetTolerance(const double t3,const double t2) override{tol3d=t3;tol2d=t2;}
 gp_Pnt BarycentreOfSurf() const override{
  if(!rational)return Approx_SweepFunction::BarycentreOfSurf();
  return gp_Pnt(0.5,0.25,0.1);
 }
 double MaximalSection() const override{
  if(!rational)return Approx_SweepFunction::MaximalSection();
  return 2.0;
 }
 void GetMinimalWeight(NCollection_Array1<double>& w) const override{
  if(!rational){Approx_SweepFunction::GetMinimalWeight(w);return;}
  for(int k=w.Lower();k<=w.Upper();++k)w(k)=std::min(weight(k,pfirst),weight(k,plast));
 }
};
void gb_axs_results(Approx_SweepApproximation& a,const GbAxSweep& f,int dim){
 gb_flag(a.IsDone());
 if(!a.IsDone()){GB_TRY(gb_int(a.UDegree()));return;}
 int ud,vd,nup,nvp,nuk,nvk;a.SurfShape(ud,vd,nup,nvp,nuk,nvk);gb_int(ud);gb_int(vd);gb_int(nup);gb_int(nvp);gb_int(nuk);gb_int(nvk);
 const NCollection_Array2<gp_Pnt>& p=a.SurfPoles();const NCollection_Array2<double>& w=a.SurfWeights();
 for(int i=p.LowerRow();i<=p.UpperRow();++i)for(int j=p.LowerCol();j<=p.UpperCol();++j){gb_pnt(p(i,j));gb_real(w(i,j));}
 for(double k:a.SurfUKnots())gb_real(k);for(double k:a.SurfVKnots())gb_real(k);
 for(int m:a.SurfUMults())gb_int(m);for(int m:a.SurfVMults())gb_int(m);
 gb_real(a.MaxErrorOnSurf());gb_real(a.AverageErrorOnSurf());gb_int(a.NbCurves2d());
 GB_TRY(int d,n,k;a.Curves2dShape(d,n,k);gb_int(d);gb_int(n);gb_int(k));
 for(int j=1;j<=a.NbCurves2d();++j){
  GB_TRY(for(const gp_Pnt2d& q:a.Curve2dPoles(j))gb_pnt2d(q));
  GB_TRY(gb_real(a.Max2dError(j)));GB_TRY(gb_real(a.Average2dError(j)));GB_TRY(gb_real(a.TolCurveOnSurf(j)));
 }
 GB_TRY(gb_int(a.Curves2dDegree()));GB_TRY(for(double k:a.Curves2dKnots())gb_real(k));GB_TRY(for(int m:a.Curves2dMults())gb_int(m));
 gb_int(f.nbinterval);gb_real(f.first);gb_real(f.last);gb_real(f.tol3d);gb_real(f.tol2d);(void)dim;
}
// `axs_sweep nbpol nb2d rational d1mode d2mode resolution nbc2 nbc3 pfirst plast first
// last tol3d boundtol tol2d tolang continuity degmax segmax count (param order ef el)...`:
// Perform, the results, then Eval at each (param, order, ef, el) with its code and
// values.
void gb_axs_sweep_case(std::istream& in){
 GbReader r{in};occ::handle<GbAxSweep> f=new GbAxSweep();
 f->nbpol=r.integer();f->nb2d=r.integer();f->rational=r.flag();f->d1mode=r.integer();f->d2mode=r.integer();f->resolution=r.flag();
 f->nbc2=r.integer();f->nbc3=r.integer();f->pfirst=r.real();f->plast=r.real();
 const double first=r.real(),last=r.real(),t3=r.real(),tb=r.real(),t2=r.real(),ta=r.real();const GeomAbs_Shape cont=GeomAbs_Shape(r.integer());
 const int degmax=r.integer(),segmax=r.integer(),count=r.integer();
 GB_TRY(Approx_SweepApproximation a(f);a.Perform(first,last,t3,tb,t2,ta,cont,degmax,segmax);
  const int dim=(f->rational?f->nbpol:0)+2*f->nb2d+3*f->nbpol;gb_axs_results(a,*f,dim);
  for(int k=0;k<count;++k){const double u=r.real();const int order=r.integer();const double ef=r.real(),el=r.real();
   std::vector<double> res(dim,-7.);GB_TRY(gb_int(a.Eval(u,order,ef,el,res[0]));for(double v:res)gb_real(v));});
 gb_end();
}
