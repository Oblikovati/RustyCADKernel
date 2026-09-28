// Dynamic matrix and Gaussian elimination differential oracle.
#include <math_Gauss.hxx>
#include <math_Crout.hxx>
#include <math_Jacobi.hxx>
void numeric_matrix_values(const math_Matrix& matrix) {
  std::cout<<matrix.LowerRow()<<' '<<matrix.UpperRow()<<' '<<matrix.LowerCol()<<' '<<matrix.UpperCol()<<' ';
  for(size_t row=0;row<matrix.RowSize();++row)for(size_t col=0;col<matrix.ColSize();++col)std::cout<<bits(matrix.At(row,col))<<' ';
}
void lower_matrix_values(const math_Matrix& matrix) {
  for(int row=0;row<matrix.RowNumber();++row)for(int col=0;col<=row;++col)std::cout<<bits(matrix.At(row,col))<<' ';
}
void crout_values(const math_Matrix& matrix,const math_Vector& rhs,double minimum) {
  math_Crout solver(matrix,minimum);std::cout<<solver.IsDone()<<' ';
  try {std::cout<<bits(solver.Determinant())<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  try {lower_matrix_values(solver.Inverse());}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  const int n=rhs.Length();math_Vector solution(5,5+n-1);solution.Init(91.0);
  try {solver.Solve(rhs,solution);numeric_vector_values(solution);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  auto aliased=rhs;try {solver.Solve(aliased,aliased);numeric_vector_values(aliased);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  math_Matrix inverse(2,2+n-1,4,4+n-1);
  try {solver.Invert(inverse);lower_matrix_values(inverse);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
void jacobi_values(const math_Matrix& matrix) {
  math_Jacobi solver(matrix);std::cout<<solver.IsDone()<<' ';
  if(!solver.IsDone())return;
  numeric_vector_values(solver.Values());numeric_matrix_values(solver.Vectors());
  math_Vector vector(5,4+matrix.RowNumber());
  for(int index=1;index<=matrix.RowNumber();++index) {
    std::cout<<bits(solver.Value(index))<<' ';
    solver.Vector(index,vector);numeric_vector_values(vector);
  }
}
void numeric_matrix_case(std::istream& input) {
  int n,rl,cl;double minimum,scalar;input>>n>>rl>>cl>>minimum>>scalar;
  math_Matrix matrix(rl,rl+n-1,cl,cl+n-1);math_Vector rhs(-3,-3+n-1);
  for(int row=0;row<n;++row)for(int col=0;col<n;++col)input>>matrix.ChangeAt(row,col);
  for(int index=0;index<n;++index)input>>rhs.ChangeAt(index);
  jacobi_values(matrix);
  crout_values(matrix,rhs,minimum);
  auto transpose=matrix.Transposed();numeric_matrix_values(transpose);
  numeric_matrix_values(matrix.Added(transpose));numeric_matrix_values(matrix.Subtracted(transpose));numeric_matrix_values(matrix.Multiplied(scalar));
  try {numeric_matrix_values(matrix.Divided(scalar));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  auto divided=matrix;try {divided.Divide(scalar);numeric_matrix_values(divided);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  numeric_matrix_values(matrix.Multiplied(transpose));auto square=matrix;square.Multiply(square);numeric_matrix_values(square);
  auto inplace=matrix;try {inplace.Multiply(transpose);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}numeric_matrix_values(inplace);
  math_Gauss solver(matrix,minimum);std::cout<<solver.IsDone()<<' ';
  try {std::cout<<bits(solver.Determinant())<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  math_Vector solution(5,5+n-1);solution.Init(91.0);
  try {solver.Solve(rhs,solution);numeric_vector_values(solution);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  try {solver.Solve(rhs);numeric_vector_values(rhs);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  math_Matrix inverse(2,2+n-1,4,4+n-1);
  try {solver.Invert(inverse);numeric_matrix_values(inverse);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  std::cout<<bits(matrix.Determinant())<<' ';
  try {numeric_matrix_values(matrix.Inverse());}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  std::cout<<'\n';
}
