// Uzawa: compare only storage initialized by the selected native branch.
#include <math_Uzawa.hxx>
struct ProbeUzawa:math_Uzawa {using math_Uzawa::math_Uzawa;using math_Uzawa::Perform;};
void uzawa_vector(const math_Vector& value,int count) {
 std::cout<<value.Lower()<<' '<<value.Upper()<<' ';for(int i=1;i<=count;i++)std::cout<<bits(value(i))<<' ';
}
void uzawa_results(const math_Uzawa& solver,int rows,int columns,int inequalities) {
 std::cout<<solver.IsDone()<<' ';
 try {std::cout<<solver.NbIterations()<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {uzawa_vector(solver.Value(),columns);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {uzawa_vector(solver.Error(),columns);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {uzawa_vector(solver.InitialError(),rows);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 try {const math_Matrix& inverse=solver.InverseCont();std::cout<<inverse.RowNumber()<<' '<<inverse.ColNumber()<<' ';if(!inequalities)for(int i=1;i<=rows;i++)for(int j=1;j<=i;j++)std::cout<<bits(inverse(i,j))<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 math_Vector dual(-3,rows-4,71.0);solver.Duale(dual);for(int i=dual.Lower();i<=dual.Upper();i++)std::cout<<bits(dual(i))<<' ';
}
void uzawa_runs(ProbeUzawa& solver,const math_Matrix& matrix,const math_Vector& rhs,math_Vector& start,int equalities,int inequalities,double epsx,double epsdual,int limit) {
 uzawa_results(solver,matrix.RowNumber(),matrix.ColNumber(),inequalities);
 start.Multiply(-1.0);solver.Perform(matrix,rhs,start,equalities,inequalities,epsx,epsdual,limit);
 uzawa_results(solver,matrix.RowNumber(),matrix.ColNumber(),inequalities);
}
void uzawa_case(std::istream& input) {
 int rows,columns,equalities,inequalities,limit,mode;double epsx,epsdual;
 input>>rows>>columns>>equalities>>inequalities>>epsx>>epsdual>>limit>>mode;
 math_Matrix matrix(1,rows,1,columns);math_Vector rhs(1,rows),start(1,columns);
 for(int i=1;i<=rows;i++)for(int j=1;j<=columns;j++)input>>matrix(i,j);
 for(int i=1;i<=rows;i++)input>>rhs(i);for(int i=1;i<=columns;i++)input>>start(i);
 try {
  if(mode==0){ProbeUzawa solver(matrix,rhs,start,epsx,epsdual,limit);uzawa_runs(solver,matrix,rhs,start,rows,0,epsx,epsdual,limit);}
  else {ProbeUzawa solver(matrix,rhs,start,equalities,inequalities,epsx,epsdual,limit);uzawa_runs(solver,matrix,rhs,start,equalities,inequalities,epsx,epsdual,limit);}
 }catch(const Standard_Failure& e){std::cout<<"EXCEPTION "<<e.ExceptionType()<<' ';}
 std::cout<<'\n';
}
