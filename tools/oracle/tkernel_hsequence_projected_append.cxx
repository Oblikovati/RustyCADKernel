// Derived ChangeSequence lookup is performed before the base Append self/empty checks.
#include <NCollection_HSequence.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
static std::string calls;
static int failed_copy = 0;
struct AppendItem {
  int value;
  explicit AppendItem(int number) : value(number) {}
  AppendItem(const AppendItem& source) : value(source.value) {
    calls += "c" + std::to_string(value) + ";";
    if (value == failed_copy) throw std::runtime_error("copy failure");
  }
  ~AppendItem() { calls += "d" + std::to_string(value) + ";"; }
};
using Sequence = NCollection_Sequence<AppendItem>;
using SharedSequence = NCollection_HSequence<AppendItem>;
struct DerivedSequence : SharedSequence {
  ::Sequence alternate;
  bool redirect = false, fail_getter = false;
  explicit DerivedSequence(const occ::handle<NCollection_BaseAllocator>& arena)
      : SharedSequence(::Sequence(arena)), alternate(arena) {}
  ::Sequence& ChangeSequence() {
    calls += "g;";
    if (fail_getter) throw std::runtime_error("getter failure");
    return redirect ? alternate : SharedSequence::ChangeSequence();
  }
};
static std::string values(const Sequence& source)
{
  std::string result;
  for (Sequence::Iterator cursor(source); cursor.More(); cursor.Next()) {
    if (!result.empty()) result += ',';
    result += std::to_string(cursor.Value().value);
  }
  return result.empty() ? "-" : result;
}
static void fill(Sequence& sequence, int first, int last)
{
  for (int value = first; value <= last; ++value) sequence.EmplaceAppend(value);
}
static void transfer_case(const char* name, bool same, int fail)
{
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator(4096);
  occ::handle<NCollection_BaseAllocator> targetArena = same ? arena : new NCollection_IncAllocator(4096);
  occ::handle<DerivedSequence> source = new DerivedSequence(arena);
  SharedSequence target{Sequence(targetArena)};
  fill(source->SharedSequence::ChangeSequence(), 1, 2); fill(target.ChangeSequence(), 0, 0);
  const AppendItem* before = &source->First();
  calls.clear(); failed_copy = fail; bool caught = false;
  try { target.Append(source); } catch (const std::runtime_error&) { caught = true; }
  std::cout << name << ' ' << values(target) << ' ' << values(*source) << ' '
            << caught << ' ' << (target.Size() > 1 && &target.Value(2) == before) << ' ' << source->GetRefCount()
            << ' ' << calls << '\n';
  failed_copy = 0;
}
static void simple_case(const char* name, bool self, bool empty, bool redirect, bool fail)
{
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator(4096);
  occ::handle<DerivedSequence> source = new DerivedSequence(arena);
  SharedSequence target{Sequence(arena)};
  if (!empty) fill(source->SharedSequence::ChangeSequence(), 1, 2);
  fill(target.ChangeSequence(), 0, 0);
  if (redirect) fill(source->alternate, 7, 8);
  source->redirect = redirect; source->fail_getter = fail; calls.clear(); bool caught = false;
  SharedSequence& receiver = self ? static_cast<SharedSequence&>(*source) : target;
  try { receiver.Append(source); } catch (const std::runtime_error&) { caught = true; }
  std::cout << name << ' ' << values(receiver) << ' ' << values(*source) << ' '
            << values(source->alternate) << ' ' << caught << ' ' << source->GetRefCount()
            << ' ' << calls << '\n';
}
static void base_handle_case()
{
  occ::handle<SharedSequence> source = new SharedSequence;
  SharedSequence target;
  fill(source->ChangeSequence(), 1, 2); calls.clear();
  target.Append(source);
  std::cout << "base " << values(target) << ' ' << values(*source) << ' '
            << source->GetRefCount() << ' ' << (calls.empty() ? "-" : calls) << '\n';
}
int main()
{
  transfer_case("same", true, 0); transfer_case("different", false, 0);
  transfer_case("copy_fail1", false, 1); transfer_case("copy_fail2", false, 2);
  simple_case("empty", false, true, false, false);
  simple_case("self", true, false, false, false);
  simple_case("redirect", false, false, true, false);
  simple_case("getter_fail", false, false, false, true);
  base_handle_case();
}
