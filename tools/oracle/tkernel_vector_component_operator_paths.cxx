#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <iostream>
#include <string>

struct Tagged
{
  int value;
  constexpr Tagged(int input = 0) : value(input) {}
  constexpr Tagged operator+(Tagged right) const { return Tagged(1000 + value * 10 + right.value); }
  constexpr Tagged operator-(Tagged right) const { return Tagged(3000 + value * 10 + right.value); }
  constexpr Tagged operator*(Tagged right) const { return Tagged(5000 + value * 10 + right.value); }
  constexpr Tagged operator/(Tagged right) const { return Tagged(7000 + value * 10 + right.value); }
  constexpr Tagged& operator+=(Tagged right) { value = 2000 + value * 10 + right.value; return *this; }
  constexpr Tagged& operator-=(Tagged right) { value = 4000 + value * 10 + right.value; return *this; }
  constexpr Tagged& operator*=(Tagged right) { value = 6000 + value * 10 + right.value; return *this; }
  constexpr Tagged& operator/=(Tagged right) { value = 8000 + value * 10 + right.value; return *this; }
};

std::ostream& operator<<(std::ostream& stream, Tagged value) { return stream << value.value; }

template<class Vector>
void print(const char* label, const Vector& vector)
{
  std::cout << label;
  for (int index = 0; index < Vector::Length(); ++index) {
    std::cout << (index == 0 ? " " : ",") << vector[index].value;
  }
  std::cout << '\n';
}

template<class Vector>
void values(const char* dimension, const Vector& left, const Vector& right)
{
  print((std::string(dimension) + "+").c_str(), left + right);
  print((std::string(dimension) + "-").c_str(), left - right);
  print((std::string(dimension) + "*").c_str(), left * right);
  print((std::string(dimension) + "/").c_str(), left / right);
}

template<class Vector>
void assignments(const char* dimension, Vector left, const Vector& right)
{
  const Vector original = left;
  left += right; print((std::string(dimension) + "+=").c_str(), left);
  left = original;
  left -= right; print((std::string(dimension) + "-=").c_str(), left);
  left = original;
  left *= right; print((std::string(dimension) + "*=").c_str(), left);
  left = original;
  left /= right; print((std::string(dimension) + "/=").c_str(), left);
}

int main()
{
  using Vec2 = NCollection_Vec2<Tagged>;
  using Vec3 = NCollection_Vec3<Tagged>;
  using Vec4 = NCollection_Vec4<Tagged>;
  const Vec2 left2(Tagged(2), Tagged(4)), right2(Tagged(3), Tagged(2));
  const Vec3 left3(Tagged(2), Tagged(4), Tagged(1)), right3(Tagged(3), Tagged(2), Tagged(5));
  const Vec4 left4(Tagged(2), Tagged(4), Tagged(1), Tagged(6));
  const Vec4 right4(Tagged(3), Tagged(2), Tagged(5), Tagged(7));
  values("2", left2, right2); values("3", left3, right3); values("4", left4, right4);
  assignments("2", left2, right2); assignments("3", left3, right3); assignments("4", left4, right4);
}
