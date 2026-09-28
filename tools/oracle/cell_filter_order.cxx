// Reproduce coordinate truncation, wrapping and traversal using upstream inspectors.
#include "NCollection_CellFilter_Test.cxx"
#include <iostream>
#include <limits>
int main()
{
  NCollection_CellFilter<TestInspector2D> filter(1.0);
  filter.Add(10, TestPoint2D(-0.9, -0.9));
  filter.Add(20, TestPoint2D(0.9, 0.9));
  filter.Add(30, TestPoint2D(1.1, 0.1));
  filter.Add(40, TestPoint2D(0.1, 1.1));
  filter.Add(50, TestPoint2D(1.1, 1.1));
  TestInspector2D inspector;
  filter.Inspect(TestPoint2D(0, 0), TestPoint2D(1.9, 1.9), inspector);
  for (int value : inspector.Found()) std::cout << value << ' ';
  std::cout << '\n';
  filter.Add(60, TestPoint2D(double(std::numeric_limits<int>::max()) + 1, 0));
  filter.Add(70, TestPoint2D(double(std::numeric_limits<int>::min()) - 1, 0));
  inspector.Reset();
  filter.Inspect(TestPoint2D(1, 0), inspector);
  filter.Inspect(TestPoint2D(-1, 0), inspector);
  for (int value : inspector.Found()) std::cout << value << ' ';
  std::cout << '\n';
}
