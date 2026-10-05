#include <NCollection_IndexedDataMap.hxx>
#include <iostream>

int main()
{
  NCollection_IndexedDataMap<int, int> map;
  const auto empty = map.IndexedItems();
  std::cout << "empty " << empty.size() << ' ' << empty.empty() << ' '
            << (empty.begin() == empty.end()) << '\n';
  map.Add(10, 100);
  map.Add(20, 200);
  map.Add(30, 300);
  map.Swap(1, 3);
  const auto view = map.IndexedItems();
  std::cout << "view " << view.size() << ' ' << view.empty() << '\n';
  for (auto [key, value, index] : view)
  {
    value += index;
    std::cout << key << ' ' << value << ' ' << index << '\n';
  }
  std::cout << "writes " << map.FindFromKey(10) << ' ' << map.FindFromKey(20)
            << ' ' << map.FindFromKey(30) << '\n';
}
