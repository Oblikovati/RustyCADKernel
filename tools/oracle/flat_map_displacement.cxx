// Reproduces the pinned OCCT Emplaced return value after Robin Hood displacement.
#include <NCollection_FlatMap.hxx>
#include <iostream>
int main() {
  NCollection_FlatMap<int> aMap;
  aMap.Add(0);
  aMap.Add(1);
  std::cout << "Emplaced " << aMap.Emplaced(8) << '\n';
  NCollection_FlatMap<int> aSecond;
  aSecond.Add(0);
  aSecond.Add(1);
  std::cout << "Added " << aSecond.Added(8) << '\n';
}
