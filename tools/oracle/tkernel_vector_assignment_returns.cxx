#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <iostream>

template <typename Vector> static void show_return(const Vector& destination, bool same)
{
  std::cout << Vector::Length() << ' ' << same;
  for (int index = 0; index < Vector::Length(); ++index)
    std::cout << ' ' << destination[index];
  std::cout << '\n';
}

template <typename Vector> static void assignment_returns()
{
  Vector destination(12);
  const Vector right(3);
  show_return(destination, &(destination += right) == &destination);
  show_return(destination, &(destination -= right) == &destination);
  show_return(destination, &(destination *= right) == &destination);
  show_return(destination, &(destination /= right) == &destination);
}

int main()
{
  assignment_returns<NCollection_Vec2<int>>();
  assignment_returns<NCollection_Vec3<int>>();
  assignment_returns<NCollection_Vec4<int>>();
}
