// Interval boundaries and their captured states, including native NumericError paths.
#include <math_FunctionAllRoots.hxx>
#include <math_FunctionSample.hxx>
void all_roots_case(std::istream& input){
 int mode,fail,count;double center,offset,scale,lo,hi,ex,ef,en;input>>mode>>fail>>center>>offset>>scale>>lo>>hi>>count>>ex>>ef>>en;
 SampledFunction f(mode,fail,center,offset,scale);math_FunctionSample sample(lo,hi,count);
 double a,b;sample.Bounds(a,b);std::cout<<bits(a)<<' '<<bits(b)<<' '<<sample.NbPoints()<<' ';
 for(int i:{0,1,count,count+1})std::cout<<bits(sample.GetParameter(i))<<' ';
 try {
 math_FunctionAllRoots s(f,sample,ex,ef,en);std::cout<<s.IsDone()<<' '<<s.NbIntervals()<<' '<<s.NbPoints()<<' ';
 for(int i=1;i<=s.NbIntervals();i++){int first,last;s.GetInterval(i,a,b);s.GetIntervalState(i,first,last);std::cout<<bits(a)<<' '<<bits(b)<<' '<<first<<' '<<last<<' ';}
 for(int i=1;i<=s.NbPoints();i++)std::cout<<bits(s.GetPoint(i))<<' '<<s.GetPointState(i)<<' ';
 for(int i:{0,s.NbIntervals()+1}){
 try{s.GetInterval(i,a,b);}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}
 try{int first,last;s.GetIntervalState(i,first,last);}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}}
 for(int i:{0,s.NbPoints()+1}){
 try{s.GetPoint(i);}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}
 try{s.GetPointState(i);}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}}
 } catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}
 std::cout<<f.calls<<' '<<f.states<<' '<<'\n';
}
