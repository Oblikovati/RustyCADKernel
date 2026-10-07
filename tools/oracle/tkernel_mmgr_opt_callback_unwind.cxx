// Pinned OCCT callback exception prefixes; every captured allocation is explicitly repaired.
#include <Standard_MMgrOpt.hxx>
#include <algorithm>
#include <cstring>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <tuple>
#include <vector>

enum class PanicPhase { Never, Free, Allocate };
using Event = std::tuple<bool, void*, size_t, size_t>;
PanicPhase phase = PanicPhase::Never;
std::vector<Event> events;
int replaced = 0;

void observe(bool allocate, void* storage, size_t round, size_t size)
{
  events.emplace_back(allocate, storage, round, size);
  if (phase == (allocate ? PanicPhase::Allocate : PanicPhase::Free))
    throw std::runtime_error("configured callback exception");
}

void reset(PanicPhase selected)
{
  phase = selected; events.clear(); replaced = 0;
  Standard_MMgrOpt::SetCallBackFunction(observe);
}

std::string shapes()
{
  std::ostringstream out;
  for (size_t i = 0; i < events.size(); ++i) {
    const auto& event = events[i];
    if (i != 0) out << ',';
    out << std::get<0>(event) << ':' << std::get<2>(event) << ':' << std::get<3>(event);
  }
  return out.str();
}

bool filled(void* storage, size_t length, unsigned char value)
{
  const auto* bytes = static_cast<unsigned char*>(storage);
  return std::all_of(bytes, bytes + length, [value](auto byte) { return byte == value; });
}

template<class Action> bool catches(Action action)
{
  try { action(); } catch (const std::runtime_error&) { return true; }
  return false;
}

void failed_free(size_t cell, size_t size, const char* label)
{
  Standard_MMgrOpt allocator(true, false, cell, 100, 512);
  void* block = allocator.Allocate(size);
  std::memset(block, 9, size);
  reset(PanicPhase::Free);
  bool caught = catches([&] { allocator.Free(block); });
  std::cout << "free-" << label << ' ' << caught << ' ' << filled(block, size, 9) << ' '
            << allocator.Purge(false) << ' ' << shapes() << '\n';
  phase = PanicPhase::Never;
  allocator.Free(block);
  std::cout << "retry-" << label << ' ' << shapes() << '\n';
}

void* old_block(Standard_MMgrOpt& allocator)
{
  void* block = allocator.Allocate(24);
  std::memset(block, 9, 24);
  return block;
}

void repair(Standard_MMgrOpt& allocator, void* old, void* moved)
{
  phase = PanicPhase::Never;
  allocator.Free(old); allocator.Free(moved);
}

void reallocate_free()
{
  Standard_MMgrOpt allocator(true, false, 0, 100, 512);
  void* old = old_block(allocator);
  reset(PanicPhase::Free);
  bool caught = catches([&] { allocator.Reallocate(old, 40); });
  void* moved = std::get<1>(events[0]);
  std::cout << "realloc-free " << caught << ' ' << filled(old, 24, 9) << ' '
            << filled(moved, 24, 9) << ' ' << filled(static_cast<char*>(moved) + 24, 16, 0)
            << ' ' << shapes() << '\n';
  repair(allocator, old, moved);
}

void reallocate_allocate()
{
  Standard_MMgrOpt allocator(true, false, 0, 100, 512);
  void* old = old_block(allocator);
  reset(PanicPhase::Allocate);
  bool caught = catches([&] { allocator.Reallocate(old, 40); });
  void* moved = std::get<1>(events[0]);
  std::cout << "realloc-allocate " << caught << ' ' << filled(old, 24, 9) << ' '
            << filled(moved, 40, 0) << ' ' << shapes() << '\n';
  repair(allocator, old, moved);
}

void allocate_unwind()
{
  Standard_MMgrOpt allocator(true, false, 0, 100, 512);
  reset(PanicPhase::Allocate);
  bool caught = catches([&] { allocator.Allocate(40); });
  void* block = std::get<1>(events[0]);
  std::cout << "allocate-unwind " << caught << ' ' << filled(block, 40, 0) << ' ' << shapes() << '\n';
  phase = PanicPhase::Never;
  allocator.Free(block);
}

void silent_paths()
{
  Standard_MMgrOpt allocator(true, false, 0, 100, 512);
  void* old = old_block(allocator);
  reset(PanicPhase::Allocate);
  void* same = allocator.Reallocate(old, 8);
  allocator.Free(nullptr);
  std::cout << "silent-paths " << (same == old) << ' ' << shapes().size() << '\n';
  phase = PanicPhase::Never;
  allocator.Free(old);
}

void replacement(bool allocate, void* storage, size_t round, size_t size)
{
  observe(allocate, storage, round, size);
  ++replaced;
}

void replace_callback(bool allocate, void* storage, size_t round, size_t size)
{
  observe(allocate, storage, round, size);
  Standard_MMgrOpt::SetCallBackFunction(replacement);
}

void callback_replacement()
{
  Standard_MMgrOpt allocator(true, false, 0, 100, 512);
  reset(PanicPhase::Never);
  Standard_MMgrOpt::SetCallBackFunction(replace_callback);
  void* block = allocator.Allocate(40);
  allocator.Free(block);
  std::cout << "replace " << replaced << ' ' << shapes() << '\n';
}

int main()
{
  failed_free(0, 24, "medium"); failed_free(16, 16, "small"); failed_free(0, 600, "large");
  reallocate_free(); reallocate_allocate(); allocate_unwind(); silent_paths(); callback_replacement();
  Standard_MMgrOpt::SetCallBackFunction(nullptr);
}
