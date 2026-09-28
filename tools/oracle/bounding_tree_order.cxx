// Reproduce supplemental traversal order using the pinned upstream test helpers.
#include "NCollection_UBTree_Test.cxx"
#include <iostream>
int main()
{
  NCollection_EBTree<int, TestBnd1D> tree;
  const double bounds[][2] = {{0, 2}, {3, 5}, {4, 6}, {8, 10}, {-2, 9}, {1, 1}};
  for (int index = 0; index < 6; ++index)
    tree.Add(index + 1, TestBnd1D(bounds[index][0], bounds[index][1]));
  TestSelector1D before(TestBnd1D(-10, 20));
  tree.Select(before);
  for (int value : before.Results()) std::cout << value << ' ';
  std::cout << '\n';
  tree.Remove(2);
  tree.Remove(5);
  TestSelector1D after(TestBnd1D(-10, 20));
  tree.Select(after);
  for (int value : after.Results()) std::cout << value << ' ';
  std::cout << '\n';
}
