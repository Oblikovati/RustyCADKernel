// Kernel fill construction and the explicit Rust-style owned-donor wrapper are separate rows.
#include <NCollection_LinearVector.hxx>
#include <Standard.hxx>
#include <iostream>
#include <optional>
#include "tkernel_linear_constructor_item.hxx"
using Vector = NCollection_LinearVector<CopyItem>;
struct FillSpec { const char* name; size_t count, fail; bool owned; };

Vector owned_fill(size_t count, CopyItem donor) { return Vector(count, donor); }

void emit(const char* phase, const FillSpec& spec, const Vector* result, bool success)
{
  std::cout << phase << ':' << spec.name << ':' << (success ? "ok" : "throw") << ':'
            << (result ? result->Size() : 0) << ':' << (result ? result->Capacity() : 0)
            << ':' << events << ':';
  if (result) for (const CopyItem& item : *result) std::cout << item.value << ',';
  std::cout << ':' << resources << ':' << !spec.owned << '\n';
}

void repair(const FillSpec& spec)
{
  const size_t capacity = spec.count < 2 ? 2 : spec.count;
  CopyItem* beginning = static_cast<CopyItem*>(firstAttempt);
  std::cout << "unpublished:" << spec.name << ':' << capacity << ':' << completed << ':';
  for (size_t i = 0; i < completed; ++i) std::cout << beginning[i].value << ',';
  std::cout << ':' << resources << '\n';
  // This explicit fixture-only recovery occurs after ordinary native failure observation.
  for (size_t i = 0; i < completed; ++i) beginning[i].~CopyItem();
  Standard::Free(firstAttempt); firstAttempt = nullptr;
}

void run(const FillSpec& spec)
{
  std::optional<CopyItem> donor;
  if (!spec.owned) donor.emplace(99);
  std::optional<Vector> result;
  attempts = completed = 0; failing = spec.fail; firstAttempt = nullptr; events.clear(); active = true;
  bool success = true;
  try
  {
    if (spec.owned) result.emplace(owned_fill(spec.count, CopyItem(99)));
    else result.emplace(spec.count, *donor);
  }
  catch (const std::runtime_error&) { success = false; }
  emit("native", spec, result ? &*result : nullptr, success);
  if (!success)
  {
    repair(spec);
    emit("repaired", spec, nullptr, false);
  }
  active = false;
}

int main()
{
  const FillSpec cases[] = {
    {"borrowed-zero",0,0,false},{"borrowed-one",1,0,false},{"borrowed-two",2,0,false},
    {"borrowed-three",3,0,false},{"borrowed-five",5,0,false},
    {"borrowed-one-first",1,1,false},{"borrowed-three-first",3,1,false},
    {"borrowed-middle",3,2,false},{"borrowed-final",3,3,false},{"borrowed-long-final",5,5,false},
    {"owned-zero",0,0,true},{"owned-one",1,0,true},{"owned-two",2,0,true},
    {"owned-three",3,0,true},{"owned-five",5,0,true},
    {"owned-one-first",1,1,true},{"owned-three-first",3,1,true},
    {"owned-middle",3,2,true},{"owned-final",3,3,true},{"owned-long-final",5,5,true},
  };
  for (const FillSpec& spec : cases) { run(spec); if (resources != 0) return 1; }
}
