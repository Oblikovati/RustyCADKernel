// Native tridiagonal QL ordering and bounded inputs.
#include <math_EigenValuesSearcher.hxx>
void eigen_case(std::istream& input) {
  int n,lower;input>>n>>lower;
  NCollection_Array1<double> diagonal(lower,lower+n-1),subdiagonal(-lower,n-lower-1);
  for(int index=0;index<n;++index)input>>diagonal(lower+index);
  for(int index=0;index<n;++index)input>>subdiagonal(index-lower);
  math_EigenValuesSearcher solver(diagonal,subdiagonal);
  std::cout<<solver.IsDone()<<' '<<solver.Dimension()<<' ';
  if(solver.IsDone())for(int index=1;index<=n;++index) {
    std::cout<<bits(solver.EigenValue(index))<<' ';auto vector=solver.EigenVector(index);
    for(int row=1;row<=n;++row)std::cout<<bits(vector(row))<<' ';
  }
  std::cout<<'\n';
}
