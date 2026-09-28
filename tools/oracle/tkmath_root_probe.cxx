// Callback traces and bracketed-root arithmetic compared with native OCCT.
#include <math_BracketedRoot.hxx>
#include <math_FunctionRoot.hxx>
#include <math_NewtonFunctionRoot.hxx>
#include <math_FunctionWithDerivative.hxx>
#include <math_BissecNewton.hxx>
#include <vector>
struct OraclePolynomial:math_FunctionWithDerivative {
  int GetStateNumber()override {std::cout<<"scalar-state "<<trace.size()<<' ';return int(trace.size());}
  double a,b,c,d;bool success;std::vector<std::pair<double,double>> trace;
  bool Value(double x,double& result) override {
    result=((a*x+b)*x+c)*x+d;trace.emplace_back(x,result);return success;
  }
  bool Derivative(double x,double& result) override {result=(3.0*a*x+2.0*b)*x+c;return success;}
  bool Values(double x,double& value,double& derivative) override {Value(x,value);return Derivative(x,derivative);}
};
void bissec_values(OraclePolynomial& function,double first,double second,double tolerance,int limit) {
  function.trace.clear();math_BissecNewton solver(tolerance);solver.Perform(function,first,second,limit);
  std::cout<<solver.IsDone()<<' ';
  try {std::cout<<bits(solver.Root())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<bits(solver.Value())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<bits(solver.Derivative())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  std::cout<<function.trace.size()<<' ';
  for(const auto& entry:function.trace)std::cout<<bits(entry.first)<<' '<<bits(entry.second)<<' ';
}
template<class Solver> void function_root_results(const Solver& solver,const OraclePolynomial& function) {
  std::cout<<solver.IsDone()<<' ';
  try {std::cout<<bits(solver.Root())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<bits(solver.Value())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<bits(solver.Derivative())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<solver.NbIterations()<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  std::cout<<function.trace.size()<<' ';for(const auto& entry:function.trace)std::cout<<bits(entry.first)<<' '<<bits(entry.second)<<' ';
}
void function_root_values(OraclePolynomial& function,double first,double second,double tolerance,int limit) {
  function.trace.clear();math_FunctionRoot unbounded(function,first,tolerance,limit);function_root_results(unbounded,function);
  function.trace.clear();math_FunctionRoot bounded(function,first,tolerance,first,second,limit);function_root_results(bounded,function);
}
void newton_root_values(OraclePolynomial& function,double first,double second,double tolerance,double epsilon,int limit) {
 function.trace.clear();math_NewtonFunctionRoot unbounded(function,first,tolerance,epsilon,limit);function_root_results(unbounded,function);
 function.trace.clear();math_NewtonFunctionRoot bounded(function,first,tolerance,epsilon,first,second,limit);function_root_results(bounded,function);
 function.trace.clear();bounded.Perform(function,second);function_root_results(bounded,function);
}
void root_case(std::istream& input) {
  double first,second,tolerance,epsilon;int limit,status;OraclePolynomial function;
  input>>first>>second>>tolerance>>limit>>epsilon>>status>>function.a>>function.b>>function.c>>function.d;
  function.success=status!=0;
  math_BracketedRoot solver(function,first,second,tolerance,limit,epsilon);
  std::cout<<solver.IsDone()<<' ';
  try {std::cout<<bits(solver.Root())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<bits(solver.Value())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<solver.NbIterations()<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  std::cout<<function.trace.size()<<' ';
  for(const auto& entry:function.trace)std::cout<<bits(entry.first)<<' '<<bits(entry.second)<<' ';
  bissec_values(function,first,second,tolerance,limit);
  function_root_values(function,first,second,tolerance,limit);
  newton_root_values(function,first,second,tolerance,epsilon,limit);
  std::cout<<'\n';
}
