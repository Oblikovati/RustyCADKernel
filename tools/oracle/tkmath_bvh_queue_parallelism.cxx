// Native concurrency contract for BVH_QueueBuilder at pinned OCCT 3d097a0.
#include <BVH_QueueBuilder.hxx>
#include <BVH_Types.hxx>
#include <atomic>
#include <barrier>
#include <chrono>
#include <iostream>
#include <thread>
#include <vector>

class ProbeSet : public BVH_Set<double, 3> {
public:
  int Size() const override { return 4; }
  BVH_Box<double, 3> Box(int index) const override {
    BVH_Vec3d low(double(index), 0.0, 0.0);
    BVH_Vec3d high(double(index + 1), 1.0, 1.0);
    return BVH_Box<double, 3>(low, high);
  }
  double Center(int index, int axis) const override { return Box(index).Center(axis); }
  void Swap(int first, int second) override { std::swap(order[first], order[second]); }
  int order[4] = {0, 1, 2, 3};
};

class ProbeBuilder : public BVH_QueueBuilder<double, 3> {
public:
  ProbeBuilder() : BVH_QueueBuilder<double, 3>(1, 8, 2), rendezvous(2) {}
  int maxActive() const { return maximum.load(); }

protected:
  BVH_ChildNodes buildNode(BVH_Set<double, 3>*, BVH_Tree<double, 3>* tree,
                           int node) const override {
    const int first = tree->BegPrimitive(node);
    const int last = tree->EndPrimitive(node);
    if (last - first + 1 == 2) rendezvous.arrive_and_wait();
    const int now = active.fetch_add(1) + 1;
    int old = maximum.load();
    while (old < now && !maximum.compare_exchange_weak(old, now)) {}
    std::this_thread::sleep_for(std::chrono::milliseconds(10));
    active.fetch_sub(1);
    if (first == last) return BVH_ChildNodes();
    const int middle = first + (last - first + 1) / 2;
    BVH_Box<double, 3> bounds(tree->MinPoint(node), tree->MaxPoint(node));
    return BVH_ChildNodes(bounds, bounds, BVH_PrimitiveRange(first, middle - 1),
                          BVH_PrimitiveRange(middle, last));
  }

private:
  mutable std::atomic<int> active{0};
  mutable std::atomic<int> maximum{0};
  mutable std::barrier<> rendezvous;
};

int main() {
  ProbeSet set;
  ProbeBuilder builder;
  BVH_Tree<double, 3> tree;
  BVH_Box<double, 3> bounds(BVH_Vec3d(0.0, 0.0, 0.0), BVH_Vec3d(4.0, 1.0, 1.0));
  builder.Build(&set, &tree, bounds);
  std::cout << builder.maxActive() << ' ' << tree.Length() << '\n';
}
