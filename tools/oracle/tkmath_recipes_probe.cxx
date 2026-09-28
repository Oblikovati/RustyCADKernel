// math_Recipes kernels called directly, plus the Dump text of the classes built on them.
#include <math_Recipes.hxx>
#include <math_Gauss.hxx>
#include <math_GaussLeastSquare.hxx>
#include <math_SVD.hxx>
#include <math_Jacobi.hxx>
namespace recipes_probe {
using vector_base_probe::cells;
using vector_base_probe::reals;
using vector_base_probe::values;
using vector_base_probe::guarded;
template<class T> void dump_call(const T& value) {
  std::ostringstream text;value.Dump(text);std::string s=text.str();
  for(char& ch:s)if(ch=='\n')ch='|';
  std::cout<<s<<' ';
}
math_Matrix read_matrix(std::istream& in,int rows,int cols) {
  math_Matrix m(1,rows,1,cols);for(int r=1;r<=rows;++r)for(int c=1;c<=cols;++c)in>>m(r,c);return m;
}
math_Vector read_vector(std::istream& in,int n) {math_Vector v(1,n);for(int i=1;i<=n;++i)in>>v(i);return v;}
// n tiny cells[n*n] rhs[n]
void lu_case(std::istream& in) {
  int n;double tiny;in>>n>>tiny;math_Matrix a=read_matrix(in,n,n);math_Vector b=read_vector(in,n);
  math_Matrix lu(a);math_IntegerVector indx(1,n);indx.Init(0);double d=91;
  int status=LU_Decompose(lu,indx,d,tiny);std::cout<<status<<' '<<bits(d)<<' ';cells(lu);values(indx);
  if(status==0){math_Vector x(b);LU_Solve(lu,indx,x);reals(x);math_Vector shifted(-2,-3+n);for(int i=1;i<=n;++i)shifted(i-3)=b(i);LU_Solve(lu,indx,shifted);reals(shifted);}
  math_Matrix scaled(a);math_Vector vv(1,n);vv.Init(91);indx.Init(0);d=91;
  status=LU_Decompose(scaled,indx,d,vv,tiny);std::cout<<status<<' '<<bits(d)<<' ';cells(scaled);values(indx);reals(vv);
  math_Matrix inv(a);std::cout<<LU_Invert(inv)<<' ';cells(inv);
  std::cout<<'\n';
}
// m n cells[m*n] rhs[m]
void svd_case(std::istream& in) {
  int m,n;in>>m>>n;math_Matrix a=read_matrix(in,m,n);math_Vector b=read_vector(in,m);
  math_Matrix u(a),v(1,n,1,n);math_Vector w(1,n);v.Init(91);w.Init(91);
  int status=SVD_Decompose(u,w,v);std::cout<<status<<' ';cells(u);reals(w);cells(v);
  math_Vector x(1,n);x.Init(91);SVD_Solve(u,w,v,b,x);reals(x);
  math_Matrix u2(a),v2(1,n,1,n);math_Vector w2(1,n),rv1(1,n);rv1.Init(91);
  std::cout<<SVD_Decompose(u2,w2,v2,rv1)<<' ';reals(rv1);
  std::cout<<'\n';
}
// n cells[n*n]
void jacobi_case(std::istream& in) {
  int n;in>>n;math_Matrix a=read_matrix(in,n,n);math_Matrix v(1,n,1,n);math_Vector d(1,n);int nrot=91;
  math_Matrix work(a);int status=Jacobi(work,d,v,nrot);std::cout<<status<<' '<<nrot<<' ';reals(d);cells(v);cells(work);
  std::cout<<'\n';
}
// neq min_pivot heights[neq] entries[total] rhs[neq]
void dactcl_case(std::istream& in) {
  int neq;double pivot;in>>neq>>pivot;math_IntegerVector indx(1,neq);int total=0;
  for(int j=1;j<=neq;++j){int h;in>>h;total+=h;indx(j)=total;}
  math_Vector a=read_vector(in,total),b=read_vector(in,neq);
  int status=DACTCL_Decompose(a,indx,pivot);std::cout<<status<<' ';reals(a);
  std::cout<<DACTCL_Solve(a,b,indx,pivot)<<' ';reals(b);
  std::cout<<'\n';
}
// rows cols pivot cells[rows*cols]
void linear_dump_case(std::istream& in) {
  int rows,cols;double pivot;in>>rows>>cols>>pivot;math_Matrix a=read_matrix(in,rows,cols);
  guarded([&]{math_Gauss g(a,pivot);dump_call(g);});
  math_GaussLeastSquare ls(a,pivot);dump_call(ls);
  math_SVD svd(a);dump_call(svd);
  guarded([&]{math_Jacobi j(a);dump_call(j);});
  std::cout<<'\n';
}
}
