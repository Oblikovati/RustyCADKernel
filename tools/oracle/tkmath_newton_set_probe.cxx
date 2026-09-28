// Newton system callbacks, LU mutation, clipping, and state capture.
#include <math_NewtonFunctionSetRoot.hxx>
#include <math_FunctionSetRoot.hxx>
#include <math_FunctionSetWithDerivatives.hxx>
void newton_vector(const math_Vector& vector){for(int i=vector.Lower();i<=vector.Upper();++i)std::cout<<bits(vector(i))<<' ';}
void newton_matrix(const math_Matrix& matrix){for(int row=matrix.LowerRow();row<=matrix.UpperRow();++row)for(int col=matrix.LowerCol();col<=matrix.UpperCol();++col)std::cout<<bits(matrix(row,col))<<' ';}
struct OraclePolynomialSystem:math_FunctionSetWithDerivatives {
  math_Matrix linear;math_Vector quadratic,rhs;int calls=0,fail;
  OraclePolynomialSystem(int n,int stop,int m=0):linear(1,m?m:n,1,n),quadratic(1,m?m:n),rhs(1,m?m:n),fail(stop){}
  int NbVariables()const override{return linear.ColNumber();}
  int NbEquations()const override{return rhs.Length();}
  bool Value(const math_Vector& point,math_Vector& residual)override {
    std::cout<<"value ";newton_vector(point);for(int row=1;row<=NbEquations();++row){double sum=0.0;for(int col=1;col<=NbVariables();++col)sum+=linear(row,col)*point(col);residual(row)=sum+quadratic(row)*point((row-1)%NbVariables()+1)*point((row-1)%NbVariables()+1)-rhs(row);}return true;
  }
  bool Derivatives(const math_Vector& point,math_Matrix& jacobian)override {
    jacobian=linear;for(int i=1;i<=NbEquations();++i){int col=(i-1)%NbVariables()+1;jacobian(i,col)+=2.0*quadratic(i)*point(col);}return true;
  }
  bool Values(const math_Vector& point,math_Vector& residual,math_Matrix& jacobian)override {
    ++calls;std::cout<<"call "<<calls<<' ';newton_vector(point);if(calls>1)newton_matrix(jacobian);
    Value(point,residual);Derivatives(point,jacobian);return calls!=fail;
  }
  int GetStateNumber()override {std::cout<<"state "<<calls<<' ';return calls;}
};
void newton_set_case(std::istream& input) {
  int n,limit,fail;double xtol,ftol,bound;input>>n>>xtol>>ftol>>limit>>bound>>fail;
  OraclePolynomialSystem function(n,fail);
  for(int row=1;row<=n;++row)for(int col=1;col<=n;++col)input>>function.linear(row,col);
  for(int i=1;i<=n;++i)input>>function.quadratic(i);
  for(int i=1;i<=n;++i)input>>function.rhs(i);
  math_Vector start(-3,n-4),tolerance(1,n,xtol),lower(1,n,-bound),upper(1,n,bound);
  for(int i=-3;i<=n-4;++i)input>>start(i);
  math_NewtonFunctionSetRoot solver(function,tolerance,ftol,limit);solver.Perform(function,start,lower,upper);
  std::cout<<"done "<<solver.IsDone()<<' ';
  if(solver.IsDone()) {
    std::cout<<solver.NbIterations()<<' ';newton_vector(solver.Root());newton_vector(solver.FunctionSetErrors());newton_matrix(solver.Derivative());
    math_Vector vector(5,n+4);math_Matrix matrix(2,n+1,4,n+3);
    solver.Root(vector);newton_vector(vector);solver.FunctionSetErrors(vector);newton_vector(vector);solver.Derivative(matrix);newton_matrix(matrix);
  }
  std::cout<<'\n';
}

void function_set_case(std::istream& input) {
  int n,m,limit,fail,stop;double tolerance,bound;input>>n>>m>>tolerance>>limit>>bound>>fail>>stop;
  OraclePolynomialSystem function(n,fail,m);
  for(int row=1;row<=m;++row)for(int col=1;col<=n;++col)input>>function.linear(row,col);
  for(int i=1;i<=m;++i)input>>function.quadratic(i);
  for(int i=1;i<=m;++i)input>>function.rhs(i);
  math_Vector start(-3,n-4),tol(1,n,tolerance),lower(1,n,-bound),upper(1,n,bound);
  for(int i=-3;i<=n-4;++i)input>>start(i);
  math_FunctionSetRoot solver(function,tol,limit);solver.Perform(function,start,lower,upper,stop!=0);
  std::cout<<"done "<<solver.IsDone()<<' '<<solver.IsDivergent()<<' ';
  if(solver.IsDone()){std::cout<<solver.NbIterations()<<' '<<solver.StateNumber()<<' ';newton_vector(solver.Root());if(function.calls>1)newton_vector(solver.FunctionSetErrors());newton_matrix(solver.Derivative());}
  std::cout<<'\n';
}
