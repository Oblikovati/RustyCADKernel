#include <NCollection_SparseArray.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <type_traits>

struct Traced {
  int value;
  static std::string events;
  static bool tracing;
  static int rejected_copy;
  explicit Traced(int v) : value(v) {}
  Traced(const Traced& other) : value(other.value) {
    if (tracing) events += "C" + std::to_string(value) + ",";
    if (value == rejected_copy) throw std::runtime_error("copy rejected");
  }
  Traced& operator=(const Traced& other) {
    if (tracing) events += "A" + std::to_string(value) + ">" + std::to_string(other.value) + ",";
    value = other.value; return *this;
  }
  ~Traced() { if (tracing) events += "D" + std::to_string(value) + ","; }
  static void reset() { events.clear(); tracing = true; rejected_copy = -1; }
};
std::string Traced::events;
bool Traced::tracing = false;
int Traced::rejected_copy = -1;
class LegacyCursor : public NCollection_SparseArrayBase::Iterator {
public:
  explicit LegacyCursor(const NCollection_SparseArrayBase* a) : Iterator(a) {}
  using Iterator::init;
};
void set_semantics() {
  NCollection_SparseArray<Traced> array(4);
  Traced first(7), second(9);
  Traced::reset();
  Traced* location = &array.SetValue(1, first);
  bool stable = &array.SetValue(1, second) == location;
  std::cout << "borrowed_set " << Traced::events << ' ' << stable << ' ' << array.Size() << '\n';
  Traced::reset();
  array.SetValue(1, array.Value(1));
  std::cout << "self_set " << Traced::events << ' ' << array.Value(1).value << '\n';
  Traced::reset();
  bool returned = &array.Assign(array) == &array;
  std::cout << "assign_self " << returned << ' ' << Traced::events.size() << '\n';
  Traced::tracing = false;
}
void assign_order() {
  NCollection_SparseArray<Traced> source(4), target(4);
  source.SetValue(0, Traced(10)); source.SetValue(8, Traced(80));
  target.SetValue(0, Traced(1)); target.SetValue(4, Traced(4));
  target.SetValue(8, Traced(8)); target.SetValue(12, Traced(12));
  Traced::reset();
  bool returned = &target.Assign(source) == &target;
  std::cout << "assign_order " << Traced::events << ' ' << returned << ' ' << target.Size() << '\n';
  Traced::tracing = false;
}
void throwing_copy() {
  NCollection_SparseArray<Traced> source(4), target(4);
  source.SetValue(0, Traced(10)); source.SetValue(2, Traced(20));
  target.SetValue(3, Traced(33));
  Traced::reset(); Traced::rejected_copy = 20;
  bool caught = false;
  try { target.Assign(source); } catch (const std::runtime_error&) { caught = true; }
  std::cout << "assign_throw " << Traced::events << ' ' << caught << ' ' << target.Size()
            << ' ' << target.HasValue(0) << ' ' << target.HasValue(2) << ' ' << target.HasValue(3) << '\n';
  Traced::tracing = false; Traced::rejected_copy = -1;
  NCollection_SparseArray<Traced> vacant(4);
  Traced value(44); Traced::reset(); Traced::rejected_copy = 44;
  caught = false;
  try { vacant.SetValue(9, value); } catch (const std::runtime_error&) { caught = true; }
  std::cout << "set_throw " << caught << ' ' << vacant.Size() << ' ' << vacant.Capacity()
            << ' ' << vacant.HasValue(9) << '\n';
  Traced::tracing = false; Traced::rejected_copy = -1;
}
void iterator_semantics() {
  NCollection_SparseArray<int> array(4);
  array.SetValue(5, 50);
  LegacyCursor legacy(&array);
  auto basic = array.begin();
  legacy.Next(); ++basic;
  std::cout << "legacy_end " << legacy.More() << ' ' << legacy.Index() << ' '
            << basic.More() << ' ' << basic.Index() << ' ' << array.Capacity() << '\n';
  legacy.Next(); legacy.Restart();
  std::cout << "legacy_restart " << legacy.More() << ' ' << legacy.Index() << '\n';
  NCollection_SparseArray<int>::const_iterator readonly(array.begin());
  auto mutable_cursor = array.begin();
  const bool identity = &(readonly = mutable_cursor) == &readonly;
  std::cout << "const_conversion " << readonly.Key() << ' ' << readonly.Index() << ' ' << identity << ' ' << (readonly == mutable_cursor) << '\n';
  auto previous = mutable_cursor++;
  std::cout << "mutable_postfix " << previous.Key() << ' ' << mutable_cursor.Key() << ' ' << *previous << '\n';
  array.UnsetValue(5);
  LegacyCursor retained(&array);
  std::cout << "legacy_empty_retained " << retained.More() << ' ' << retained.Index() << '\n';
}
int main() {
  NCollection_SparseArray<int> zero(0);
  std::cout << "zero_constructor " << zero.Size() << ' ' << zero.Capacity() << ' ' << zero.IsEmpty() << '\n';
  set_semantics(); assign_order(); throwing_copy(); iterator_semantics();
  std::cout << "copy_move_traits " << std::is_copy_constructible_v<NCollection_SparseArray<int>> << ' '
            << std::is_move_constructible_v<NCollection_SparseArray<int>> << '\n';
}
