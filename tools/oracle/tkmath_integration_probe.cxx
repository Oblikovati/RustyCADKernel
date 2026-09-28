// Integration callback traces, table literals, computed fallback and adaptive errors.
#include <math_GaussSingleIntegration.hxx>
#include <math_KronrodSingleIntegration.hxx>
#include <math_ComputeGaussPointsAndWeights.hxx>
#include <math.hxx>
void gauss_result(const math_GaussSingleIntegration&s){std::cout<<s.IsDone()<<' ';try{double v=s.Value();std::cout<<bits(v)<<' ';}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}}
void kronrod_result(const math_KronrodSingleIntegration&s){std::cout<<s.IsDone()<<' ';if(s.IsDone()){std::cout<<bits(s.Value())<<' '<<bits(s.ErrorReached())<<' '<<bits(s.AbsolutError())<<' '<<s.OrderReached()<<' '<<s.NbIterReached()<<' ';}else{try{s.Value();}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}try{s.ErrorReached();}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}try{s.AbsolutError();}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}try{s.OrderReached();}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}try{s.NbIterReached();}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}}}
void integration_case(std::istream& input){
 int mode,fail,order,limit;double center,offset,scale,lo,hi,tol;input>>mode>>fail>>center>>offset>>scale>>lo>>hi>>order>>tol>>limit;
 {SampledFunction f(mode,fail,center,offset,scale);math_GaussSingleIntegration s(f,lo,hi,order);gauss_result(s);}
 {SampledFunction f(mode,fail,center,offset,scale);math_GaussSingleIntegration s(f,lo,hi,order,tol);gauss_result(s);}
 {SampledFunction f(mode,fail,center,offset,scale);math_KronrodSingleIntegration s(f,lo,hi,order);kronrod_result(s);s.Perform(f,lo,lo,order);kronrod_result(s);s.Perform(f,lo,hi,order);kronrod_result(s);}
 {SampledFunction f(mode,fail,center,offset,scale);math_KronrodSingleIntegration s(f,lo,hi,order,tol,limit);kronrod_result(s);s.Perform(f,hi,lo,order,tol,limit);kronrod_result(s);}
 std::cout<<'\n';
}
void table_result(const math_Vector&p,const math_Vector&w){for(int i=p.Lower();i<=p.Upper();i++)std::cout<<bits(p(i))<<' '<<bits(w(i))<<' ';}
void quadrature_case(std::istream& input){int n;input>>n;std::cout<<math::GaussPointsMax()<<' '<<math::KronrodPointsMax()<<' ';
 math_Vector p(1,n),w(1,n);if(n<=61){math::GaussPoints(n,p);math::GaussWeights(n,w);table_result(p,w);}
 bool good=math::OrderedGaussPointsAndWeights(n,p,w);std::cout<<good<<' ';if(good)table_result(p,w);
 math_ComputeGaussPointsAndWeights computed(n);std::cout<<computed.IsDone()<<' ';if(computed.IsDone())table_result(computed.Points(),computed.Weights());
 math_Vector kp(-3,n-4),kw(-3,n-4);good=math::KronrodPointsAndWeights(n,kp,kw);std::cout<<good<<' ';if(good)table_result(kp,kw);
 std::cout<<'\n';}
