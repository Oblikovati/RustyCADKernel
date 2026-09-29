// FairCurve probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/fair_curve.rs.
#include <FairCurve_Batten.hxx>
#include <FairCurve_BattenLaw.hxx>
#include <FairCurve_DistributionOfJerk.hxx>
#include <FairCurve_DistributionOfSagging.hxx>
#include <FairCurve_DistributionOfTension.hxx>
#include <FairCurve_EnergyOfBatten.hxx>
#include <FairCurve_EnergyOfMVC.hxx>
#include <FairCurve_MinimalVariation.hxx>
#include <FairCurve_Newton.hxx>
#include <BSplCLib.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <math_Matrix.hxx>
#include <math_Vector.hxx>
#include <memory>
#include <sstream>
// `h slope sliding h2 slope2 sliding2 n t*n`: the law before and after its setters.
void ga_fc_law_case(std::istream& input){
 GaReader r{input};double h=r.real(),s=r.real(),l=r.real(),h2=r.real(),s2=r.real(),l2=r.real();std::vector<double> ts=r.reals();
 FairCurve_BattenLaw law(h,s,l);
 for(double t:ts){double v=0;ga_flag(law.Value(t,v));ga_real(v);}
 law.SetHeigth(h2);law.SetSlope(s2);law.SetSliding(l2);
 for(double t:ts){double v=0;ga_flag(law.Value(t,v));ga_real(v);}
 ga_end();
}
// The spline of a request: `nk knots*nk mults*nk np (x y)*np`, its flat knots and order.
struct GaFcSpline{occ::handle<NCollection_HArray1<double>> flat;occ::handle<NCollection_HArray1<gp_Pnt2d>> poles;int order;};
GaFcSpline ga_fc_spline(GaReader& r){
 int nk=r.integer();NCollection_Array1<double> knots(1,nk);NCollection_Array1<int> mults(1,nk);
 for(int i=1;i<=nk;i++)knots(i)=r.real();
 int total=0;for(int i=1;i<=nk;i++){mults(i)=r.integer();total+=mults(i);}
 int np=r.integer();GaFcSpline s;s.poles=new NCollection_HArray1<gp_Pnt2d>(1,np);
 for(int i=1;i<=np;i++)s.poles->ChangeValue(i)=r.pnt2d();
 s.order=total-np;s.flat=new NCollection_HArray1<double>(1,total);
 BSplCLib::KnotSequence(knots,mults,s.order-1,false,s.flat->ChangeArray1());
 return s;
}
void ga_fc_poles(const occ::handle<NCollection_HArray1<gp_Pnt2d>>& p){for(int i=p->Lower();i<=p->Upper();i++)ga_pnt2d(p->Value(i));}
void ga_fc_vector(const math_Vector& v){for(int i=v.Lower();i<=v.Upper();i++)ga_real(v(i));}
// `kind spline derivative aux h slope sliding lsliding uniform d2 n t*n`: a density
// (0 tension, 1 sagging, 2 jerk) at every t, then at the first t after SetDerivativeOrder(d2).
void ga_fc_density_case(std::istream& input){
 GaReader r{input};int kind=r.integer();GaFcSpline s=ga_fc_spline(r);
 int d=r.integer(),aux=r.integer();double h=r.real(),slope=r.real(),sl=r.real(),ls=r.real();bool uniform=r.flag();int d2=r.integer();
 std::vector<double> ts=r.reals();FairCurve_BattenLaw law(h,slope,sl);
 std::unique_ptr<FairCurve_DistributionOfEnergy> f;
 if(kind==0)f.reset(new FairCurve_DistributionOfTension(s.order,s.flat,s.poles,d,ls,law,aux,uniform));
 else if(kind==1)f.reset(new FairCurve_DistributionOfSagging(s.order,s.flat,s.poles,d,law,aux));
 else f.reset(new FairCurve_DistributionOfJerk(s.order,s.flat,s.poles,d,law,aux));
 ga_int(f->NbVariables());ga_int(f->NbEquations());
 math_Vector t(1,1);
 for(double x:ts){t(1)=x;math_Vector v(1,f->NbEquations(),0.0);ga_flag(f->Value(t,v));ga_fc_vector(v);}
 f->SetDerivativeOrder(d2);ga_int(f->NbEquations());
 if(!ts.empty()){t(1)=ts[0];math_Vector v(1,f->NbEquations(),0.0);ga_flag(f->Value(t,v));ga_fc_vector(v);}
 ga_end();
}
// Value, Values and the Hessian of an energy at X.
void ga_fc_evaluate(math_MultipleVarFunctionWithHessian& e,const math_Vector& x){
 int n=e.NbVariables();double v=0;ga_flag(e.Value(x,v));ga_real(v);
 math_Vector g(1,n,0.0);ga_flag(e.Values(x,v,g));ga_real(v);ga_fc_vector(g);
 math_Matrix hm(1,n,1,n,0.0);math_Vector g2(1,n,0.0);ga_flag(e.Values(x,v,g2,hm));ga_real(v);ga_fc_vector(g2);
 for(int i=1;i<=n;i++)for(int j=1;j<=n;j++)ga_real(hm(i,j));
}
template<class E> void ga_fc_energy_report(E& e,const GaFcSpline& s,double pert,bool newton,const double nt[4]){
 int n=e.NbVariables();ga_int(n);math_Vector x(1,n,0.0);ga_flag(e.Variable(x));ga_fc_vector(x);
 ga_fc_evaluate(e,x);ga_int(int(e.Status()));ga_real(e.LengthSliding());
 math_Vector x2(1,n);for(int k=1;k<=n;k++)x2(k)=x(k)*(1+pert*k);
 ga_fc_evaluate(e,x2);ga_int(int(e.Status()));ga_real(e.LengthSliding());ga_fc_poles(s.poles);
 if(!newton)return;
 FairCurve_Newton solver(e,nt[0],nt[1],int(nt[2]),nt[3]);solver.Perform(e,x);
 ga_flag(solver.IsDone());ga_flag(solver.IsConverged());
 if(solver.IsDone()){math_Vector loc(1,n);solver.Location(loc);ga_fc_vector(loc);ga_real(solver.Minimum());}
 ga_int(int(e.Status()));ga_real(e.LengthSliding());ga_fc_poles(s.poles);
}
// `kind spline o1 o2 free a1 a2 c1 c2 h slope sliding lsliding ratio pert newton
// sptol tol nbiter convexity`: an energy of batten (kind 0) or of MVC (kind 1).
void ga_fc_energy_case(std::istream& input){
 GaReader r{input};int kind=r.integer();GaFcSpline s=ga_fc_spline(r);
 int o1=r.integer(),o2=r.integer();bool free=r.flag();double a1=r.real(),a2=r.real(),c1=r.real(),c2=r.real();
 double h=r.real(),slope=r.real(),sl=r.real(),ls=r.real(),ratio=r.real(),pert=r.real();bool newton=r.flag();
 double nt[4];for(double& x:nt)x=r.real();FairCurve_BattenLaw law(h,slope,sl);
 GA_TRY(if(kind==0){FairCurve_EnergyOfBatten e(s.order,s.flat,s.poles,o1,o2,law,ls,free,a1,a2);ga_fc_energy_report(e,s,pert,newton,nt);}
  else{FairCurve_EnergyOfMVC e(s.order,s.flat,s.poles,o1,o2,law,ratio,ls,free,a1,a2,c1,c2);ga_fc_energy_report(e,s,pert,newton,nt);});
 ga_end();
}
// The state of a batten (and of a minimal-variation curve when `mvc`), its curve and Dump.
void ga_fc_batten_report(const FairCurve_Batten& b,const FairCurve_MinimalVariation* mvc){
 ga_pnt2d(b.GetP1());ga_pnt2d(b.GetP2());ga_real(b.GetAngle1());ga_real(b.GetAngle2());ga_real(b.GetHeight());
 ga_real(b.GetSlope());ga_real(b.GetSlidingFactor());ga_flag(b.GetFreeSliding());ga_int(b.GetConstraintOrder1());
 ga_int(b.GetConstraintOrder2());
 if(mvc){ga_real(mvc->GetCurvature1());ga_real(mvc->GetCurvature2());ga_real(mvc->GetPhysicalRatio());}
 ga_real(b.SlidingOfReference());
 GA_TRY(occ::handle<Geom2d_BSplineCurve> c=b.Curve();ga_int(c->Degree());ga_int(c->NbPoles());
  for(int i=1;i<=c->NbPoles();i++)ga_pnt2d(c->Pole(i));ga_int(c->NbKnots());
  for(int i=1;i<=c->NbKnots();i++){ga_real(c->Knot(i));ga_int(c->Multiplicity(i));});
 std::ostringstream o;b.Dump(o);std::string text=o.str();
 for(char& ch:text){if(ch=='\n')ch='|';if(ch==' ')ch='_';}std::cout<<'"'<<text<<"\" ";
}
// `kind p1 p2 height slope ratio free o1 o2 a1 a2 factor c1 c2 height2 nbiter tol
// moves (p1 p2)*moves`: a batten (kind 0) or minimal-variation curve (kind 1) computed
// once, then again after each move of its end points.
void ga_fc_batten_case(std::istream& input){
 GaReader r{input};int kind=r.integer();gp_Pnt2d p1=r.pnt2d(),p2=r.pnt2d();double h=r.real(),slope=r.real(),ratio=r.real();
 bool free=r.flag();int o1=r.integer(),o2=r.integer();double a1=r.real(),a2=r.real(),factor=r.real(),c1=r.real(),c2=r.real();
 double h2=r.real();int nbiter=r.integer();double tol=r.real();int moves=r.integer();
 std::unique_ptr<FairCurve_Batten> b;FairCurve_MinimalVariation* mvc=nullptr;
 GA_TRY(if(kind==0)b.reset(new FairCurve_Batten(p1,p2,h,slope));else{mvc=new FairCurve_MinimalVariation(p1,p2,h,slope,ratio);b.reset(mvc);});
 if(!b){ga_end();return;}
 b->SetFreeSliding(free);b->SetConstraintOrder1(o1);b->SetConstraintOrder2(o2);b->SetAngle1(a1);b->SetAngle2(a2);
 b->SetSlidingFactor(factor);
 if(mvc){mvc->SetCurvature1(c1);mvc->SetCurvature2(c2);GA_TRY(mvc->SetPhysicalRatio(ratio));}
 GA_TRY(b->SetHeight(h2));
 for(int m=0;m<=moves;m++){
  if(m>0){gp_Pnt2d q1=r.pnt2d(),q2=r.pnt2d();GA_TRY(b->SetP1(q1));GA_TRY(b->SetP2(q2));}
  FairCurve_AnalysisCode code=FairCurve_OK;
  GA_TRY(bool ok=b->Compute(code,nbiter,tol);ga_flag(ok);ga_int(int(code)));
  ga_fc_batten_report(*b,mvc);
 }
 ga_end();
}
