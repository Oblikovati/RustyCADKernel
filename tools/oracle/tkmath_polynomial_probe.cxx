// Native algebraic-root order, multiplicity, guards and protected Solve flag persistence.
#include <math_DirectPolynomialRoots.hxx>
struct ProbePolynomial:math_DirectPolynomialRoots {using math_DirectPolynomialRoots::math_DirectPolynomialRoots;using math_DirectPolynomialRoots::Solve;};
void polynomial_results(const ProbePolynomial& solver,bool initializedCount=false) {
 std::cout<<solver.IsDone()<<' '<<solver.InfiniteRoots()<<' ';
 // Failed fresh constructors may leave NbSolutions uninitialized in native OCCT.
 if(!initializedCount && !solver.IsDone() && !solver.InfiniteRoots())return;
 int count=0;try {count=solver.NbSolutions();std::cout<<count<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 for(int i=1;i<=count;i++)std::cout<<bits(solver.Value(i))<<' ';
 for(int index:{0,count+1}){try {std::cout<<bits(solver.Value(index))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
}
void polynomial_repeat(ProbePolynomial& solver) {
 polynomial_results(solver);
 if(solver.IsDone()&&!solver.InfiniteRoots()){solver.Solve(1.0,1e60,1.0,1.0);polynomial_results(solver,true);}
 solver.Solve(0.0,0.0);polynomial_results(solver);solver.Solve(1.0,-2.0);polynomial_results(solver);
 solver.Solve(1.0,-5.0,6.0);polynomial_results(solver);solver.Solve(1.0,-6.0,11.0,-6.0);polynomial_results(solver);
 solver.Solve(1.0,0.0,-5.0,0.0,4.0);polynomial_results(solver);
}
void polynomial_case(std::istream& input) {
 int degree;double a,b,c=0,d=0,e=0;input>>degree>>a>>b;if(degree>1)input>>c;if(degree>2)input>>d;if(degree>3)input>>e;
 if(degree==1){ProbePolynomial solver(a,b);polynomial_repeat(solver);}
 else if(degree==2){ProbePolynomial solver(a,b,c);polynomial_repeat(solver);}
 else if(degree==3){ProbePolynomial solver(a,b,c,d);polynomial_repeat(solver);}
 else {ProbePolynomial solver(a,b,c,d,e);polynomial_repeat(solver);}
 std::cout<<'\n';
}
