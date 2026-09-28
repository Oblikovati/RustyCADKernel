// MathUtils scalar functor oracle at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathUtils_FunctorScalar.hxx>
template<class F>void scalar_functor_value(F f,double x) {
  double y=91;bool ok=f.Value(x,y);std::cout<<ok<<' '<<bits(y)<<' ';
}
template<class F>void scalar_functor_both(F f,double x) {
  scalar_functor_value(f,x);double y=91,dy=92;bool ok=f.Values(x,y,dy);
  std::cout<<ok<<' '<<bits(y)<<' '<<bits(dy)<<' ';
}
void scalar_functor_case(std::istream& input) {
  double x,a,b,c,d;int mode;input>>x>>a>>b>>c>>d>>mode;
  if(mode==1)x=std::numeric_limits<double>::quiet_NaN();
  if(mode==2)a=std::numeric_limits<double>::infinity();
  if(mode==3)b=std::numeric_limits<double>::quiet_NaN();
  MathUtils::Polynomial p({a,b,c,d});
  scalar_functor_both(p,x);std::cout<<p.Degree()<<' '<<bits(p.Coefficient(0))<<' '<<bits(p.Coefficient(8))<<' ';
  scalar_functor_both(MathUtils::Constant(a),x);scalar_functor_both(MathUtils::Linear(a,b),x);
  scalar_functor_both(MathUtils::Sine(a,b,c,d),x);scalar_functor_both(MathUtils::Cosine(a,b,c,d),x);
  scalar_functor_both(MathUtils::Exponential(a,b,c),x);scalar_functor_both(MathUtils::Power(a,b,c),x);
  scalar_functor_both(MathUtils::Gaussian(a,b,c),x);scalar_functor_value(MathUtils::Rational({a,b},{c,d}),x);
  MathUtils::Linear left(a,b),right(c,d);
  scalar_functor_value(MathUtils::MakeComposite(left,right),x);
  scalar_functor_value(MathUtils::MakeSum(left,right),x);scalar_functor_value(MathUtils::MakeDifference(left,right),x);
  scalar_functor_value(MathUtils::MakeProduct(left,right),x);scalar_functor_value(MathUtils::MakeQuotient(left,right),x);
  scalar_functor_value(MathUtils::MakeScaled(left,c),x);scalar_functor_value(MathUtils::MakeShifted(left,c),x);
  scalar_functor_value(MathUtils::MakeNegated(left),x);std::cout<<'\n';
}
