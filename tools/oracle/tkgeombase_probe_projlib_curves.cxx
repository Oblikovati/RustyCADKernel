// ProjLib approximations (tools/tkgeombase_corpus_projlib.py): ProjLib_ComputeApprox,
// ProjLib_ComputeApproxOnPolarSurface, ProjLib_ProjectedCurve and
// ProjLib_CompProjectedCurve on the GeomTools builders (gt_curve, gt_curve2d,
// gt_surface) through the adaptors of tkgeombase_probe_approx_adaptors.cxx.

#include <ProjLib_CompProjectedCurve.hxx>
#include <ProjLib_ComputeApprox.hxx>
#include <ProjLib_ComputeApproxOnPolarSurface.hxx>
#include <ProjLib_ProjectedCurve.hxx>
// The shared options: `tol degmin degmax maxseg bndpnt maxdist`.
struct GbPlOptions{double tol;int degmin,degmax,maxseg;AppParCurves_Constraint bnd;double maxdist;};
GbPlOptions gb_pl_options(GbReader& r){
 GbPlOptions o;o.tol=r.real();o.degmin=r.integer();o.degmax=r.integer();o.maxseg=r.integer();
 o.bnd=AppParCurves_Constraint(r.integer());o.maxdist=r.real();return o;
}
// `pl_approx variant options curve surface`: variant 0 the empty constructor, the
// setters and Perform; 1 the performing constructor (C, S, Tol).
void gb_pl_approx_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const GbPlOptions o=gb_pl_options(r);
 GB_TRY(occ::handle<Adaptor3d_Curve> c=gb_axa_curve(r);occ::handle<Adaptor3d_Surface> s=gb_axa_surface(r);
  ProjLib_ComputeApprox a;
  if(variant==1)a=ProjLib_ComputeApprox(c,s,o.tol);
  else{a.SetTolerance(o.tol);a.SetDegree(o.degmin,o.degmax);a.SetMaxSegments(o.maxseg);a.SetBndPnt(o.bnd);a.Perform(c,s);}
  gb_g2c_bspline(a.BSpline());gb_g2c_bezier(a.Bezier());gb_real(a.Tolerance()));
 gb_end();
}
void gb_pl_polar_results(const ProjLib_ComputeApproxOnPolarSurface& p){
 gb_flag(p.IsDone());gb_g2c_bspline(p.BSpline());GB_TRY(gb_gcv_curve2d(p.Curve2d()));gb_real(p.Tolerance());
}
occ::handle<Adaptor2d_Curve2d> gb_pl_init2d(GbReader& r){return gb_axa_curve2d(r);}
void gb_pl_polar_setup(ProjLib_ComputeApproxOnPolarSurface& p,const GbPlOptions& o){
 p.SetTolerance(o.tol);p.SetDegree(o.degmin,o.degmax);p.SetMaxSegments(o.maxseg);p.SetBndPnt(o.bnd);p.SetMaxDist(o.maxdist);
}
// `pl_polar variant options curve surface [init2d [init2d_bis]]`: 0 the setters and
// Perform(C, S); 1 the constructor (C, S, Tol); 2 BuildInitialCurve2d then
// ProjectUsingInitialCurve2d; 3 the constructor (Init, C, S, Tol); 4 the constructor
// (Init, InitBis, C, S, Tol); 5 the setters and Perform(Init, C, S).
void gb_pl_polar_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const GbPlOptions o=gb_pl_options(r);
 GB_TRY(occ::handle<Adaptor3d_Curve> c=gb_axa_curve(r);occ::handle<Adaptor3d_Surface> s=gb_axa_surface(r);
  if(variant==0){ProjLib_ComputeApproxOnPolarSurface p;gb_pl_polar_setup(p,o);p.Perform(c,s);gb_pl_polar_results(p);}
  else if(variant==1){ProjLib_ComputeApproxOnPolarSurface p(c,s,o.tol);gb_pl_polar_results(p);}
  else if(variant==2){ProjLib_ComputeApproxOnPolarSurface p;gb_pl_polar_setup(p,o);
   occ::handle<Adaptor2d_Curve2d> init=p.BuildInitialCurve2d(c,s);gb_flag(init.IsNull());gb_pl_polar_results(p);
   if(!init.IsNull()){gb_g2c_bspline(p.ProjectUsingInitialCurve2d(c,s,init));gb_pl_polar_results(p);}}
  else if(variant==3){occ::handle<Adaptor2d_Curve2d> init=gb_pl_init2d(r);ProjLib_ComputeApproxOnPolarSurface p(init,c,s,o.tol);gb_pl_polar_results(p);}
  else if(variant==4){occ::handle<Adaptor2d_Curve2d> init=gb_pl_init2d(r),bis=gb_pl_init2d(r);
   ProjLib_ComputeApproxOnPolarSurface p(init,bis,c,s,o.tol);gb_pl_polar_results(p);}
  else{occ::handle<Adaptor2d_Curve2d> init=gb_pl_init2d(r);ProjLib_ComputeApproxOnPolarSurface p;gb_pl_polar_setup(p,o);
   gb_g2c_bspline(p.Perform(init,c,s));gb_pl_polar_results(p);});
 gb_end();
}
// The result of a ProjectedCurve: kind, tolerance, periodicity, bounds, typed
// results, spline data, MakePCurveOfType and the members that raise natively.
void gb_pl_projected_results(const occ::handle<ProjLib_ProjectedCurve>& p){
 gb_int(p->GetType());gb_real(p->GetTolerance());gb_flag(p->IsPeriodic());
 GB_TRY(gb_real(p->FirstParameter()));GB_TRY(gb_real(p->LastParameter()));
 GB_TRY(gb_lin2d(p->Line()));GB_TRY(gb_circ2d(p->Circle()));GB_TRY(gb_elips2d(p->Ellipse()));
 GB_TRY(gb_hypr2d(p->Hyperbola()));GB_TRY(gb_parab2d(p->Parabola()));
 // A general surface without a projected part leaves the initial BSplineCurve type
 // with a null curve, which the spline accessors dereference natively.
 const bool nullSpline=(p->GetType()==GeomAbs_BSplineCurve&&p->BSpline().IsNull())||(p->GetType()==GeomAbs_BezierCurve&&p->Bezier().IsNull());
 gb_flag(nullSpline);
 if(!nullSpline){GB_TRY(gb_int(p->Degree()));GB_TRY(gb_flag(p->IsRational()));GB_TRY(gb_int(p->NbPoles()));GB_TRY(gb_int(p->NbKnots()));}
 gb_g2c_bspline(p->BSpline());gb_g2c_bezier(p->Bezier());
 GB_TRY(occ::handle<Geom2d_Curve> pc;ProjLib::MakePCurveOfType(*p,pc);gb_gcv_curve2d(pc));
 GB_TRY(gb_int(p->Continuity()));GB_TRY(gb_int(p->NbIntervals(GeomAbs_C1)));GB_TRY(gb_flag(p->IsClosed()));
 GB_TRY(gb_real(p->Period()));GB_TRY(gb_real(p->Resolution(1.0e-3)));GB_TRY(p->Trim(0.,1.,1.0e-7));
 GB_TRY(gb_pnt2d(p->Value(p->FirstParameter())));
 gb_flag(p->GetCurve().IsNull());if(!p->GetCurve().IsNull()){gb_real(p->GetCurve()->FirstParameter());gb_real(p->GetCurve()->LastParameter());}
 GB_TRY(occ::handle<Adaptor2d_Curve2d> c=p->ShallowCopy();gb_int(c->GetType()));
}
// `pl_projected variant options curve surface`: 0 the empty constructor, Load(S),
// Load(Tol), the setters and Perform; 1 the constructor (S, C); 2 (S, C, Tol).
void gb_pl_projected_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const GbPlOptions o=gb_pl_options(r);
 GB_TRY(occ::handle<Adaptor3d_Curve> c=gb_axa_curve(r);occ::handle<Adaptor3d_Surface> s=gb_axa_surface(r);
  occ::handle<ProjLib_ProjectedCurve> p;
  if(variant==0){p=new ProjLib_ProjectedCurve();p->Load(s);p->Load(o.tol);p->SetDegree(o.degmin,o.degmax);
   p->SetMaxSegments(o.maxseg);p->SetBndPnt(o.bnd);p->SetMaxDist(o.maxdist);p->Perform(c);}
  else if(variant==1)p=new ProjLib_ProjectedCurve(s,c);
  else p=new ProjLib_ProjectedCurve(s,c,o.tol);
  gb_pl_projected_results(p));
 gb_end();
}
// The parts of a CompProjectedCurve: bounds, flags, distance and triples.
void gb_pl_comp_parts(const ProjLib_CompProjectedCurve& p){
 gb_int(p.NbCurves());
 for(int i=1;i<=p.NbCurves();++i){
  double f,l,u,v;gp_Pnt2d pt;p.Bounds(i,f,l);gb_real(f);gb_real(l);
  gb_flag(p.IsSinglePnt(i,pt));gb_pnt2d(pt);gb_flag(p.IsUIso(i,u));gb_real(u);gb_flag(p.IsVIso(i,v));gb_real(v);
  gb_real(p.MaxDistance(i));
  const auto& seq=p.GetSequence()->Value(i);gb_int(seq->Length());for(int j=1;j<=seq->Length();++j)gb_pnt(seq->Value(j));
 }
}
// Evaluations at five parameters of the first part.
void gb_pl_comp_eval(const ProjLib_CompProjectedCurve& p){
 if(p.NbCurves()<1)return;
 double f,l;p.Bounds(1,f,l);
 for(int i=0;i<=4;++i){const double u=f+(l-f)*i/4.;
  GB_TRY(gb_pnt2d(p.Value(u)));GB_TRY(gp_Pnt2d q;gp_Vec2d v;p.D1(u,q,v);gb_pnt2d(q);gb_vec2d(v));
  GB_TRY(gp_Pnt2d q;gp_Vec2d v1,v2;p.D2(u,q,v1,v2);gb_pnt2d(q);gb_vec2d(v1);gb_vec2d(v2));
  GB_TRY(gb_vec2d(p.DN(u,1)));GB_TRY(gb_vec2d(p.DN(u,3)));GB_TRY(gb_vec2d(p.DN(u,0)));}
}
void gb_pl_comp_adaptor(const occ::handle<ProjLib_CompProjectedCurve>& p){
 GB_TRY(gb_real(p->FirstParameter()));GB_TRY(gb_real(p->LastParameter()));GB_TRY(gb_int(p->Continuity()));gb_int(p->GetType());
 double tu,tv;p->GetTolerance(tu,tv);gb_real(tu);gb_real(tv);gb_flag(p->GetProj2d());gb_flag(p->GetProj3d());
 for(int s=0;s<=6;++s){GB_TRY(const int n=p->NbIntervals(GeomAbs_Shape(s));gb_int(n);NCollection_Array1<double> t(1,n+1);
  p->Intervals(t,GeomAbs_Shape(s));for(double v:t)gb_real(v));}
 GB_TRY(occ::handle<Adaptor2d_Curve2d> c=p->ShallowCopy();gb_int(c->GetType());gb_pnt2d(c->Value(0.5*(c->FirstParameter()+c->LastParameter()))));
 GB_TRY(double f,l;p->Bounds(1,f,l);occ::handle<Adaptor2d_Curve2d> t=p->Trim(f,0.5*(f+l),1.0e-7);gb_real(t->FirstParameter());gb_real(t->LastParameter()));
 GB_TRY(p->Line());GB_TRY(p->Resolution(1.0e-3));GB_TRY(gb_flag(p->IsPeriodic()));
}
// The results of Perform for each part.
void gb_pl_comp_results(const ProjLib_CompProjectedCurve& p){
 for(int i=1;i<=p.NbCurves();++i){
  gb_flag(p.ResultIsPoint(i));gb_real(p.GetResult2dUApproxError(i));gb_real(p.GetResult2dVApproxError(i));gb_real(p.GetResult3dApproxError(i));
  if(p.GetProj2d()){gb_gcv_curve2d(p.GetResult2dC(i));GB_TRY(gb_pnt2d(p.GetResult2dP(i)));}
  if(p.GetProj3d()){gb_gcv_curve(p.GetResult3dC(i));GB_TRY(gb_pnt(p.GetResult3dP(i)));}
 }
}
// `pl_comp variant tol3d tolu tolv maxdist proj2d proj3d continuity maxdeg maxseg
// curve surface`: 0 the constructor (S, C, TolU, TolV), 1 (S, C, TolU, TolV,
// MaxDist), 2 (Tol3d, S, C, MaxDist); then the parts, evaluations, the adaptor
// members, the options, Perform and its results.
void gb_pl_comp_case(std::istream& in){
 GbReader r{in};const int variant=r.integer();const double tol3d=r.real(),tu=r.real(),tv=r.real(),md=r.real();
 const bool p2=r.flag(),p3=r.flag();const GeomAbs_Shape cont=GeomAbs_Shape(r.integer());const int deg=r.integer(),seg=r.integer();
 GB_TRY(occ::handle<Adaptor3d_Curve> c=gb_axa_curve(r);occ::handle<Adaptor3d_Surface> s=gb_axa_surface(r);
  occ::handle<ProjLib_CompProjectedCurve> p=variant==0?new ProjLib_CompProjectedCurve(s,c,tu,tv)
   :variant==1?new ProjLib_CompProjectedCurve(s,c,tu,tv,md):new ProjLib_CompProjectedCurve(tol3d,s,c,md);
  gb_pl_comp_parts(*p);gb_pl_comp_eval(*p);gb_pl_comp_adaptor(p);
  p->SetTol3d(tol3d);p->SetContinuity(cont);p->SetMaxDegree(deg);p->SetMaxSeg(seg);p->SetProj2d(p2);p->SetProj3d(p3);
  GB_TRY(p->Perform();gb_pl_comp_results(*p)));
 gb_end();
}
