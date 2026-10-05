#include <NCollection_Vec2.hxx>
#include <iostream>

struct Tagged
{
  int value;
  constexpr Tagged(int input = 0) : value(input) {}
  constexpr Tagged operator/(Tagged divisor) const
  {
    return Tagged(1000 + value * 10 + divisor.value);
  }
  constexpr Tagged& operator/=(Tagged divisor)
  {
    value = 2000 + value * 10 + divisor.value;
    return *this;
  }
};

int main()
{
  const NCollection_Vec2<Tagged> initial(Tagged(2), Tagged(4));
  const auto divided = initial / Tagged(3);
  auto assigned = initial;
  assigned /= Tagged(3);
  std::cout << divided.x().value << ' ' << divided.y().value << '\n';
  std::cout << assigned.x().value << ' ' << assigned.y().value << '\n';
}
