// Failed ordinary copy construction frees only the failing node, leaving completed nodes alive.
#include <NCollection_HSequence.hxx>
#include <algorithm>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
#include "tkernel_sequence_copy_item.hxx"
using Sequence = NCollection_Sequence<CopyItem>;
using SharedSequence = NCollection_HSequence<CopyItem>;
static std::string trace() { return events.empty() ? "-" : events; }
static std::string source_state(const Sequence& source)
{
  std::string values, reads;
  for (Sequence::Iterator cursor(source); cursor.More(); cursor.Next()) {
    if (!values.empty()) { values += ','; reads += ','; }
    values += std::to_string(cursor.Value().number);
    reads += std::to_string(cursor.Value().reads);
  }
  return source.IsEmpty() ? "- -" : values + " " + reads;
}
static std::string successful_row(const std::string& name, const Sequence& source, const Sequence& copy)
{
  if (source.Allocator() != copy.Allocator()) throw std::runtime_error("allocator identity mismatch");
  return name + " " + std::to_string(copy.Size()) + " " + source_state(source) + " "
       + std::to_string(observed->live.size()) + " " + trace() + "\n";
}
static std::string construct_observed(const std::string& name, bool wrapped, const Sequence& source)
{
  if (wrapped) {
    SharedSequence copy(source);
    const std::string row = successful_row(name, source, copy);
    events.clear(); return row;
  }
  Sequence copy(source);
  const std::string row = successful_row(name, source, copy);
  events.clear(); return row;
}
static void recover_raw_prefix(CopyArena& arena, size_t source_length)
{
  // No failed container is inspected: the allocator retained original allocation provenance.
  std::vector<void*> remaining(arena.live.begin() + source_length, arena.live.end());
  for (void* pointer : remaining) {
    auto node = static_cast<Sequence::Node*>(pointer);
    node->~Node(); arena.Free(node);
  }
}
static void case_rows(bool wrapped, int length, int failure)
{
  occ::handle<CopyArena> arena = new CopyArena;
  Sequence source(arena);
  for (int item = 1; item <= length; ++item) source.EmplaceAppend(item);
  observed = arena.get(); failing = failure; events.clear();
  const int baseline = arena->GetRefCount();
  const std::string name = std::string(wrapped ? "wrapper_" : "sequence_")
                         + std::to_string(length) + "_" + std::to_string(failure);
  try { std::cout << construct_observed(name, wrapped, source); }
  catch (const std::runtime_error&) {
    std::cout << name << " - " << source_state(source) << ' ' << arena->live.size() << ' ' << trace() << '\n';
    events.clear();
  }
  std::cout << name << "_end " << arena->live.size() << ' ' << trace() << ' '
            << (arena->GetRefCount() == baseline) << '\n';
  recover_raw_prefix(*arena, static_cast<size_t>(length));
}
int main()
{
  for (bool wrapped : {false, true}) {
    case_rows(wrapped, 0, 0);
    for (int failure = 0; failure <= 3; ++failure) case_rows(wrapped, 3, failure);
  }
}
