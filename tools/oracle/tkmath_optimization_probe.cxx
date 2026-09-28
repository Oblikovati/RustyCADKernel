// Exact behavior probe for OCCT BFGS and FRPR at the pinned revision.
#include <math_BFGS.hxx>
#include <math_Powell.hxx>
#include <math_NewtonMinimum.hxx>
#include <math_MultipleVarFunctionWithHessian.hxx>
#include <math_FRPR.hxx>
#include <math_MultipleVarFunctionWithGradient.hxx>
struct OracleObjective:math_MultipleVarFunctionWithHessian {
 int n,mode,fail,calls=0,states=0;bool recordInput=true;double offset;std::vector<double> centers,weights,trace;
 int NbVariables() const override {return n;}
 double Evaluate(const math_Vector& point) const {
  double result=offset;
  for(int i=1;i<=n;i++){double d=point(i)-centers[i-1];result+=weights[i-1]*d*d;if(mode==1)result+=0.1*d*d*d*d;}
  return result;
 }
 bool Value(const math_Vector& point,double& value) override {
  trace.push_back(1);for(int i=1;i<=n;i++)trace.push_back(point(i));if(recordInput)trace.push_back(value);
  value=Evaluate(point);trace.push_back(value);return ++calls!=fail;
 }
 bool Gradient(const math_Vector& point,math_Vector& gradient) override {
  for(int i=1;i<=n;i++){double d=point(i)-centers[i-1];gradient(i)=2.0*weights[i-1]*d;if(mode==1)gradient(i)+=0.4*d*d*d;}
  return true;
 }
 bool Values(const math_Vector& point,double& value,math_Vector& gradient) override {
  trace.push_back(2);for(int i=1;i<=n;i++)trace.push_back(point(i));if(recordInput)trace.push_back(value);
  value=Evaluate(point);Gradient(point,gradient);trace.push_back(value);
  for(int i=1;i<=n;i++)trace.push_back(gradient(i));return ++calls!=fail;
 }
 bool Values(const math_Vector& point,double& value,math_Vector& gradient,math_Matrix& hessian) override {
  trace.push_back(4);for(int i=1;i<=n;i++)trace.push_back(point(i));
  value=Evaluate(point);Gradient(point,gradient);trace.push_back(value);
  for(int i=1;i<=n;i++)trace.push_back(gradient(i));hessian.Init(0.0);
  for(int i=1;i<=n;i++){double d=point(i)-centers[i-1];hessian(i,i)=2.0*weights[i-1];if(mode==1)hessian(i,i)+=1.2*d*d;}
  for(int i=1;i<=n;i++)for(int j=1;j<=n;j++)trace.push_back(hessian(i,j));return ++calls!=fail;
 }
 int GetStateNumber() override {states++;trace.push_back(3);return 29;}
};
template<class Solver> void optimization_results(const Solver& solver,const OracleObjective& function) {
 std::cout<<solver.IsDone()<<' ';
 try {std::cout<<bits(solver.Minimum())<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {std::cout<<solver.NbIterations()<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {const math_Vector& p=solver.Location();for(int i=1;i<=p.Upper();i++)std::cout<<bits(p(i))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {const math_Vector& p=solver.Gradient();for(int i=1;i<=p.Upper();i++)std::cout<<bits(p(i))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {math_Vector p(-3,function.n-4,71.0);solver.Location(p);for(int i=p.Lower();i<=p.Upper();i++)std::cout<<bits(p(i))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {math_Vector p(-3,function.n-4,71.0);solver.Gradient(p);for(int i=p.Lower();i<=p.Upper();i++)std::cout<<bits(p(i))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 std::cout<<function.calls<<' '<<function.states<<' '<<function.trace.size()<<' ';for(double v:function.trace)std::cout<<bits(v)<<' ';
}
void powell_results(const math_Powell& solver,const OracleObjective& function) {
 std::cout<<solver.IsDone()<<' ';
 try {std::cout<<bits(solver.Minimum())<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {std::cout<<solver.NbIterations()<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {const math_Vector& p=solver.Location();for(int i=1;i<=p.Upper();i++)std::cout<<bits(p(i))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {math_Vector p(-3,function.n-4,71.0);solver.Location(p);for(int i=p.Lower();i<=p.Upper();i++)std::cout<<bits(p(i))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 std::cout<<function.calls<<' '<<function.states<<' '<<function.trace.size()<<' ';for(double v:function.trace)std::cout<<bits(v)<<' ';
}
void extra_optimization(OracleObjective& function,const math_Vector& start,int limit,double tolerance,double epsilon,double width,int repeat) {
 const int n=function.n;function.calls=0;function.states=0;function.trace.clear();function.fail=n%3;function.recordInput=false;
 math_Matrix directions(1,n,1,n);
 for(int i=1;i<=n;i++)for(int j=1;j<=n;j++)directions(i,j)=i==j?1.0:0.125*(i-j);
 math_Powell powell(function,tolerance,limit,epsilon);powell.Perform(function,start,directions);powell_results(powell,function);
 if(repeat){function.calls=0;function.states=0;function.trace.clear();function.fail=1;powell.Perform(function,start,directions);powell_results(powell,function);}
 function.calls=0;function.states=0;function.trace.clear();function.fail=n%4;
 if(n%2==0)function.weights[0]=-function.weights[0];if(n==3)function.weights[0]=0.0;
 const double convexity=n==5?100.0:1e-6;
 math_NewtonMinimum solver(function,tolerance,limit,convexity,repeat!=0);
 if(width>=0)solver.SetBoundary(math_Vector(1,n,-width),math_Vector(1,n,width));
 solver.Perform(function,start);std::cout<<int(solver.GetStatus())<<' '<<solver.IsConverged()<<' ';optimization_results(solver,function);
 if(repeat){function.calls=0;function.states=0;function.trace.clear();function.fail=0;solver.Perform(function,start);std::cout<<int(solver.GetStatus())<<' '<<solver.IsConverged()<<' ';optimization_results(solver,function);}
}
void optimization_case(std::istream& input) {
 OracleObjective function;int limit,repeat;double tolerance,epsilon,width;
 input>>function.n>>function.mode>>function.offset>>function.fail>>limit>>tolerance>>epsilon>>width>>repeat;
 const int n=function.n;function.centers.resize(n);function.weights.resize(n);
 for(double& v:function.centers)input>>v;for(double& v:function.weights)input>>v;
 math_Vector start(1,n),left(1,n,-width),right(1,n,width);for(int i=1;i<=n;i++)input>>start(i);
 math_BFGS bfgs(n,tolerance,limit,epsilon);if(width>=0)bfgs.SetBoundary(left,right);
 bfgs.Perform(function,start);optimization_results(bfgs,function);
 if(repeat){function.calls=0;function.states=0;function.trace.clear();function.fail=1;bfgs.Perform(function,start);optimization_results(bfgs,function);}
 function.calls=0;function.states=0;function.trace.clear();function.fail=0;
 math_FRPR frpr(function,tolerance,limit,epsilon);frpr.Perform(function,start);optimization_results(frpr,function);
 if(repeat){function.calls=0;function.states=0;function.trace.clear();function.fail=1;frpr.Perform(function,start);optimization_results(frpr,function);}
 extra_optimization(function,start,limit,tolerance,epsilon,width,repeat);
 std::cout<<'\n';
}
