#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <cmath>
#include <iostream>

template <typename Element> static void cancellation_products()
{
  const Element large = Element(1.0e20);
  const NCollection_Vec3<Element> triple(large, -large, Element(1));
  const NCollection_Vec4<Element> quadruple(large, -large, Element(1), Element(2));
  std::cout << "dot f" << sizeof(Element) * 8 << ' '
            << triple.Dot(NCollection_Vec3<Element>(Element(1))) << ' '
            << quadruple.Dot(NCollection_Vec4<Element>(Element(1))) << '\n';
}

static void cross_and_extrapolation()
{
  using Vector = NCollection_Vec3<int>;
  const Vector first(1, 2, 3), second(4, 5, 6);
  for (const Vector& result : {Vector::Cross(first, second), Vector::Cross(second, first),
                              Vector::GetLERP(first, second, 2),
                              Vector::GetLERP(first, second, -1)})
  {
    std::cout << "components " << result.x() << ' ' << result.y() << ' ' << result.z() << '\n';
  }
}

int main()
{
  const NCollection_Vec2<double> zeros(-0.0), ones(1.0);
  std::cout << "dot zero " << std::signbit(zeros.Dot(ones)) << '\n';
  cancellation_products<float>();
  cancellation_products<double>();
  cross_and_extrapolation();
  const NCollection_Vec3<double> finite(2.0), infinite(INFINITY);
  const auto endpoint = NCollection_Vec3<double>::GetLERP(finite, infinite, 0.0);
  std::cout << "endpoint " << std::isnan(endpoint.x()) << ' ' << std::isnan(endpoint.y())
            << ' ' << std::isnan(endpoint.z()) << '\n';
}
