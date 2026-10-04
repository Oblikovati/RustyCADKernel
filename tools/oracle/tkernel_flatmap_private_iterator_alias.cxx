#include <NCollection_FlatDataMap.hxx>
#include <NCollection_FlatMap.hxx>

#include <iostream>
#include <type_traits>

int main()
{
  NCollection_FlatMap<int> aKeys;
  aKeys.Add(7);
  auto aKeyIterator = aKeys.begin();
  static_assert(std::is_same_v<decltype(aKeyIterator), NCollection_FlatMap<int>::iterator>);
  std::cout << aKeyIterator.More() << '/' << aKeyIterator.Key() << '/' << *aKeyIterator << '/';
  ++aKeyIterator;
  std::cout << aKeyIterator.More() << ';';

  NCollection_FlatDataMap<int, int> aMap;
  aMap.Bind(3, 4);
  auto aMapIterator = aMap.begin();
  static_assert(
    std::is_same_v<decltype(aMapIterator), NCollection_FlatDataMap<int, int>::iterator>);
  std::cout << aMapIterator.More() << '/' << aMapIterator.Key() << '/' << *aMapIterator << '/';
  *aMapIterator = 5;
  std::cout << *aMapIterator << '/';
  ++aMapIterator;
  std::cout << aMapIterator.More() << ';';

  const auto& aConstMap = aMap;
  auto aConstIterator = aConstMap.begin();
  static_assert(
    std::is_same_v<decltype(aConstIterator), NCollection_FlatDataMap<int, int>::const_iterator>);
  std::cout << aConstIterator.Key() << '/' << *aConstIterator << '\n';
  return 0;
}
