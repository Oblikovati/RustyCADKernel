// Normal-equation least-squares oracle, including disabled-dimension-check paths.
#include <math_GaussLeastSquare.hxx>
#include <math_Householder.hxx>
void householder_values(const math_Householder& solver,int columns) {
  std::cout<<solver.IsDone()<<' ';
  try {numeric_matrix_values(solver.AllValues());}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  math_Vector solution(5,5+columns-1);solution.Init(91.0);
  try {solver.Value(solution,1);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}numeric_vector_values(solution);
}
void householder_cases(const math_Matrix& matrix,const math_Vector& rhs,double epsilon) {
  math_Householder vectorSolver(matrix,rhs,epsilon);householder_values(vectorSolver,matrix.ColNumber());
  math_Matrix rebased(-4,-4+matrix.RowNumber()-1,1,matrix.ColNumber());rebased=matrix;
  math_Matrix multiple(-4,-4+matrix.RowNumber()-1,1,2);
  for(int row=0;row<matrix.RowNumber();++row){multiple.ChangeAt(row,0)=rhs.At(row);multiple.ChangeAt(row,1)=rhs.At(row)*0.5+row;}
  math_Householder matrixSolver(rebased,multiple,epsilon);householder_values(matrixSolver,matrix.ColNumber());
  math_Householder rangeSolver(matrix,multiple,matrix.LowerRow(),matrix.UpperRow(),matrix.LowerCol(),matrix.UpperCol(),epsilon);
  householder_values(rangeSolver,matrix.ColNumber());
  math_Matrix padded(matrix.LowerRow()-1,matrix.UpperRow()+1,matrix.LowerCol()-1,matrix.UpperCol()+1,91.0);
  for(int row=matrix.LowerRow();row<=matrix.UpperRow();++row)for(int col=matrix.LowerCol();col<=matrix.UpperCol();++col)padded(row,col)=matrix(row,col);
  math_Householder submatrixSolver(padded,multiple,matrix.LowerRow(),matrix.UpperRow(),matrix.LowerCol(),matrix.UpperCol(),epsilon);
  householder_values(submatrixSolver,matrix.ColNumber());
}
void least_squares_case(std::istream& input) {
  int rows,cols,rl,cl;double minimum;input>>rows>>cols>>rl>>cl>>minimum;
  math_Matrix matrix(rl,rl+rows-1,cl,cl+cols-1);
  for(int row=0;row<rows;++row)for(int col=0;col<cols;++col)input>>matrix.ChangeAt(row,col);
  math_Vector rhs(-3,-3+rows+1);rhs.Init(91.0);
  for(int row=0;row<rows;++row)input>>rhs.ChangeAt(row);
  householder_cases(matrix,rhs,minimum);
  math_GaussLeastSquare solver(matrix,minimum);std::cout<<solver.IsDone()<<' ';
  math_Vector solution(5,5+cols-1);solution.Init(91.0);
  try {solver.Solve(rhs,solution);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}numeric_vector_values(solution);
  math_Vector oversized(2,2+cols+1);oversized.Init(91.0);
  try {solver.Solve(rhs,oversized);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}numeric_vector_values(oversized);
  auto aliased=rhs;
  try {solver.Solve(aliased,aliased);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}numeric_vector_values(aliased);
  std::cout<<'\n';
}
