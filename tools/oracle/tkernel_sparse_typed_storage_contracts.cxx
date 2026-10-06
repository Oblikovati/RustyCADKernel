#include <NCollection_SparseArray.hxx>
#include <iostream>
#include <stdexcept>
#include <string>

struct TypedTrace {
  int value;
  static int reject_create, reject_assign;
  static bool logging;
  static std::string events;
  explicit TypedTrace(int v) : value(v) {}
  TypedTrace(const TypedTrace& v) : value(v.value) {
    if (logging) events += "C" + std::to_string(value) + ",";
    if (value == reject_create) throw std::runtime_error("create");
  }
  TypedTrace& operator=(const TypedTrace& v) {
    if (logging) events += "A" + std::to_string(value) + ">" + std::to_string(v.value) + ",";
    value = v.value;
    if (value == reject_assign) throw std::runtime_error("assign");
    return *this;
  }
  ~TypedTrace() { if (logging) events += "D" + std::to_string(value) + ","; }
  static void reset() { events.clear(); reject_create = reject_assign = -1; logging = true; }
};
int TypedTrace::reject_create = -1, TypedTrace::reject_assign = -1;
bool TypedTrace::logging = false;
std::string TypedTrace::events;
using Array = NCollection_SparseArray<TypedTrace>;

void growth_addresses() {
  Array values(4);
  TypedTrace one(1), far(4096), replacement(2);
  TypedTrace* first = &values.SetValue(1, one);
  values.SetValue(4096, far);
  TypedTrace* replaced = &values.SetValue(1, replacement);
  std::cout << "stable_growth " << (first == &values.ChangeValue(1)) << ' '
            << (replaced == first) << ' ' << values.Size() << ' ' << values.Capacity() << '\n';
}
void assignment_geometry() {
  Array source(4), target(4);
  source.SetValue(1, TypedTrace(11)); target.SetValue(1, TypedTrace(1));
  target.SetValue(64, TypedTrace(64));
  const TypedTrace* retained = &target.Value(1);
  TypedTrace::reset(); target.Assign(source);
  std::cout << "assign_retained " << TypedTrace::events << ' ' << (retained == &target.Value(1))
            << ' ' << target.Size() << ' ' << target.Capacity() << '\n';
  TypedTrace::logging = false;
}
void changed_geometry() {
  Array source(3), target(4);
  source.SetValue(2, TypedTrace(22)); target.SetValue(1, TypedTrace(1));
  TypedTrace::reset(); target.Assign(source);
  std::cout << "assign_geometry " << TypedTrace::events << ' ' << target.Size()
            << ' ' << target.Capacity() << ' ' << target.Value(2).value << '\n';
  TypedTrace::logging = false;
}
void copy_failure() {
  Array source(4), target(4);
  source.SetValue(0, TypedTrace(10)); source.SetValue(2, TypedTrace(20));
  target.SetValue(0, TypedTrace(1)); target.SetValue(3, TypedTrace(3));
  TypedTrace::reset(); TypedTrace::reject_create = 20;
  bool caught = false;
  try { target.Assign(source); } catch (const std::runtime_error&) { caught = true; }
  std::cout << "create_failure " << TypedTrace::events << ' ' << caught << ' ' << target.Size()
            << ' ' << target.Value(0).value << ' ' << target.HasValue(2) << ' ' << target.HasValue(3) << '\n';
  TypedTrace::logging = false; TypedTrace::reject_create = -1;
}
void assignment_failure() {
  Array source(4), target(4);
  source.SetValue(0, TypedTrace(10)); source.SetValue(2, TypedTrace(20));
  target.SetValue(0, TypedTrace(1)); target.SetValue(2, TypedTrace(2));
  TypedTrace::reset(); TypedTrace::reject_assign = 20;
  bool caught = false;
  try { target.Assign(source); } catch (const std::runtime_error&) { caught = true; }
  std::cout << "assign_failure " << TypedTrace::events << ' ' << caught << ' ' << target.Size()
            << ' ' << target.Value(0).value << ' ' << target.Value(2).value << '\n';
  TypedTrace::logging = false; TypedTrace::reject_assign = -1;
}
void failed_block_then_clear() {
  Array values(4);
  TypedTrace item(9); TypedTrace::reset(); TypedTrace::reject_create = 9;
  try { values.SetValue(33, item); } catch (const std::runtime_error&) {}
  auto cursor = values.cbegin();
  std::cout << "empty_allocated " << values.Size() << ' ' << values.Capacity() << ' '
            << cursor.More() << ' ' << cursor.Index() << '\n';
  TypedTrace::reset(); values.Clear();
  std::cout << "clear_failed_block " << TypedTrace::events.size() << ' ' << values.Size()
            << ' ' << values.Capacity() << '\n';
  TypedTrace::logging = false;
}
void exchange_addresses() {
  Array first(3), second(4);
  first.SetValue(2, TypedTrace(2)); second.SetValue(17, TypedTrace(17));
  const TypedTrace* original_first = &first.Value(2);
  const TypedTrace* original_second = &second.Value(17);
  TypedTrace::reset(); first.Exchange(second);
  std::cout << "exchange_addresses " << TypedTrace::events.size() << ' '
            << (original_first == &second.Value(2)) << ' ' << (original_second == &first.Value(17))
            << ' ' << first.Capacity() << ' ' << second.Capacity() << '\n';
  TypedTrace::logging = false;
}
void destruction_order() {
  Array values(4);
  values.SetValue(5, TypedTrace(5)); values.SetValue(1, TypedTrace(1)); values.SetValue(0, TypedTrace(0));
  TypedTrace::reset(); values.Clear();
  std::cout << "typed_clear " << TypedTrace::events << ' ' << values.Size() << ' ' << values.Capacity() << '\n';
  TypedTrace::logging = false;
}
int main() {
  growth_addresses(); assignment_geometry(); changed_geometry(); copy_failure();
  assignment_failure(); failed_block_then_clear(); exchange_addresses(); destruction_order();
}
