// Pinned OCCT 3d097a03: callable protected helpers with caller-visible recursive cursors.
#include <NCollection_CellFilter.hxx>
#include <array>
#include <iostream>
#include <vector>
struct Inspector {
  using Target = int;
  using Point = std::array<double, 3>;
  static constexpr int Dimension = 3;
  std::vector<int> seen;
  int fail = 0;
  bool purge = false;
  static double Coord(int axis, const Point& point) { return point[axis]; }
  static bool IsEqual(int first, int second) { return first == second; }
  NCollection_CellFilter_Action Inspect(int target) {
    seen.push_back(target);
    if (fail && int(seen.size()) == fail) throw 17;
    return purge ? CellFilter_Purge : CellFilter_Keep;
  }
};
struct Access : NCollection_CellFilter<Inspector> {
  using NCollection_CellFilter::NCollection_CellFilter;
  using NCollection_CellFilter::Cell;
  using NCollection_CellFilter::CellIndex;
  using NCollection_CellFilter::add;
  using NCollection_CellFilter::remove;
  using NCollection_CellFilter::inspect;
  using NCollection_CellFilter::iterateAdd;
  using NCollection_CellFilter::iterateRemove;
  using NCollection_CellFilter::iterateInspect;
  using NCollection_CellFilter::resetAllocator;
};
Access::CellIndex index_of(std::initializer_list<int> values) {
  Access::CellIndex index(values.size());
  std::copy(values.begin(), values.end(), static_cast<int*>(index));
  return index;
}
template<class Probe>
void print_row(const char* label, const Access::CellIndex& cursor, const Probe& inspector) {
  std::cout << label << " cursor";
  for (size_t i = 0; i < cursor.Size(); ++i) std::cout << ' ' << cursor[i];
  std::cout << " seen";
  for (int value : inspector.seen) std::cout << ' ' << value;
  std::cout << '\n';
}
void direct_rows() {
  Access filter(1.0); auto index = index_of({4,-2,9}); Access::Cell cell(index);
  for (int target : {7,8,7}) filter.add(index, target);
  Inspector inspector; filter.inspect(cell, inspector); print_row("direct",index,inspector);
  filter.remove(cell,7); inspector.seen.clear(); filter.inspect(cell,inspector);
  print_row("removed",index,inspector);
  filter.resetAllocator(nullptr); inspector.seen.clear(); filter.inspect(cell,inspector);
  print_row("reset",index,inspector);
  filter.Add(11, Inspector::Point{4,-2,9}); filter.inspect(cell,inspector);
  print_row("size-kept",index,inspector);
}
void range_rows(const char* name, int axis, Access::CellIndex cursor,
                const Access::CellIndex& min, const Access::CellIndex& max) {
  Access filter(1.0); Access::Cell lower(min), upper(max);
  filter.iterateAdd(axis,cursor,lower,upper,5);
  Access::Cell cell(cursor); Inspector inspector;
  filter.iterateInspect(axis,cell,lower,upper,inspector);
  print_row(name,cell.index,inspector);
  filter.iterateRemove(axis,cell,lower,upper,5); inspector.seen.clear();
  filter.iterateInspect(axis,cell,lower,upper,inspector);
  print_row("range-empty",cell.index,inspector);
}
void panic_rows() {
  Access filter(1.0); auto index = index_of({90,91,92});
  Access::Cell lower(index_of({1,2,3})), upper(index_of({2,3,4}));
  filter.iterateAdd(1,index,lower,upper,5);
  Access::Cell cursor(index_of({90,91,92})); Inspector inspector; inspector.fail=3; inspector.purge=true;
  try { filter.iterateInspect(1,cursor,lower,upper,inspector); } catch (int) {}
  print_row("panic",cursor.index,inspector);
  inspector.seen.clear(); inspector.fail=0; inspector.purge=false;
  filter.iterateInspect(1,cursor,lower,upper,inspector);
  print_row("panic-remains",cursor.index,inspector);
}
struct FailureState { int copies=0, comparisons=0, failCopy=0, failCompare=0; };
struct FailureTarget {
  FailureState* state;
  explicit FailureTarget(FailureState& value) : state(&value) {}
  FailureTarget(const FailureTarget& other) : state(other.state) {
    if (++state->copies == state->failCopy) throw 18;
  }
};
struct FailureInspector {
  using Target = FailureTarget;
  using Point = Inspector::Point;
  static constexpr int Dimension=3;
  std::vector<int> seen;
  static double Coord(int axis,const Point& point) { return point[axis]; }
  static bool IsEqual(const Target& first,const Target&) {
    if (++first.state->comparisons == first.state->failCompare) throw 19;
    return true;
  }
  NCollection_CellFilter_Action Inspect(const Target&) { seen.push_back(5); return CellFilter_Keep; }
};
struct FailureAccess : NCollection_CellFilter<FailureInspector> {
  using NCollection_CellFilter::NCollection_CellFilter;
  using NCollection_CellFilter::Cell;
  using NCollection_CellFilter::iterateAdd;
  using NCollection_CellFilter::iterateRemove;
  using NCollection_CellFilter::iterateInspect;
};
void callback_failure(bool removal) {
  FailureState state; FailureTarget target(state); FailureAccess filter(1.0);
  auto cursor = index_of({90,91,92});
  FailureAccess::Cell lower(index_of({1,2,3})), upper(index_of({2,3,4}));
  if (removal) filter.iterateAdd(1,cursor,lower,upper,target);
  state.failCopy = removal ? 0 : 3; state.failCompare = removal ? 3 : 0;
  FailureAccess::Cell cell(index_of({90,91,92}));
  try {
    if (removal) filter.iterateRemove(1,cell,lower,upper,target);
    else filter.iterateAdd(1,cell.index,lower,upper,target);
  } catch (int) {}
  FailureInspector probe;
  print_row(removal ? "equal-failure" : "copy-failure",cell.index,probe);
  filter.iterateInspect(1,cell,lower,upper,probe);
  print_row(removal ? "equal-remains" : "copy-remains",cell.index,probe);
}
int main() {
  direct_rows();
  range_rows("axis0",0,index_of({90,91,92}),index_of({1,2,3}),index_of({2,3,4}));
  range_rows("axis1",1,index_of({90,91,92}),index_of({1,2}),index_of({2,3}));
  range_rows("reverse-inner",1,index_of({90,91,92}),index_of({2,2}),index_of({1,3}));
  range_rows("reverse-outer",1,index_of({90,91,92}),index_of({1,3}),index_of({2,2}));
  range_rows("max",0,index_of({90}),index_of({INT_MAX}),index_of({INT_MAX}));
  range_rows("min",0,index_of({90}),index_of({INT_MIN}),index_of({INT_MIN}));
  panic_rows();
  callback_failure(false); callback_failure(true);
  range_rows("reverse-empty",0,index_of({}),index_of({2}),index_of({1}));
}
