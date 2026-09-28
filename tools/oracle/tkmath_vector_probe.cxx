// Bounded-vector arithmetic and storage oracle for pinned math_VectorBase<double>.
#include <math_Vector.hxx>
#include <math_Matrix.hxx>
void numeric_vector_values(const math_Vector& vector) {
  std::cout<<vector.Lower()<<' '<<vector.Upper()<<' '<<vector.Length()<<' ';
  for(size_t i=0;i<vector.Size();++i)std::cout<<bits(vector.At(i))<<' ';
}
void numeric_vector_case(std::istream& input) {
  int length,lower;double a,b,c,divisor;input>>length>>lower>>a>>b>>c>>divisor;
  math_Vector left(lower,lower+length-1),right(-3,-3+length-1);
  for(int i=0;i<length;++i) {left.ChangeAt(i)=a+i*b;right.ChangeAt(i)=c-i*b;}
  std::cout<<bits(left.Norm())<<' '<<bits(left.Norm2())<<' '<<left.Max()<<' '<<left.Min()<<' '<<bits(left.Multiplied(right))<<' ';
  numeric_vector_values(left.Added(right));numeric_vector_values(left.Subtracted(right));numeric_vector_values(left.Multiplied(divisor));
  try {numeric_vector_values(left.Divided(divisor));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  math_Vector divided(left);try {divided.Divide(divisor);numeric_vector_values(divided);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  try {numeric_vector_values(left.Normalized());}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  numeric_vector_values(left.Inverse());numeric_vector_values(left.Slice(left.Upper(),left.Lower()));
  math_Matrix matrix(2,4,-2,-2+length-1);
  for(int row=0;row<3;++row)for(int col=0;col<length;++col)matrix.ChangeAt(row,col)=c+row*a-col*b;
  math_Vector row(7,9);row(7)=a;row(8)=b;row(9)=c;
  numeric_vector_values(row.Multiplied(matrix));
  math_Vector output(3,5);output.Multiply(matrix,left);numeric_vector_values(output);
  output.TMultiply(left,matrix);numeric_vector_values(output);
  math_Vector transposed(1,length);transposed.TMultiply(matrix,row);numeric_vector_values(transposed);
  math_Vector moved(std::move(left));numeric_vector_values(moved);numeric_vector_values(left);
  moved.Resize(length+40);moved.Resize(length);numeric_vector_values(moved);
  math_Vector empty(size_t{0});
  try {std::cout<<bits(empty.Norm())<<' ';}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
  std::cout<<'\n';
}
