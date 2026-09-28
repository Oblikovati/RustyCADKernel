// Multiple roots and null intervals at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathRoot_Multiple.hxx>
#include <MathRoot_All.hxx>
struct ModernMultipleFunction{
 int mode,fail;double target;std::vector<std::pair<int,double>> calls;
 std::pair<double,double> Formula(double x){
  if(mode==1){double a=x-1.003,b=x-1.007;return {a*a*b*b,2.0*a*b*(a+b)};}
  if(mode==2)return {std::sin(x),std::cos(x)};
  if(mode==3)return {0.0,0.0};
  if(mode==4){if(std::abs(x)<std::abs(target))return {0.0,0.0};return {std::abs(x)-std::abs(target),x>0.0?1.0:-1.0};}
  if(mode==5)return {std::numeric_limits<double>::quiet_NaN(),0.0};
  if(mode==6)return {x*x-target,std::numeric_limits<double>::infinity()};
  if(mode==9)return {-(x-target)*(x-target),-2.0*(x-target)};
  return {x*x-target,2.0*x};
 }
 bool Evaluate(int event,double x,double& value,double& derivative){
  calls.push_back({event,x});if(int(calls.size())==fail)return false;
  auto pair=Formula(x);if(mode!=7||calls.size()%3!=0)value=pair.first;derivative=pair.second;return true;
 }
 bool Value(double x,double& value){double derivative=0;return Evaluate(0,x,value,derivative);}
 bool Values(double x,double& value,double& derivative){return Evaluate(1,x,value,derivative);}
};
void multiple_trace(ModernMultipleFunction& f){std::cout<<f.calls.size()<<' ';for(auto c:f.calls)std::cout<<c.first<<' '<<bits(c.second)<<' ';f.calls.clear();}
void multiple_result(const MathRoot::MultipleResult& r,ModernMultipleFunction& f){
 std::cout<<int(r.Status)<<' '<<r.NbIterations<<' '<<r.IsAllNull<<' '<<r.NbRoots()<<' ';
 for(size_t i=0;i<r.NbRoots();++i)std::cout<<bits(r.Roots.Value(i))<<' '<<bits(r.Values.Value(i))<<' ';multiple_trace(f);
}
void all_result(const MathRoot::AllRootsResult& r,ModernMultipleFunction& f){
 std::cout<<int(r.Status)<<' '<<r.NbRoots()<<' '<<r.NbIntervals()<<' ';
 for(size_t i=0;i<r.NbRoots();++i)std::cout<<bits(r.Roots.Value(i))<<' '<<r.RootStates.Value(i)<<' ';
 for(size_t i=0;i<r.NbIntervals();++i){const auto& n=r.NullIntervals.Value(i);std::cout<<bits(n.A)<<' '<<bits(n.B)<<' '<<n.State<<' ';}multiple_trace(f);
}
void modern_multiple_case(std::istream& input){
 int mode,fail,limit;size_t n;double target,lower,upper,xtol,ftol,ntol,offset;input>>mode>>target>>lower>>upper>>n>>xtol>>ftol>>ntol>>limit>>fail>>offset;
 MathRoot::MultipleConfig cfg;cfg.NbSamples=n;cfg.XTolerance=xtol;cfg.FTolerance=ftol;cfg.NullTolerance=ntol;cfg.MaxIterations=limit;cfg.Offset=offset;
 ModernMultipleFunction f{mode,fail,target,{}};
 multiple_result(MathRoot::FindAllRoots(f,lower,upper,cfg),f);
 multiple_result(MathRoot::FindAllRootsWithDerivative(f,lower,upper,cfg),f);
 all_result(MathRoot::FindAllRootsWithIntervals(f,lower,upper,n,xtol,ftol,ntol),f);
 math_Vector samples(-2,2);for(size_t i=0;i<5;++i){double ratio=double(i)/4.0;samples.ChangeAt(i)=(1.0-ratio)*lower+ratio*upper;}
 if(mode==10)samples.ChangeAt(2)=samples.At(1);
 all_result(MathRoot::FindAllRootsWithIntervals(f,samples,xtol,ftol,ntol),f);std::cout<<std::endl;
}
