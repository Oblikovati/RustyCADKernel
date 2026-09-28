// Native trigonometric special cases, periodic bounds, refinement, and callback expressions.
#include <math_TrigonometricFunctionRoots.hxx>
#include <math_TrigonometricEquationFunction.hxx>
struct ProbeTrig:math_TrigonometricFunctionRoots {using math_TrigonometricFunctionRoots::math_TrigonometricFunctionRoots;using math_TrigonometricFunctionRoots::Perform;};
void trig_results(const ProbeTrig& solver) {
 std::cout<<solver.IsDone()<<' '<<solver.InfiniteRoots()<<' ';int count=0;
 try {count=solver.NbSolutions();std::cout<<count<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 for(int i=1;i<=count;i++)std::cout<<bits(solver.Value(i))<<' ';
 for(int i:{0,count+1}){try {std::cout<<bits(solver.Value(i))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
}
void trig_runs(ProbeTrig& solver,double a,double b,double c,double d,double e,double lower,double upper) {
 trig_results(solver);solver.Perform(a,b,c,d,e,upper,lower);trig_results(solver);
 solver.Perform(0,0,0,0,0,0,2*M_PI);trig_results(solver);
 solver.Perform(0,0,0,1,0,0,2*M_PI);trig_results(solver);
 math_TrigonometricEquationFunction function(a,b,c,d,e);double f=91,derivative=92;
 for(double x:{0.0,0.731,lower,upper}) {
  std::cout<<function.Value(x,f)<<' '<<bits(f)<<' ';
  std::cout<<function.Derivative(x,derivative)<<' '<<bits(derivative)<<' ';
  std::cout<<function.Values(x,f,derivative)<<' '<<bits(f)<<' '<<bits(derivative)<<' ';
 }
}
void trig_case(std::istream& input) {
 int mode;double a,b,c,d,e,lower,upper;input>>mode>>a>>b>>c>>d>>e>>lower>>upper;
 if(mode==0){ProbeTrig solver(d,e,lower,upper);trig_runs(solver,0,0,0,d,e,lower,upper);}
 else if(mode==1){ProbeTrig solver(c,d,e,lower,upper);trig_runs(solver,0,0,c,d,e,lower,upper);}
 else {ProbeTrig solver(a,b,c,d,e,lower,upper);trig_runs(solver,a,b,c,d,e,lower,upper);}
 std::cout<<'\n';
}
