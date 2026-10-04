#include <math_VectorBase.hxx>
#include <iostream>
#include <vector>

class ExposedVector : public math_VectorBase<double>
{
public:
  using math_VectorBase<double>::math_VectorBase;
  using math_VectorBase<double>::SetLower;
};

int main()
{
  double source[34];
  for (int i = 0; i < 34; ++i) source[i] = i + 0.25;
  ExposedVector small(source, -2, 0);
  ExposedVector large(source, 4, 36);
  source[0] = -9.0;
  small.SetLower(5);
  large.SetLower(-10);
  small(6) = 77.0;
  std::cout << small.Lower() << ' ' << small.Upper() << ' ' << small(5) << ' '
            << small(7) << ' ' << large.Lower() << ' ' << large.Upper() << ' '
            << large(-10) << ' ' << large(-9) << ' ' << large(22) << ' '
            << source[1] << '\n';
}
