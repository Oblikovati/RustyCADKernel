#include <MathPoly_Types.hxx>
#include <MathUtils_Types.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>

constexpr uint64_t rootBits[] = {0, 0x8000000000000000ULL, 0x7ff0000000000000ULL, 0x7ff8000000001234ULL};

template<class Result, size_t Capacity>
void probe_index()
{
  Result result;
  for (size_t index = 0; index < Capacity; ++index)
    std::memcpy(&result.Roots[index], &rootBits[index % 4], sizeof(double));
  for (size_t index = 0; index < Capacity; ++index)
  {
    const double value = result[index];
    uint64_t bits;
    std::memcpy(&bits, &value, sizeof(double));
    std::cout << (index == 0 ? "" : " ") << std::hex << bits;
  }
  std::cout << '\n';
}

int main()
{
  probe_index<MathPoly::PolyResult, 20>();
  probe_index<MathUtils::PolyResult, 4>();
}
