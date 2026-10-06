// OCCT 3d097a defines unsigned Size arithmetic independently of allocation validity.
#include <NCollection_SparseArrayBase.hxx>
#include <cassert>
#include <iostream>
#include <limits>
#include <type_traits>
#include <utility>

struct SparseBlockExposure : NCollection_SparseArrayBase
{
  using NCollection_SparseArrayBase::getBlock;
};
using InferredBlock = decltype(std::declval<SparseBlockExposure>().getBlock(nullptr));
constexpr size_t maximum = std::numeric_limits<size_t>::max();
static_assert(std::is_same_v<InferredBlock::Cell, unsigned char>);
static_assert(InferredBlock::Size(maximum, 0) == sizeof(size_t));
static_assert(InferredBlock::Size(maximum, 1) == sizeof(size_t) - 1);

static void sizeArithmetic()
{
  const std::pair<size_t, size_t> cases[] = {
    {0, 0}, {0, maximum}, {1, 0}, {1, maximum}, {7, maximum}, {8, maximum},
    {9, 12}, {maximum, 0}, {maximum, 1}, {maximum, maximum},
    {maximum - 7, 0}, {maximum - 6, 3}, {maximum / 2 + 1, 2}
  };
  for (const auto& [items, stride] : cases)
    std::cout << "size " << items << ' ' << stride << ' ' << InferredBlock::Size(items, stride) << '\n';
}

static void ignoredArrayArguments()
{
  alignas(size_t) unsigned char storage[sizeof(size_t) + 16] = {};
  auto* expected = reinterpret_cast<char*>(storage) + sizeof(size_t);
  auto* ordinary = InferredBlock::ToArray(storage, 4, 4);
  auto* zero = InferredBlock::ToArray(storage, 0, 0);
  auto* extreme = InferredBlock::ToArray(storage, maximum, maximum);
  assert(ordinary == expected && zero == expected && extreme == expected);
  *ordinary = 42;
  std::cout << "array " << (ordinary - reinterpret_cast<char*>(storage)) << ' '
            << int(*zero) << ' ' << int(*extreme) << '\n';
  std::cout << "cell " << sizeof(InferredBlock::Cell) << ' ' << InferredBlock::bitsPerCell() << '\n';
}

int main()
{
  sizeArithmetic();
  ignoredArrayArguments();
}
