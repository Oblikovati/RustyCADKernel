// Differential coverage of MathUtils at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <MathUtils_Core.hxx>
#include <MathUtils_Bracket.hxx>
#include <MathUtils_Poly.hxx>
#include <vector>
void utils_core_case(std::istream& input) {
  int n; input >> n; math_Vector a(n), b(n); std::vector<double> coefficients(n);
  for(int i=0;i<n;++i) {input >> a.ChangeAt(i);coefficients[i]=a.At(i);}
  for(int i=0;i<n;++i) input >> b.ChangeAt(i);
  for(double value: {MathUtils::DotProduct(a,b),MathUtils::VectorNorm(a),MathUtils::Norm2(a),
      MathUtils::VectorInfNorm(a),MathUtils::ComputeScaleFactor(coefficients.data(),n)}) std::cout<<bits(value)<<' ';
  std::cout<<'\n';
}
struct UtilsProbeFunction {
  int mode; double center; std::vector<double> calls;
  bool Value(double x,double& value) {
    calls.push_back(x);
    value=mode==0?x-center:(x-center)*(x-center);
    if(mode==2 && x>center) return false;
    if(mode==3 && x>center) value=std::numeric_limits<double>::quiet_NaN();
    if(mode==4) value=1;
    return true;
  }
};
void utils_bracket_case(std::istream& input) {
  int mode,limit; double center,a,b,lo,hi; input>>mode>>center>>a>>b>>limit>>lo>>hi;
  UtilsProbeFunction function{mode,center,{}};
  auto root=MathUtils::BracketRoot(function,a,b,limit);
  std::cout<<int(root.Status)<<' '<<root.IsValid<<' '<<root.NbIterations<<' '<<root.HasFA<<' '<<root.HasFB<<' ';
  for(double v:{root.A,root.B,root.Fa,root.Fb}) std::cout<<bits(v)<<' ';
  MathUtils::MinBracketOptions options;options.MaxIterations=limit;
  options.UseLimits=lo!=hi;options.LeftLimit=lo;options.RightLimit=hi;
  auto minimum=MathUtils::BracketMinimum(function,a,b,options);
  std::cout<<int(minimum.Status)<<' '<<minimum.IsValid<<' '<<minimum.NbIterations<<' ';
  for(double v:{minimum.A,minimum.B,minimum.C,minimum.Fa,minimum.Fb,minimum.Fc}) std::cout<<bits(v)<<' ';
  std::cout<<function.calls.size()<<' ';for(double x:function.calls) std::cout<<bits(x)<<' ';std::cout<<'\n';
}
#include <MathUtils_LineSearch.hxx>
#include <MathUtils_Deriv.hxx>
struct UtilsObjective {
  double center;int fail;std::vector<std::pair<int,double>> calls;
  bool Value(const math_Vector& x,double& value) {
    calls.push_back({0,x.At(0)});value=(x.At(0)-center)*(x.At(0)-center);
    return int(calls.size())!=fail;
  }
  bool Gradient(const math_Vector& x,math_Vector& g) {
    calls.push_back({1,x.At(0)});g.ChangeAt(0)=2*(x.At(0)-center);
    return int(calls.size())!=fail;
  }
};
void utils_line_result(const MathUtils::LineSearchResult& r) {
  std::cout<<int(r.Status)<<' '<<r.IsValid<<' '<<bits(r.Alpha)<<' '<<bits(r.FNew)<<' '<<r.NbEvals<<' ';
}
void utils_line_case(std::istream& input) {
  double center,x,d,alpha,tol;int limit,fail;input>>center>>x>>d>>alpha>>tol>>limit>>fail;
  math_Vector point(4,4,x),direction(-1,-1,d),gradient(7,7,2*(x-center));
  UtilsObjective f{center,fail,{}};double fx=(x-center)*(x-center);
  utils_line_result(MathUtils::ArmijoBacktrack(f,point,direction,gradient,fx,alpha,1e-4,.5,limit));
  utils_line_result(MathUtils::WolfeSearch(f,point,direction,gradient,fx,alpha,1e-4,.9,limit));
  utils_line_result(MathUtils::ExactLineSearch(f,point,direction,alpha,tol,limit));
  size_t count=7;bool improved=MathUtils::BrentAlongCoordinate(f,point,0,-100,100,fx,tol,limit,count);
  std::cout<<improved<<' '<<bits(point.At(0))<<' '<<bits(fx)<<' '<<count<<' ';
  math_Vector grad(-2,-2,91);bool ok=MathUtils::NumericalGradientAdaptive(f,point,grad,tol);
  std::cout<<ok<<' '<<bits(grad.At(0))<<' '<<bits(point.At(0))<<' ';
  math_Matrix hessian(3,3,-2,-2,92);ok=MathUtils::NumericalHessian(f,point,hessian,tol);
  std::cout<<ok<<' '<<bits(hessian.At(0,0))<<' '<<bits(point.At(0))<<' ';
  std::cout<<f.calls.size()<<' ';for(const auto& c:f.calls)std::cout<<c.first<<' '<<bits(c.second)<<' ';std::cout<<'\n';
}
