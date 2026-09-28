// Minimum bracket outputs and callback traces.
#include <math_BracketMinimum.hxx>
#include <math_BrentMinimum.hxx>
struct OracleBowl:math_Function {
  double center,scale,offset;int fail;std::vector<std::pair<double,double>> trace;
  double Evaluate(double x) const {return scale*(x-center)*(x-center)+offset;}
  bool Value(double x,double& result) override {result=Evaluate(x);trace.emplace_back(x,result);return int(trace.size())!=fail;}
};
void minimum_results(const math_BracketMinimum& solver,const OracleBowl& function) {
  std::cout<<solver.IsDone()<<' ';double a,b,c;
  try {solver.Values(a,b,c);std::cout<<bits(a)<<' '<<bits(b)<<' '<<bits(c)<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {solver.FunctionValues(a,b,c);std::cout<<bits(a)<<' '<<bits(b)<<' '<<bits(c)<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  std::cout<<function.trace.size()<<' ';
  for(const auto& entry:function.trace)std::cout<<bits(entry.first)<<' '<<bits(entry.second)<<' ';

}
void print_brent(const math_BrentMinimum& solver,const OracleBowl& function) {
  std::cout<<solver.IsDone()<<' ';
  try {std::cout<<bits(solver.Location())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<bits(solver.Minimum())<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  try {std::cout<<solver.NbIterations()<<' ';}catch(const Standard_Failure& error){std::cout<<error.ExceptionType()<<' ';}
  std::cout<<function.trace.size()<<' ';
  for(const auto& entry:function.trace)std::cout<<bits(entry.first)<<' '<<bits(entry.second)<<' ';
}
void brent_runs(math_BrentMinimum& solver,OracleBowl& function,double first,double second) {
  function.trace.clear();solver.Perform(function,first,0.5*(first+second),second);print_brent(solver,function);
  function.trace.clear();function.fail=1;solver.Perform(function,first,first,second);print_brent(solver,function);
}
void brent_values(OracleBowl& function,double first,double second,int mode) {
  const int limit=mode==1?2:100;
  if(mode==2){math_BrentMinimum solver(1e-10,function.Evaluate(0.5*(first+second)),limit,1e-12);brent_runs(solver,function,first,second);}
  else {math_BrentMinimum solver(1e-10,limit,1e-12);brent_runs(solver,function,first,second);}
}
void minimum_case(std::istream& input) {
  double first,second,left,right;int mode;OracleBowl function;
  input>>first>>second>>function.center>>function.scale>>function.offset>>mode>>left>>right>>function.fail;
  const double fa=function.Evaluate(first),fb=function.Evaluate(second);
  if(mode==0){math_BracketMinimum solver(function,first,second);minimum_results(solver,function);}
  else if(mode==1){math_BracketMinimum solver(function,first,second,fa);minimum_results(solver,function);}
  else if(mode==2){math_BracketMinimum solver(function,first,second,fa,fb);minimum_results(solver,function);}
  else {math_BracketMinimum solver(first,second);solver.SetLimits(left,right);solver.Perform(function);minimum_results(solver,function);}
  brent_values(function,first,second,mode);std::cout<<'\n';
}
