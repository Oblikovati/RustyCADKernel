// Stateful SVD solves and pseudoinverses; native input is unchanged.
#include <math_SVD.hxx>
void svd_outputs(math_SVD& solver,const math_Vector& rhs,int rows,int cols,double epsilon) {
  math_Vector output(5,4+cols);output.Init(91.0);solver.Solve(rhs,output,epsilon);
  for(int index=5;index<=4+cols;++index)std::cout<<bits(output(index))<<' ';
  math_Matrix inverse(1,cols,1,rows);solver.PseudoInverse(inverse,epsilon);
  for(int row=1;row<=cols;++row)for(int col=1;col<=rows;++col)std::cout<<bits(inverse(row,col))<<' ';
}
void svd_case(std::istream& input) {
  int rows,cols,rl,cl;double first,second;input>>rows>>cols>>rl>>cl>>first>>second;
  math_Matrix matrix(rl,rl+rows-1,cl,cl+cols-1);
  for(int row=rl;row<rl+rows;++row)for(int col=cl;col<cl+cols;++col)input>>matrix(row,col);
  math_Vector rhs(-2,rows-3);for(int row=-2;row<rows-2;++row)input>>rhs(row);
  math_SVD solver(matrix);std::cout<<solver.IsDone()<<' ';
  if(solver.IsDone()){svd_outputs(solver,rhs,rows,cols,first);svd_outputs(solver,rhs,rows,cols,second);}
  std::cout<<'\n';
}
