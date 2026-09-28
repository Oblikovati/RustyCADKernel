// MathInteg Gauss rules and modern node generator at pinned 3d097a0.
#include <MathInteg_Gauss.hxx>
struct ModernIntegrand {
 int mode,fail;std::vector<double> calls;
 bool Value(double x,double& value){
  calls.push_back(x);if(int(calls.size())==fail)return false;
  if(mode==5&&calls.size()%3==0)return true;
  switch(mode){case 1:value=std::sin(30.0*x);break;case 2:value=1.0/(1.0+x*x);break;case 3:value=std::numeric_limits<double>::quiet_NaN();break;case 4:value=std::numeric_limits<double>::max();break;case 6:value=std::exp(x);break;case 7:value=-0.0;break;case 8:value=std::exp(-x);break;case 9:value=std::exp(-x*x);break;default:value=x*x*x*x*x;}return true;
 }
};
void integ_optional(const std::optional<double>& value){std::cout<<value.has_value()<<' ';if(value)std::cout<<bits(*value)<<' ';}
void integ_result(const MathUtils::IntegResult& r,ModernIntegrand& f){
 std::cout<<int(r.Status)<<' '<<r.NbIterations<<' '<<r.NbPoints<<' ';
 integ_optional(r.Value);integ_optional(r.AbsoluteError);integ_optional(r.RelativeError);
 std::cout<<f.calls.size()<<' ';for(double x:f.calls)std::cout<<bits(x)<<' ';f.calls.clear();
}
void modern_integ_case(std::istream& input){
 int mode,initial,maxorder,limit,fail,n;double lower,upper,tolerance;input>>mode>>lower>>upper>>initial>>maxorder>>limit>>tolerance>>fail>>n;
 ModernIntegrand f{mode,fail,{}};MathUtils::IntegConfig cfg;cfg.InitialOrder=initial;cfg.MaxOrder=maxorder;cfg.MaxIterations=limit;cfg.Tolerance=tolerance;
 integ_result(MathInteg::Gauss(f,lower,upper,initial),f);
 integ_result(MathInteg::GaussComposite(f,lower,upper,limit,initial),f);
 integ_result(MathInteg::GaussAdaptive(f,lower,upper,cfg),f);
 math_Vector points(size_t(std::max(n,1))),weights(size_t(std::max(n,1)));
 bool ok=MathUtils::GetGaussPointsAndWeights(n,points,weights);std::cout<<ok<<' ';
 if(ok)for(size_t i=0;i<points.Size();++i)std::cout<<bits(points.At(i))<<' '<<bits(weights.At(i))<<' ';std::cout<<std::endl;
}

#include <MathInteg_Kronrod.hxx>
void modern_kronrod_case(std::istream& input){
 int mode,order,adaptive,limit,fail,maxorder;double lower,upper,tolerance;input>>mode>>lower>>upper>>order>>adaptive>>limit>>tolerance>>fail>>maxorder;
 ModernIntegrand f{mode,fail,{}};MathInteg::KronrodConfig cfg(tolerance,limit);cfg.NbGaussPoints=order;cfg.Adaptive=bool(adaptive);
 integ_result(MathInteg::KronrodRule(f,lower,upper,order),f);
 integ_result(MathInteg::Kronrod(f,lower,upper,cfg),f);
 integ_result(MathInteg::KronrodAuto(f,lower,upper,tolerance,maxorder),f);
 integ_result(MathInteg::KronrodSemiInfinite(f,lower,cfg),f);
 integ_result(MathInteg::KronrodInfinite(f,cfg),f);std::cout<<std::endl;
}

#include <MathInteg_DoubleExp.hxx>
void modern_double_exp_case(std::istream& input){
 int mode,levels,limit,budget,fail,kind;double lower,upper,step,tolerance,singular;input>>mode>>lower>>upper>>levels>>limit>>step>>tolerance>>budget>>fail>>kind>>singular;
 ModernIntegrand f{mode,fail,{}};MathInteg::DoubleExpConfig cfg(tolerance,limit);cfg.NbLevels=levels;cfg.StepFactor=step;cfg.MaxPointsPerLevel=budget;
 integ_result(MathInteg::TanhSinh(f,lower,upper,cfg),f);
 integ_result(MathInteg::ExpSinh(f,lower,cfg),f);
 integ_result(MathInteg::SinhSinh(f,cfg),f);
 double a=lower,b=upper;const double inf=std::numeric_limits<double>::infinity();
 if(kind==1)b=inf;if(kind==2)a=-inf;if(kind==3){a=-inf;b=inf;}if(kind==4)a=inf;if(kind==5)a=std::numeric_limits<double>::quiet_NaN();
 integ_result(MathInteg::DoubleExponential(f,a,b,cfg),f);
 integ_result(MathInteg::TanhSinhWithSingularity(f,lower,upper,singular,cfg),f);std::cout<<std::endl;
}
