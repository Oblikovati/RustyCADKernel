// Both native assignment forms clear before copying and preserve the receiver allocator.
#include <NCollection_Sequence.hxx>
#include <iostream>
#include "tkernel_sequence_copy_item.hxx"
using Sequence = NCollection_Sequence<CopyItem>;
struct InspectedSequence : Sequence {
  using Sequence::Sequence;
  using Sequence::operator=;
  size_t cached_index() const { return myCurrentIndex; }
};
static std::string trace() { return events.empty() ? "-" : events; }
#include "tkernel_sequence_copy_state.hxx"
static void fill(InspectedSequence& sequence, int first, int length)
{
  for (int item = first; item < first + length; ++item) sequence.EmplaceAppend(item);
  if (length > 1) (void)sequence.Value(2);
}
static void assigned_row(const std::string& name, const InspectedSequence& target,
                         const InspectedSequence& source, CopyArena& arena,
                         CopyArena& donor_arena, bool caught)
{
  std::cout << name << ' ' << state(target) << ' ' << state(source) << ' '
            << target.Size() << ' ' << target.cached_index() << ' ' << source.cached_index()
            << ' ' << (target.Allocator().get() == &arena) << ' ' << caught
            << ' ' << arena.live.size() << ' ' << donor_arena.live.size() << ' ' << trace() << '\n';
}
static void clear_row(const std::string& name, InspectedSequence& target,
                      const InspectedSequence& source, CopyArena& arena, CopyArena& donor_arena)
{
  events.clear(); target.Clear();
  std::cout << name << "_clear " << target.Size() << ' ' << target.cached_index() << ' '
            << state(source) << ' ' << arena.live.size() << ' ' << donor_arena.live.size()
            << ' ' << trace() << '\n';
}
static void case_rows(bool assignment, bool same, int length, int failure)
{
  occ::handle<CopyArena> arena = new CopyArena;
  occ::handle<CopyArena> donor_arena = same ? arena : new CopyArena;
  InspectedSequence target(arena), source(donor_arena);
  fill(target, 9, 2); fill(source, 1, length);
  observed = arena.get(); failing = failure; events.clear();
  const std::string name = std::string(assignment ? "assign" : "operator")
    + '_' + std::to_string(same) + '_' + std::to_string(length) + '_' + std::to_string(failure);
  bool caught = false;
  try {
    Sequence& result = assignment ? target.Assign(source) : target.operator=(static_cast<const Sequence&>(source));
    if (&result != static_cast<Sequence*>(&target)) std::terminate();
  } catch (const std::runtime_error&) { caught = true; }
  assigned_row(name, target, source, *arena, *donor_arena, caught);
  clear_row(name, target, source, *arena, *donor_arena);
}
static void self_row(bool assignment)
{
  occ::handle<CopyArena> arena = new CopyArena;
  InspectedSequence target(arena); fill(target, 9, 2);
  observed = arena.get(); failing = 9; events.clear();
  Sequence& result = assignment ? target.Assign(target) : target.operator=(static_cast<const Sequence&>(target));
  std::cout << (assignment ? "assign_self" : "operator_self") << ' ' << state(target) << ' '
            << target.Size() << ' ' << target.cached_index() << ' ' << arena->live.size()
            << ' ' << (&result == static_cast<Sequence*>(&target)) << ' ' << trace() << '\n';
}
int main()
{
  for (bool assignment : {true, false}) {
    for (bool same : {false, true}) {
      case_rows(assignment, same, 0, 0);
      for (int failure = 0; failure <= 3; ++failure) case_rows(assignment, same, 3, failure);
    }
    self_row(assignment);
  }
}
