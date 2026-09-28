// MathUtils_FunctorScalar / MathUtils_FunctorVector API-completion oracle at
// 3d097a0328e71b826377d4814ab05ec3c3d23871. Complements scalar_functor/vector_functor with
// variable-length coefficient vectors (non-default bounds, empty, single coefficient), failing
// operands inside the arithmetic combinators, the lambda wrappers and N-dimensional functors.
#include <MathUtils_FunctorScalar.hxx>
#include <MathUtils_FunctorVector.hxx>

namespace mathutils_functor_api
{
const double THE_NAN = std::numeric_limits<double>::quiet_NaN();
const double THE_INF = std::numeric_limits<double>::infinity();

math_Vector read_vector(std::istream& theInput, int theLower, int theSize)
{
  math_Vector aResult = theSize == 0 ? math_Vector(size_t{0}) : math_Vector(theLower, theLower + theSize - 1);
  for (int anIndex = 0; anIndex < theSize; ++anIndex)
  {
    theInput >> aResult.ChangeAt(static_cast<size_t>(anIndex));
  }
  return aResult;
}

template <class F>
void value(const F& theF, double theX)
{
  double aY = 91;
  const bool anOk = theF.Value(theX, aY);
  std::cout << anOk << ' ' << bits(aY) << ' ';
}

template <class F>
void values(const F& theF, double theX)
{
  double aY = 91, aDY = 92;
  const bool anOk = theF.Values(theX, aY, aDY);
  std::cout << anOk << ' ' << bits(aY) << ' ' << bits(aDY) << ' ';
}

void print_vector(const math_Vector& theV)
{
  for (size_t anIndex = 0; anIndex < theV.Size(); ++anIndex)
  {
    std::cout << bits(theV.At(anIndex)) << ' ';
  }
}

template <class F>
void nd_value(const F& theF, const math_Vector& theX)
{
  double aY = 91;
  const bool anOk = theF.Value(theX, aY);
  std::cout << anOk << ' ' << bits(aY) << ' ';
}

template <class F>
void nd_all(const F& theF, const math_Vector& theX)
{
  nd_value(theF, theX);
  math_Vector aGradient(1, static_cast<int>(theX.Size()) + 1, 92.0);
  bool anOk = theF.Gradient(theX, aGradient);
  std::cout << anOk << ' ';
  print_vector(aGradient);
  double aY = 71;
  aGradient.Init(72.0);
  anOk = theF.Values(theX, aY, aGradient);
  std::cout << anOk << ' ' << bits(aY) << ' ';
  print_vector(aGradient);
}
} // namespace mathutils_functor_api

//! mathutils_functor_poly lower n k mode x c[0..n): Polynomial(math_Vector) on all n
//! coefficients, Rational(c[0..k), c[k..n)); mode 1 x=NaN, 2 c0=+inf, 3 x=+inf.
void mathutils_functor_poly_case(std::istream& input)
{
  using namespace mathutils_functor_api;
  int aLower, aSize, aSplit, aMode;
  double aX;
  input >> aLower >> aSize >> aSplit >> aMode >> aX;
  math_Vector aAll = read_vector(input, aLower, aSize);
  if (aMode == 1) aX = THE_NAN;
  if (aMode == 2 && aSize > 0) aAll.ChangeAt(0) = THE_INF;
  if (aMode == 3) aX = THE_INF;
  const MathUtils::Polynomial aPoly(aAll);
  value(aPoly, aX);
  values(aPoly, aX);
  std::cout << aPoly.Degree() << ' ';
  for (size_t anIndex = 0; anIndex <= static_cast<size_t>(aSize); ++anIndex)
  {
    std::cout << bits(aPoly.Coefficient(anIndex)) << ' ';
  }
  math_Vector aNum = aSplit == 0 ? math_Vector(size_t{0}) : math_Vector(aLower + 3, aLower + 2 + aSplit);
  math_Vector aDen = aSize == aSplit ? math_Vector(size_t{0}) : math_Vector(-aLower, -aLower + aSize - aSplit - 1);
  for (int anIndex = 0; anIndex < aSize; ++anIndex)
  {
    (anIndex < aSplit ? aNum.ChangeAt(static_cast<size_t>(anIndex))
                      : aDen.ChangeAt(static_cast<size_t>(anIndex - aSplit))) = aAll.At(static_cast<size_t>(anIndex));
  }
  value(MathUtils::Rational(aNum, aDen), aX);
  std::cout << '\n';
}

//! mathutils_functor_arith mode x a b c d e k: f = Gaussian(a,b,c), g = Polynomial{d,e};
//! mode 1 x=NaN, 2 c=0 (f fails), 3 d=e=0 (g is zero), 4 k=+inf.
void mathutils_functor_arith_case(std::istream& input)
{
  using namespace mathutils_functor_api;
  int aMode;
  double aX, aA, aB, aC, aD, aE, aK;
  input >> aMode >> aX >> aA >> aB >> aC >> aD >> aE >> aK;
  if (aMode == 1) aX = THE_NAN;
  if (aMode == 2) aC = 0.0;
  if (aMode == 3) aD = aE = 0.0;
  if (aMode == 4) aK = THE_INF;
  const MathUtils::Gaussian   aF(aA, aB, aC);
  const MathUtils::Polynomial aG({aD, aE});
  value(MathUtils::Composite<MathUtils::Gaussian, MathUtils::Polynomial>(aF, aG), aX);
  value(MathUtils::MakeComposite(aG, aF), aX);
  value(MathUtils::Sum<MathUtils::Gaussian, MathUtils::Polynomial>(aF, aG), aX);
  value(MathUtils::MakeSum(aG, aF), aX);
  value(MathUtils::Difference<MathUtils::Gaussian, MathUtils::Polynomial>(aF, aG), aX);
  value(MathUtils::MakeDifference(aG, aF), aX);
  value(MathUtils::Product<MathUtils::Polynomial, MathUtils::Gaussian>(aG, aF), aX);
  value(MathUtils::MakeProduct(aF, aG), aX);
  value(MathUtils::Quotient<MathUtils::Gaussian, MathUtils::Polynomial>(aF, aG), aX);
  value(MathUtils::MakeQuotient(aG, aF), aX);
  value(MathUtils::Scaled<MathUtils::Gaussian>(aF, aK), aX);
  value(MathUtils::MakeScaled(aG, aK), aX);
  value(MathUtils::Shifted<MathUtils::Polynomial>(aG, aK), aX);
  value(MathUtils::MakeShifted(aF, aK), aX);
  value(MathUtils::Negated<MathUtils::Gaussian>(aF), aX);
  value(MathUtils::MakeNegated(aG), aX);
  std::cout << '\n';
}

//! mathutils_functor_lambda x p: scalar lambda wrappers whose callables fail on some inputs
//! but still write their outputs first.
void mathutils_functor_lambda_case(std::istream& input)
{
  using namespace mathutils_functor_api;
  double aX, aP;
  input >> aX >> aP;
  auto aSquare = [aP](double theX, double& theY) {
    theY = aP * theX * theX - 2.0;
    return theX >= 0.0;
  };
  auto aCube = [aP](double theX, double& theY, double& theDY) {
    theY  = aP * theX * theX * theX;
    theDY = 3.0 * aP * theX * theX;
    return theX != 1.0;
  };
  value(MathUtils::ScalarLambda<decltype(aSquare)>(aSquare), aX);
  value(MathUtils::MakeScalar(aSquare), aX);
  const MathUtils::ScalarLambdaWithDerivative<decltype(aCube)> aWith(aCube);
  value(aWith, aX);
  values(aWith, aX);
  values(MathUtils::MakeScalarWithDerivative(aCube), aX);
  std::cout << '\n';
}

//! mathutils_functor_vlambda lower n x[0..n): VectorLambda, VectorLambdaWithGradient and
//! SystemLambda over callables that fail on the sign of the first/last coordinate.
void mathutils_functor_vlambda_case(std::istream& input)
{
  using namespace mathutils_functor_api;
  int aLower, aSize;
  input >> aLower >> aSize;
  const math_Vector aX = read_vector(input, aLower, aSize);
  auto aValue = [](const math_Vector& theX, double& theY) {
    theY = 0.0;
    for (size_t anIndex = 0; anIndex < theX.Size(); ++anIndex)
    {
      theY += theX.At(anIndex) * theX.At(anIndex);
    }
    theY += theX.At(0);
    return theX.At(0) >= 0.0;
  };
  auto aGradient = [](const math_Vector& theX, math_Vector& theG) {
    for (size_t anIndex = 0; anIndex < theX.Size(); ++anIndex)
    {
      theG.ChangeAt(anIndex) = 2.0 * theX.At(anIndex);
    }
    return theX.At(theX.Size() - 1) >= 0.0;
  };
  auto aSystem = [](const math_Vector& theX, math_Vector& theF) {
    for (size_t anIndex = 0; anIndex < theX.Size(); ++anIndex)
    {
      theF.ChangeAt(anIndex) = theX.At(anIndex) * theX.At(anIndex) - static_cast<double>(anIndex);
    }
    return theX.Size() > 1;
  };
  nd_value(MathUtils::VectorLambda<decltype(aValue)>(aValue), aX);
  nd_value(MathUtils::MakeVector(aValue), aX);
  nd_all(MathUtils::VectorLambdaWithGradient<decltype(aValue), decltype(aGradient)>(aValue, aGradient), aX);
  nd_all(MathUtils::MakeVectorWithGradient(aValue, aGradient), aX);
  const MathUtils::SystemLambda<decltype(aSystem)> aSys(aSystem, static_cast<size_t>(aSize));
  const auto aMade = MathUtils::MakeSystem(aSystem, static_cast<size_t>(aSize) + 1);
  math_Vector aF(-1, aSize - 2, 93.0);
  bool anOk = aSys.Value(aX, aF);
  std::cout << aSys.NbEquations() << ' ' << aMade.NbEquations() << ' ' << anOk << ' ';
  print_vector(aF);
  aF.Init(94.0);
  anOk = aMade.Value(aX, aF);
  std::cout << anOk << ' ';
  print_vector(aF);
  std::cout << '\n';
}

//! mathutils_functor_nd lower n m mode a b c x[0..n) A[m*n row-major] rhs[m]: Sphere,
//! Rastrigin, Ackley (n may be 0), LinearResidual (m x n) and QuadraticForm on the leading
//! n x n block when m >= n. mode 1 x0=NaN, 2 a=+inf, 3 x0=+inf, 4 c=NaN.
void mathutils_functor_nd_case(std::istream& input)
{
  using namespace mathutils_functor_api;
  int aLower, aSize, aRows, aMode;
  double aA, aB, aC;
  input >> aLower >> aSize >> aRows >> aMode >> aA >> aB >> aC;
  math_Vector aX = read_vector(input, aLower, aSize);
  if (aMode == 1 && aSize > 0) aX.ChangeAt(0) = THE_NAN;
  if (aMode == 2) aA = THE_INF;
  if (aMode == 3 && aSize > 0) aX.ChangeAt(0) = THE_INF;
  if (aMode == 4) aC = THE_NAN;
  nd_all(MathUtils::Sphere(), aX);
  nd_all(MathUtils::Rastrigin(aA), aX);
  nd_value(MathUtils::Ackley(aA, aB, aC), aX);
  nd_value(MathUtils::Ackley(), aX);
  nd_value(MathUtils::Rastrigin(), aX);
  if (aSize > 0 && aRows > 0)
  {
    math_Matrix aMatrix(aLower - 2, aLower + aRows - 3, aLower + 5, aLower + aSize + 4);
    for (int aRow = 0; aRow < aRows; ++aRow)
      for (int aCol = 0; aCol < aSize; ++aCol)
        input >> aMatrix.ChangeAt(static_cast<size_t>(aRow), static_cast<size_t>(aCol));
    const math_Vector aRhs = read_vector(input, aLower + 7, aRows);
    nd_all(MathUtils::LinearResidual(aMatrix, aRhs), aX);
    if (aRows >= aSize)
    {
      nd_all(MathUtils::QuadraticForm(aMatrix, aRhs, aC), aX);
    }
  }
  std::cout << '\n';
}
