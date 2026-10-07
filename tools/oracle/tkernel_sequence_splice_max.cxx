// OCCT 3d097a03: distinct-allocator size_t MAX wraps to prependSeq(...,0).
#include <NCollection_Sequence.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <limits>
#include "tkernel_sequence_copy_item.hxx"
using Sequence = NCollection_Sequence<CopyItem>;
#include "tkernel_sequence_copy_state.hxx"
static void fill(Sequence& sequence, int length)
{
  for (int number = 1; number <= length; ++number) sequence.EmplaceAppend(number);
}
static std::string snapshot(const Sequence& target, const Sequence& source,
                            const CopyArena& arena, const CopyArena& donor)
{
  return state(target) + ' ' + state(source) + ' ' + std::to_string(arena.live.size())
    + ' ' + std::to_string(donor.live.size()) + ' ' + (events.empty() ? "-" : events);
}
static int insert(size_t index, Sequence& target, Sequence& source)
{
  try { target.InsertAfter(index, source); }
  catch (const Standard_OutOfRange&) { return 1; }
  catch (const std::runtime_error&) { return 2; }
  return 0;
}
static void case_rows(size_t index, bool same, int length, int failure)
{
  occ::handle<CopyArena> arena = new CopyArena;
  occ::handle<CopyArena> donor = same ? arena : new CopyArena;
  Sequence target(arena), source(donor); fill(target, 2); fill(source, length);
  observed = arena.get(); failing = failure; events.clear();
  int caught = insert(index, target, source);
  const std::string label = std::to_string(same) + '_' + std::to_string(length)
    + '_' + std::to_string(failure) + '_'
    + (index == std::numeric_limits<size_t>::max() ? "max" : std::to_string(index));
  std::cout << label << ' ' << caught << ' ' << snapshot(target, source, *arena, *donor) << '\n';
  events.clear(); target.Clear();
  std::cout << label << "_clear " << snapshot(target, source, *arena, *donor) << '\n';
}
static void self_and_empty_rows()
{
  NCollection_Sequence<int> target, empty; target.Append(7); target.Append(8);
  target.InsertAfter(std::numeric_limits<size_t>::max(), target);
  bool self_ok = target.Size() == 2 && target.First() == 7 && target.Last() == 8;
  bool negative = false, zero = false;
  try { target.InsertAfter(-1, empty); } catch (const Standard_OutOfRange&) { negative = true; }
  try { target.InsertBefore(size_t(0), empty); } catch (const Standard_OutOfRange&) { zero = true; }
  std::cout << "guards " << self_ok << ' ' << negative << ' ' << zero
            << ' ' << target.Size() << ' ' << empty.Size() << '\n';
}
int main()
{
  for (bool same : {false, true}) {
    case_rows(std::numeric_limits<size_t>::max(), same, 0, 0);
    for (int failure = 0; failure <= 3; ++failure)
      case_rows(std::numeric_limits<size_t>::max(), same, 3, failure);
    for (size_t index = 0; index <= 2; ++index) case_rows(index, same, 3, 0);
  }
  self_and_empty_rows();
}
