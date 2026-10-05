#include <NCollection_Vec2.hxx>
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>

#include <iostream>
#include <string>
#include <vector>

struct Probe
{
  int value;
  std::vector<std::string>* events;
};

bool operator==(const Probe& left, const Probe& right)
{
  left.events->push_back("eq(" + std::to_string(left.value) + "," + std::to_string(right.value) + ")");
  return left.value == right.value;
}

template <class Vector>
void emit(const char* name, bool result, const Vector& events)
{
  std::cout << name << '=' << result << " trace=";
  for (std::size_t index = 0; index < events.size(); ++index)
  {
    std::cout << (index == 0 ? "" : ",") << events[index];
  }
  std::cout << '\n';
}

int main()
{
  std::vector<std::string> twoEvents;
  const auto twoLeft = NCollection_Vec2<Probe>({1, &twoEvents}, {2, &twoEvents});
  const auto twoRight = NCollection_Vec2<Probe>({1, &twoEvents}, {9, &twoEvents});
  emit("vec2_eq", twoLeft == twoRight, twoEvents);

  std::vector<std::string> threeEvents;
  const auto threeLeft = NCollection_Vec3<Probe>({1, &threeEvents}, {2, &threeEvents}, {3, &threeEvents});
  const auto threeRight = NCollection_Vec3<Probe>({9, &threeEvents}, {2, &threeEvents}, {3, &threeEvents});
  emit("vec3_ne", threeLeft != threeRight, threeEvents);

  std::vector<std::string> fourEvents;
  const auto fourLeft = NCollection_Vec4<Probe>({1, &fourEvents}, {2, &fourEvents}, {3, &fourEvents}, {4, &fourEvents});
  const auto fourRight = NCollection_Vec4<Probe>({1, &fourEvents}, {2, &fourEvents}, {3, &fourEvents}, {9, &fourEvents});
  emit("vec4_is_equal", fourLeft.IsEqual(fourRight), fourEvents);
}
