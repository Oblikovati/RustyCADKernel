// MathUtils vector functor oracle at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathUtils_FunctorVector.hxx>
template<class F>void vector_functor_both(F f,const math_Vector& x) {
  double value=91;math_Vector gradient(-2,-1,92);bool ok=f.Value(x,value);
  std::cout<<ok<<' '<<bits(value)<<' ';ok=f.Gradient(x,gradient);
  std::cout<<ok<<' '<<bits(gradient.At(0))<<' '<<bits(gradient.At(1))<<' ';
  value=71;gradient.Init(72);ok=f.Values(x,value,gradient);
  std::cout<<ok<<' '<<bits(value)<<' '<<bits(gradient.At(0))<<' '<<bits(gradient.At(1))<<' ';
}
void vector_functor_case(std::istream& input) {
  double x,y,a,b,c,d;int mode;input>>x>>y>>a>>b>>c>>d>>mode;
  if(mode==1)x=std::numeric_limits<double>::quiet_NaN();
  if(mode==2)a=std::numeric_limits<double>::infinity();
  math_Vector point(4,5);point(4)=x;point(5)=y;
  vector_functor_both(MathUtils::Rosenbrock(a,b),point);vector_functor_both(MathUtils::Booth(),point);
  vector_functor_both(MathUtils::Beale(),point);vector_functor_both(MathUtils::Himmelblau(),point);
  vector_functor_both(MathUtils::Sphere(),point);vector_functor_both(MathUtils::Rastrigin(a),point);
  math_Matrix matrix(-1,0,3,4);matrix(-1,3)=a;matrix(-1,4)=b;matrix(0,3)=c;matrix(0,4)=d;
  math_Vector rhs(7,8);rhs(7)=a;rhs(8)=b;
  vector_functor_both(MathUtils::QuadraticForm(matrix,rhs,c),point);
  vector_functor_both(MathUtils::LinearResidual(matrix,rhs),point);
  MathUtils::Ackley ackley(a,b,c);double value=91;bool ok=ackley.Value(point,value);
  std::cout<<ok<<' '<<bits(value)<<' ';value=91;ok=ackley.Value(math_Vector(size_t{0}),value);
  std::cout<<ok<<' '<<bits(value)<<'\n';
}
