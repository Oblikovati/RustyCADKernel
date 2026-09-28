// Sampled roots: callback order, states, shifted extrema, sampling failures and guards.
#include <math_FunctionRoots.hxx>
struct SampledFunction:math_FunctionWithDerivative {
 int mode,fail,calls=0,states=0;double center,offset,scale;
 SampledFunction(int m,int f,double c,double o,double s):mode(m),fail(f),center(c),offset(o),scale(s){}
 double val(double x){double t=x-center;if(mode==0)return scale*t+offset;if(mode==1)return scale*t*t+offset;if(mode==2)return scale*t*t*t+offset;if(mode==3)return scale*std::sin(t)+offset;if(mode==5)return scale*(t < -1 ? t+1 : t > 1 ? t-1 : 0)+offset;if(mode==6)return scale*(t < -2 ? t+2 : t > 2 ? t-2 : std::abs(t)<1 ? t*(1-t*t) : 0)+offset;return offset;}
 double der(double x){double t=x-center;if(mode==0)return scale;if(mode==1)return 2*scale*t;if(mode==2)return 3*scale*t*t;if(mode==3)return scale*std::cos(t);if(mode==5)return std::abs(t)>1?scale:0;if(mode==6)return std::abs(t)>2?scale:std::abs(t)<1?scale*(1-3*t*t):0;return 0;}
 bool trace(char kind,double x){++calls;bool good=calls!=fail;std::cout<<kind<<' '<<bits(x)<<' '<<good<<' ';return good;}
 bool Value(double x,double& v)override{v=val(x);bool good=trace('v',x);std::cout<<bits(v)<<' ';return good;}
 bool Derivative(double x,double& d)override{d=der(x);bool good=trace('d',x);std::cout<<bits(d)<<' ';return good;}
 bool Values(double x,double& v,double& d)override{v=val(x);d=der(x);bool good=trace('c',x);std::cout<<bits(v)<<' '<<bits(d)<<' ';return good;}
 int GetStateNumber()override{++states;std::cout<<"s "<<states<<' ';return states;}
};
void sampled_case(std::istream& input){
 int mode,fail,samples;double center,offset,scale,lo,hi,ex,ef,en,target;input>>mode>>fail>>center>>offset>>scale>>lo>>hi>>samples>>ex>>ef>>en>>target;
 SampledFunction f(mode,fail,center,offset,scale);math_FunctionRoots s(f,lo,hi,samples,ex,ef,en,target);
 std::cout<<s.IsDone()<<' ';int count=0;
 try{bool all=s.IsAllNull();std::cout<<all<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try{count=s.NbSolutions();std::cout<<count<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 for(int i=1;i<=count;i++)std::cout<<bits(s.Value(i))<<' '<<s.StateNumber(i)<<' ';
 for(int i:{0,count+1}){
  try{double v=s.Value(i);std::cout<<bits(v)<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  try{int n=s.StateNumber(i);std::cout<<n<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 }
 std::cout<<f.calls<<' '<<f.states<<' '<<'\n';
}
