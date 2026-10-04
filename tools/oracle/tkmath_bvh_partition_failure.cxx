// Native failure-state probe for the pinned BVH_BinnedBuilder implementation.
#include <BVH_BinnedBuilder.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <utility>

class FailingSet : public BVH_Set<double, 3> {
public:
  int Size() const override { return 4; }
  BVH_Box<double, 3> Box(int index) const override {
    const double x = double(3 - index);
    return BVH_Box<double, 3>(BVH_Vec3d(x, 0.0, 0.0), BVH_Vec3d(x + 1.0, 1.0, 1.0));
  }
  double Center(int index, int axis) const override { return Box(index).Center(axis); }
  void Swap(int, int) override { ++swapCount; throw Standard_Failure("partition swap"); }
  int swapCount = 0;
};

int main() {
  FailingSet set;
  BVH_BinnedBuilder<double, 3, 4> builder(1, 8, false, 1);
  BVH_Tree<double, 3> tree;
  BVH_Box<double, 3> bounds(BVH_Vec3d(0.0, 0.0, 0.0), BVH_Vec3d(4.0, 1.0, 1.0));
  try { builder.Build(&set, &tree, bounds); }
  catch (const Standard_Failure&) { std::cout << set.swapCount << ' ' << tree.IsOuter(0) << '\n'; }
}
