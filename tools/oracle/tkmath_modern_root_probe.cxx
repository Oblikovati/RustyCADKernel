// Modern scalar root oracle at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathRoot_Newton.hxx>
#include <MathRoot_Brent.hxx>
#include <MathRoot_Bisection.hxx>
#include <MathRoot_Secant.hxx>
struct ModernRootFunction {
  int mode,fail;double target;std::vector<std::pair<int,double>> calls;
  void Formula(double x,double& f,double& df) {
    if(mode==0) {f=x*x-target;df=2*x;}
    else if(mode==1) {f=std::cos(x)-x;df=-std::sin(x)-1;}
    else if(mode==2) {f=1;df=0;}
    else if(mode==3) {f=x-target;df=1e300;}
    else {f=x-target;df=1;}
    if(mode==5)f=std::numeric_limits<double>::quiet_NaN();
    if(mode==6)df=std::numeric_limits<double>::infinity();
  }
  bool Value(double x,double& f) {
    calls.push_back({0,x});if(int(calls.size())==fail)return false;
    double df=0;Formula(x,f,df);return true;
  }
  bool Values(double x,double& f,double& df) {
    calls.push_back({1,x});if(int(calls.size())==fail)return false;
    Formula(x,f,df);return true;
  }
};
void modern_root_result(const MathUtils::ScalarResult& r,const ModernRootFunction& f) {
  std::cout<<int(r.Status)<<' '<<r.NbIterations<<' ';
  for(const auto& value:{r.Root,r.Value,r.Derivative})std::cout<<value.has_value()<<' '<<bits(value.value_or(91))<<' ';
  std::cout<<f.calls.size()<<' ';for(const auto& c:f.calls)std::cout<<c.first<<' '<<bits(c.second)<<' ';
}
void modern_root_case(std::istream& input) {
  int mode,fail,limit;double target,guess,lower,upper,xtol,ftol;
  input>>mode>>target>>guess>>lower>>upper>>xtol>>ftol>>limit>>fail;
  MathUtils::Config config;config.XTolerance=xtol;config.FTolerance=ftol;config.MaxIterations=limit;
  ModernRootFunction f{mode,fail,target,{}};
  modern_root_result(MathRoot::Newton(f,guess,config),f);f.calls.clear();
  modern_root_result(MathRoot::NewtonBounded(f,guess,lower,upper,config),f);f.calls.clear();
  modern_root_result(MathRoot::Brent(f,lower,upper,config),f);f.calls.clear();
  modern_root_result(MathRoot::Bisection(f,lower,upper,config),f);f.calls.clear();
  modern_root_result(MathRoot::BisectionNewton(f,lower,upper,config),f);f.calls.clear();
  modern_root_result(MathRoot::Secant(f,lower,upper,config),f);f.calls.clear();
  modern_root_result(MathRoot::SecantAuto(f,guess,config),f);std::cout<<'\n';
}
