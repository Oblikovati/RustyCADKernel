#include <NCollection_DoubleMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <vector>
using Map = NCollection_DoubleMap<int, int>;
struct Observed {
  static std::vector<int> copies;
  int value;
  explicit Observed(int v) : value(v) {}
  Observed(const Observed& other) : value(other.value) { copies.push_back(value); }
};
std::vector<int> Observed::copies;
struct Policy {
  size_t operator()(const Observed& key) const { return key.value; }
  bool operator()(const Observed& left, const Observed& right) const { return left.value == right.value; }
};
static void signed_inputs() {
  int failures = 0;
  try { Map invalid(-1); } catch (const Standard_OutOfRange&) { ++failures; }
  Map map(0);
  try { map.ReSize(-1); } catch (const Standard_OutOfRange&) { ++failures; }
  auto before = map.NbBuckets(); map.ReSize(0);
  std::cout << "signed " << failures << ' ' << before << ' ' << map.NbBuckets() << '\n';
}
static void move_constructor() {
  Map source(500, new NCollection_IncAllocator);
  source.Bind(1, 10); auto pointer = source.Seek1(1); auto allocator = source.Allocator();
  Map moved(std::move(source));
  std::cout << "move_constructor " << (pointer == moved.Seek1(1)) << ' ' << source.Size() << ' ' << source.NbBuckets() << ' ' << (source.Allocator() == allocator) << '\n';
}
static void move_assignment() {
  Map destination(700, new NCollection_IncAllocator), source(500, new NCollection_IncAllocator);
  destination.Bind(9, 90); source.Bind(1, 10);
  auto old_allocator = destination.Allocator(); auto pointer = source.Seek1(1);
  auto* returned = &(destination = std::move(source));
  std::cout << "move_assignment " << (returned == &destination) << ' ' << (pointer == destination.Seek1(1)) << ' ' << source.Size() << ' ' << source.NbBuckets() << ' ' << (source.Allocator() == old_allocator) << '\n';
}
static void copy_order() {
  NCollection_DoubleMap<Observed, Observed, Policy, Policy> source, destination;
  source.Bind(Observed(1), Observed(2)); Observed::copies.clear();
  destination.Assign(source);
  std::cout << "copy_order"; for (int value : Observed::copies) std::cout << ' ' << value;
  std::cout << '\n';
}
static void borrowed_bind() {
  NCollection_DoubleMap<Observed, Observed, Policy, Policy> map;
  Observed first(1), second(2); Observed::copies.clear();
  map.Bind(first, second);
  std::cout << "borrowed_bind"; for (int value : Observed::copies) std::cout << ' ' << value;
  Observed::copies.clear(); bool duplicate = map.TryBind(first, second);
  std::cout << ' ' << duplicate << ' ' << Observed::copies.size() << '\n';
}
static void assignment() {
  Map source, destination(700); source.Bind(1, 10);
  bool returned = &destination.Assign(source) == &destination;
  auto pointer = destination.Seek1(1); destination.Assign(destination);
  bool self = pointer == destination.Seek1(1); source.Clear(); destination.Assign(source);
  std::cout << "assignment " << returned << ' ' << self << ' ' << destination.Size() << ' ' << destination.NbBuckets() << '\n';
}
static void cursor() {
  Map map; map.Bind(1, 10); map.Bind(2, 20);
  Map::Iterator it(map); auto previous = it++;
  std::cout << "cursor " << *previous << ' ' << it.Value() << ' ' << (it.operator->() == map.Seek1(2));
  ++it; ++it; bool ended = !it.More(); it.Reset();
  std::cout << ' ' << ended << ' ' << it.Value() << '\n';
}
int main() { signed_inputs(); move_constructor(); move_assignment(); copy_order(); borrowed_bind(); assignment(); cursor(); }
