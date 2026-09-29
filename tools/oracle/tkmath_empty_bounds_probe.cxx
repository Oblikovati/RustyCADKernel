// Empty bounds (upper = lower - 1) of math_DoubleTab, math_Matrix and math_VectorBase:
// every constructor and member over empty, one-element and two-element ranges, and the
// solvers and decompositions over no variable at all.
#include <math_BFGS.hxx>
#include <math_Crout.hxx>
#include <math_DoubleTab.hxx>
#include <math_FRPR.hxx>
#include <math_FunctionSetRoot.hxx>
#include <math_FunctionSetWithDerivatives.hxx>
#include <math_Gauss.hxx>
#include <math_GaussLeastSquare.hxx>
#include <math_Householder.hxx>
#include <math_IntegerVector.hxx>
#include <math_Jacobi.hxx>
#include <math_Matrix.hxx>
#include <math_MultipleVarFunctionWithGradient.hxx>
#include <math_NewtonFunctionSetRoot.hxx>
#include <math_Powell.hxx>
#include <math_SVD.hxx>
#include <math_Uzawa.hxx>
#include <math_Vector.hxx>
namespace empty_bounds_probe {
using vector_base_probe::cells;
using vector_base_probe::dumped;
using vector_base_probe::guarded;
using vector_base_probe::reals;
using vector_base_probe::values;
void table(const math_DoubleTab& t) {
  std::cout<<t.LowerRow()<<' '<<t.UpperRow()<<' '<<t.LowerCol()<<' '<<t.UpperCol()<<' '<<t.NbRows()<<' '
           <<t.NbColumns()<<' '<<t.RowSize()<<' '<<t.ColSize()<<' '<<t.IsDeletable()<<' ';
}
void double_tab_part(int rl,int ru,int cl,int cu,double s) {
  math_DoubleTab t(rl,ru,cl,cu);table(t);t.Init(s);
  math_DoubleTab copy(t);table(copy);
  math_DoubleTab other(rl,ru,cl,cu);guarded([&]{t.Copy(other);table(other);});
  guarded([&]{std::cout<<bits(t.Value(rl,cl))<<' ';});
  guarded([&]{std::cout<<bits(t.At(0,0))<<' ';});
  math_DoubleTab moved(std::move(copy));table(moved);
  math_DoubleTab zero(size_t(ru-rl+1),size_t(cu-cl+1));table(zero);
  double buffer[4]={s,s,s,s};math_DoubleTab external(buffer,rl,ru,cl,cu);table(external);
}
void matrix_values(const math_Matrix& m,int rl,int ru,int cl,int cu,double s) {
  cells(m.Multiplied(s));cells(m.Opposite());cells(m.Transposed());guarded([&]{cells(m.Divided(s));});
  math_Matrix a(m);a.Multiply(s);guarded([&]{a.Add(m);});guarded([&]{a.Subtract(m);});cells(a);
  guarded([&]{cells(m.Added(m));});guarded([&]{cells(m.Subtracted(m));});
  guarded([&]{cells(m.Multiplied(m.Transposed()));});guarded([&]{cells(m.TMultiply(m));});
  math_Matrix product(rl,ru,rl,ru);guarded([&]{product.Multiply(m,m.Transposed());cells(product);});
  math_Matrix tproduct(cl,cu,cl,cu);guarded([&]{tproduct.TMultiply(m,m);cells(tproduct);});
  guarded([&]{math_Matrix d(m);d.SetDiag(s);cells(d);});
  guarded([&]{math_Matrix t(m);t.Transpose();cells(t);});
  guarded([&]{math_Matrix w(m);w.Set(rl,ru,cl,cu,m);cells(w);});
}
void matrix_rows(const math_Matrix& m,int rl,int ru,int cl,int cu,double s) {
  guarded([&]{reals(m.Row(rl));});guarded([&]{reals(m.Col(cl));});
  math_Vector row(cl,cu,s),col(rl,ru,-s);
  guarded([&]{math_Matrix w(m);w.SetRow(rl,row);cells(w);});
  guarded([&]{math_Matrix w(m);w.SetCol(cl,col);cells(w);});
  guarded([&]{math_Matrix w(m);w.SwapRow(rl,ru);cells(w);});
  guarded([&]{math_Matrix w(m);w.SwapCol(cl,cu);cells(w);});
  guarded([&]{reals(m.Multiplied(row));});
  guarded([&]{math_Matrix w(m);w.Multiply(col,row);cells(w);});
}
void matrix_solvers(const math_Matrix& m) {
  guarded([&]{std::cout<<bits(m.Determinant())<<' ';});guarded([&]{cells(m.Inverse());});
  guarded([&]{math_Gauss g(m);std::cout<<g.IsDone()<<' '<<bits(g.Determinant())<<' ';});
  guarded([&]{math_Crout c(m);std::cout<<c.IsDone()<<' '<<bits(c.Determinant())<<' ';cells(c.Inverse());});
}
void matrix_part(int rl,int ru,int cl,int cu,double s) {
  math_Matrix m(rl,ru,cl,cu,s);cells(m);std::cout<<m.RowNumber()<<' '<<m.ColNumber()<<' ';
  math_Matrix plain(rl,ru,cl,cu);std::cout<<plain.RowNumber()<<' '<<plain.ColNumber()<<' ';
  matrix_values(m,rl,ru,cl,cu,s);matrix_rows(m,rl,ru,cl,cu,s);matrix_solvers(m);
  math_Matrix sized(size_t(ru-rl+1),size_t(cu-cl+1),s);cells(sized);
  double buffer[4]={s,-s,s,-s};math_Matrix external(buffer,rl,ru,cl,cu);cells(external);
  math_Matrix moved(std::move(m));cells(moved);dumped(moved);
}
void vector_part(int vl,int vu,double s) {
  math_Vector v(vl,vu,s);reals(v);
  guarded([&]{std::cout<<bits(v.Norm())<<' ';});guarded([&]{std::cout<<bits(v.Norm2())<<' ';});
  std::cout<<v.Max()<<' '<<v.Min()<<' ';
  reals(v.Opposite());reals(v.Multiplied(s));guarded([&]{reals(v.Divided(s));});
  guarded([&]{reals(v.Normalized());});guarded([&]{math_Vector w(v);w.Normalize();reals(w);});
  guarded([&]{reals(v.Inverse());});guarded([&]{math_Vector w(v);w.Invert();reals(w);});
  guarded([&]{reals(v.Added(v));});guarded([&]{reals(v.Subtracted(v));});
  guarded([&]{std::cout<<bits(v.Multiplied(v))<<' ';});
  guarded([&]{reals(v.Slice(vl,vu));});guarded([&]{math_Vector w(v);w.Set(vl,vu,v);reals(w);});
  math_Vector sized(size_t(vu-vl+1),s);reals(sized);
  double buffer[2]={s,-s};math_Vector external(buffer,vl,vu);reals(external);
  math_Vector moved(std::move(v));reals(moved);dumped(moved);
  math_IntegerVector iv(vl,vu,int(s));values(iv);
  guarded([&]{std::cout<<bits(iv.Norm())<<' ';});std::cout<<iv.Max()<<' '<<iv.Min()<<' ';
  guarded([&]{values(iv.Added(iv));});guarded([&]{std::cout<<iv.Multiplied(iv)<<' ';});
}
// Objective and equation set without any variable; `value` is returned, `ok` is the
// callback status.
struct NoVariable : math_MultipleVarFunctionWithGradient {
  double value;bool ok;
  NoVariable(double v,bool o):value(v),ok(o){}
  int NbVariables() const override {return 0;}
  bool Value(const math_Vector&,double& v) override {v=value;return ok;}
  bool Gradient(const math_Vector&,math_Vector&) override {return ok;}
  bool Values(const math_Vector&,double& v,math_Vector&) override {v=value;return ok;}
};
struct NoEquation : math_FunctionSetWithDerivatives {
  bool ok;
  explicit NoEquation(bool o):ok(o){}
  int NbVariables() const override {return 0;}
  int NbEquations() const override {return 0;}
  bool Value(const math_Vector&,math_Vector&) override {return ok;}
  bool Derivatives(const math_Vector&,math_Matrix&) override {return ok;}
  bool Values(const math_Vector&,math_Vector&,math_Matrix&) override {return ok;}
};
template<class Solver> void minimizer(const Solver& s) {
  std::cout<<s.IsDone()<<' ';
  guarded([&]{std::cout<<bits(s.Minimum())<<' ';});guarded([&]{std::cout<<s.NbIterations()<<' ';});
}
void empty_minimizers(double value,bool ok) {
  math_Vector start(1,0);
  guarded([&]{NoVariable f(value,ok);math_BFGS s(0);s.Perform(f,start);minimizer(s);});
  guarded([&]{NoVariable f(value,ok);math_FRPR s(f,1e-8);s.Perform(f,start);minimizer(s);});
  guarded([&]{NoVariable f(value,ok);math_Powell s(f,1e-8);math_Matrix d(1,0,1,0);s.Perform(f,start,d);minimizer(s);});
  math_Vector tolerance(1,0);
  guarded([&]{NoEquation f(ok);math_FunctionSetRoot s(f,tolerance);s.Perform(f,start);std::cout<<s.IsDone()<<' ';});
  guarded([&]{NoEquation f(ok);math_NewtonFunctionSetRoot s(f,tolerance,1e-8);s.Perform(f,start);std::cout<<s.IsDone()<<' ';});
}
void empty_decompositions() {
  math_Matrix e00(1,0,1,0),e02(1,0,1,2);math_Vector none(1,0);
  guarded([&]{math_Jacobi s(e00);std::cout<<s.IsDone()<<' ';reals(s.Values());});
  guarded([&]{math_SVD s(e00);std::cout<<s.IsDone()<<' ';});
  guarded([&]{math_SVD s(e02);std::cout<<s.IsDone()<<' ';});
  guarded([&]{math_GaussLeastSquare s(e00);std::cout<<s.IsDone()<<' ';});
  guarded([&]{math_GaussLeastSquare s(e02);std::cout<<s.IsDone()<<' ';});
  guarded([&]{math_Householder s(e00,e00,1e-20);std::cout<<s.IsDone()<<' ';cells(s.AllValues());});
  guarded([&]{math_Uzawa s(e00,none,none);std::cout<<s.IsDone()<<' ';reals(s.Value());});
}
// value ok: the solvers and decompositions over no variable at all.
void empty_solvers_case(std::istream& in) {
  double value;int ok;in>>value>>ok;
  empty_minimizers(value,ok!=0);empty_decompositions();
  std::cout<<'\n';
}
// rl ru cl cu vl vu scalar
void empty_bounds_case(std::istream& in) {
  int rl,ru,cl,cu,vl,vu;double s;in>>rl>>ru>>cl>>cu>>vl>>vu>>s;
  double_tab_part(rl,ru,cl,cu,s);matrix_part(rl,ru,cl,cu,s);vector_part(vl,vu,s);
  std::cout<<'\n';
}
}
