// math_IntegerVector instantiation, vector/matrix dumps and the remaining math_Matrix members.
#include <math_IntegerVector.hxx>
#include <math_Vector.hxx>
#include <math_Matrix.hxx>
#include <sstream>
namespace vector_base_probe {
template<class V> void values(const V& vector) {
  std::cout<<vector.Lower()<<' '<<vector.Upper()<<' '<<vector.Length()<<' ';
  for(int i=vector.Lower();i<=vector.Upper();++i)std::cout<<vector(i)<<' ';
}
void reals(const math_Vector& vector) {
  std::cout<<vector.Lower()<<' '<<vector.Upper()<<' '<<vector.Length()<<' ';
  for(int i=vector.Lower();i<=vector.Upper();++i)std::cout<<bits(vector(i))<<' ';
}
void cells(const math_Matrix& matrix) {
  std::cout<<matrix.LowerRow()<<' '<<matrix.UpperRow()<<' '<<matrix.LowerCol()<<' '<<matrix.UpperCol()<<' ';
  for(int r=matrix.LowerRow();r<=matrix.UpperRow();++r)for(int c=matrix.LowerCol();c<=matrix.UpperCol();++c)std::cout<<bits(matrix(r,c))<<' ';
}
template<class T> void dumped(const T& value) {
  std::ostringstream text;text<<value;std::string s=text.str();
  for(char& ch:s)if(ch=='\n')ch='|';
  std::cout<<s<<' ';
}
template<class F> void guarded(F&& body) {
  try {body();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
// n lower v[n] scalar divisor m[n*2]
void int_vector_case(std::istream& in) {
  int n,lower;in>>n>>lower;math_IntegerVector v(lower,lower+n-1);
  for(int i=lower;i<=v.Upper();++i){double x;in>>x;v(i)=int(x);}
  double s,d;in>>s>>d;math_Matrix m(1,n,1,2);for(int r=1;r<=n;++r)for(int c=1;c<=2;++c)in>>m(r,c);
  std::cout<<bits(v.Norm())<<' '<<bits(v.Norm2())<<' '<<v.Max()<<' '<<v.Min()<<' ';
  guarded([&]{values(v.Normalized());});
  guarded([&]{values(v.Inverse());});values(v.Multiplied(int(s)));values(v.TMultiplied(int(s)));values(int(s)*v);
  guarded([&]{values(v.Divided(int(d)));});
  guarded([&]{math_IntegerVector w(v);w.Divide(int(d));values(w);});
  values(v.Opposite());values(v.Added(v.Multiplied(2)));values(v.Subtracted(v.Multiplied(2)));std::cout<<v.Multiplied(v.Multiplied(2))<<' ';
  math_IntegerVector w(3,3+n-1);guarded([&]{w.Multiply(int(s),v);values(w);});w.Add(v,v.Multiplied(2));values(w);w.Subtract(v,v.Opposite());values(w);
  math_IntegerVector p(4,5);p.Multiply(v,m);values(p);p.TMultiply(m,v);values(p);values(v.Multiplied(m));
  math_Matrix t=m.Transposed();p.Multiply(t,v);values(p);p.TMultiply(v,t);values(p);
  values(v.Slice(v.Upper(),v.Lower()));dumped(v);std::cout<<v.Array1().Lower()<<' '<<v.Array1().Upper()<<' '<<v.Size()<<' ';
  math_IntegerVector sized(static_cast<size_t>(n),static_cast<int>(s));values(sized);math_IntegerVector xy(gp_XY(s,d));values(xy);
  std::cout<<'\n';
}
// n lower v[n] scalar
void real_vector_extra_case(std::istream& in) {
  int n,lower;in>>n>>lower;math_Vector v(lower,lower+n-1);for(int i=lower;i<=v.Upper();++i)in>>v(i);
  double s;in>>s;reals(v.TMultiplied(s));reals(s*v);dumped(v);
  std::cout<<v.Array1().Lower()<<' '<<v.Array1().Upper()<<' '<<v.Size()<<' ';
  math_Vector sized(size_t(n),s);reals(sized);math_Vector xyz(gp_XYZ(s,-s,2*s));reals(xyz);
  std::cout<<'\n';
}
// r c rl cl cells[r*c] scalar block_row block_col
void matrix_extra_case(std::istream& in) {
  int r,c,rl,cl,br,bc;in>>r>>c>>rl>>cl;math_Matrix m(rl,rl+r-1,cl,cl+c-1);
  for(int i=0;i<r;++i)for(int j=0;j<c;++j)in>>m.ChangeAt(i,j);double s;in>>s>>br>>bc;
  cells(m.Opposite());cells(-m);cells(m.TMultiplied(s));cells(s*m);cells(m*s);
  math_Matrix a(m);a.Add(m.Opposite());cells(a);a.Subtract(m);cells(a);guarded([&]{a.Add(m.Transposed());cells(a);});
  math_Matrix sum(-1,-2+r,5,4+c);sum.Add(m,m.Multiplied(s));cells(sum);sum.Subtract(m,m.Multiplied(s));cells(sum);
  guarded([&]{sum.Add(m,m.Transposed());});
  cells(m.TMultiply(m));guarded([&]{cells(m.TMultiply(m.Transposed()));});
  math_Matrix prod(2,1+r,-3,-4+r);prod.Multiply(m,m.Transposed());cells(prod);
  guarded([&]{prod.Multiply(m,m);cells(prod);});
  math_Matrix tprod(0,c-1,7,6+c);tprod.TMultiply(m,m);cells(tprod);guarded([&]{tprod.TMultiply(m.Transposed(),m);});
  math_Matrix block(1,1,1,1,s);math_Matrix target(m);
  guarded([&]{target.Set(br,br,bc,bc,block);});cells(target);guarded([&]{target.Set(br,br+1,bc,bc,block);});
  math_Matrix swap(m);guarded([&]{swap.Transpose();});cells(swap);
  math_Matrix sized(size_t(r),size_t(c),s);cells(sized);std::cout<<m.RowSize()<<' '<<m.ColSize()<<' ';
  math_Matrix rescaled(m);guarded([&]{rescaled/=s;});cells(rescaled);
  dumped(m);std::cout<<'\n';
}
}
