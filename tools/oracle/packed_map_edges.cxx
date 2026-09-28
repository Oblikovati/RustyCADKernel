// Preserve native cardinality reporting and extremal-block selection quirks.
#include <NCollection_PackedMap.hxx>
#include <NCollection_PackedMapAlgo.hxx>
#include <iostream>
#include <limits>
int main()
{
  NCollection_PackedMap<int> left, right, lowest;
  left.Add(1); left.Add(2); right.Add(2); right.Add(3);
  std::cout << NCollection_PackedMapAlgo::Differ(left, right) << '\n';
  lowest.Add(std::numeric_limits<int>::lowest() + 1);
  std::cout << lowest.GetMinimalMapped() << ' ' << lowest.GetMaximalMapped() << '\n';
  NCollection_PackedMap<unsigned int> zeroBlock;
  zeroBlock.Add(7);
  std::cout << zeroBlock.GetMinimalMapped() << ' ' << zeroBlock.GetMaximalMapped() << '\n';
}
