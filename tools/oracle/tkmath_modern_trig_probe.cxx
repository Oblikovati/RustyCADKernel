// Modern trigonometric oracle at pinned 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathRoot_Trig.hxx>
void modern_trig_result(const MathRoot::TrigResult& r){
 std::cout<<int(r.Status)<<' '<<r.NbRoots<<' '<<r.InfiniteRoots<<' ';for(double x:r.Roots)std::cout<<bits(x)<<' ';
}
void modern_trig_case(std::istream& input){
 double c[5],lower,upper,eps,period,tolerance;int mode;
 for(double& x:c)input>>x;input>>lower>>upper>>eps>>mode>>period>>tolerance;
 if(mode==1)c[0]=std::numeric_limits<double>::quiet_NaN();
 if(mode==2)upper=std::numeric_limits<double>::infinity();
 if(mode==3)lower=std::numeric_limits<double>::quiet_NaN();
 if(mode==4)eps=std::numeric_limits<double>::infinity();
 modern_trig_result(MathRoot::Trigonometric(c[0],c[1],c[2],c[3],c[4],lower,upper,eps));
 modern_trig_result(MathRoot::TrigonometricLinear(c[3],c[4],lower,upper));
 modern_trig_result(MathRoot::TrigonometricCDE(c[2],c[3],c[4],lower,upper));
 for(bool exclude:{false,true}){size_t count=99;bool ok=MathRoot::Utils::ComputePeriodicRootCount(lower,upper,period,tolerance,exclude,count);std::cout<<ok<<' '<<count<<' ';}
 std::cout<<std::endl;
}
