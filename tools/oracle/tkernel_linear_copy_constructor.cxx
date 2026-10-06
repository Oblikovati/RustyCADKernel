// Failed native copy construction keeps raw allocation and completed payloads alive.
#include <NCollection_LinearVector.hxx>
#include <Standard.hxx>
#include <iostream>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>

static bool active = false;
static size_t attempts = 0, completed = 0, failing = 0;
static int resources = 0;
static std::string events;
static void* firstAttempt = nullptr;

struct CopyPayload
{
  int value;
  explicit CopyPayload(int initial) : value(initial) { ++resources; }
  ~CopyPayload() { --resources; }
};
struct CopyItem
{
  int value;
  std::unique_ptr<CopyPayload> payload;
  explicit CopyItem(int initial) : value(initial), payload(new CopyPayload(initial)) {}
  CopyItem(const CopyItem& source) : value(source.value), payload(new CopyPayload(source.payload->value))
  {
    if (!active) return;
    if (firstAttempt == nullptr) firstAttempt = this;
    events += "C" + std::to_string(value) + ',';
    if (++attempts == failing) throw std::runtime_error("injected copy-constructor failure");
    ++completed;
  }
  CopyItem& operator=(const CopyItem& source)
  { value = source.value; payload.reset(new CopyPayload(source.payload->value)); return *this; }
  ~CopyItem() { if (active) events += "D" + std::to_string(value) + ','; }
};
using Vector = NCollection_LinearVector<CopyItem>;
struct Spec { const char* name; size_t capacity, count, fail; bool assigning; size_t receiverCount; };

void emit(const char* phase, const Spec& spec, const Vector& source,
          const Vector* result, const CopyItem* before, bool success)
{
  std::cout << phase << ':' << spec.name << ':' << (success ? "ok" : "throw") << ':'
            << (result ? result->Size() : 0) << ':' << (result ? result->Capacity() : 0)
            << ':' << (result && result->Data() == before) << ':' << events << ':';
  for (const CopyItem& item : source) std::cout << item.value << ',';
  std::cout << ':';
  if (result) for (const CopyItem& item : *result) std::cout << item.value << ',';
  std::cout << ':' << resources << '\n';
}

void repair(const Spec& spec)
{
  CopyItem* beginning = static_cast<CopyItem*>(firstAttempt);
  std::cout << "unpublished:" << spec.name << ':' << spec.count << ':' << completed << ':';
  for (size_t i = 0; i < completed; ++i) std::cout << beginning[i].value << ',';
  std::cout << ':' << resources << '\n';
  // Explicit fixture-only recovery follows the unchanged native failure observation.
  for (size_t i = 0; i < completed; ++i) beginning[i].~CopyItem();
  Standard::Free(firstAttempt); firstAttempt = nullptr;
}

void run(const Spec& spec)
{
  Vector source(spec.capacity), receiver(spec.receiverCount);
  for (size_t i = 0; i < spec.count; ++i) source.EmplaceAppend(10 + i);
  for (size_t i = 0; i < spec.receiverCount; ++i) receiver.EmplaceAppend(90 + i);
  const CopyItem* before = spec.assigning ? receiver.Data() : source.Data();
  std::optional<Vector> copied;
  attempts = completed = 0; failing = spec.fail; firstAttempt = nullptr; events.clear(); active = true;
  bool success = true;
  try { if (spec.assigning) receiver = source; else copied.emplace(source); }
  catch (const std::runtime_error&) { success = false; }
  const Vector* result = spec.assigning ? &receiver : (copied ? &*copied : nullptr);
  emit("native", spec, source, result, before, success);
  if (!success)
  {
    repair(spec);
    emit("repaired", spec, source, result, before, success);
  }
  active = false;
}

int main()
{
  const Spec cases[] = {
    {"clone-empty",0,0,0,false,0}, {"clone-reserved-empty",8,0,0,false,0},
    {"clone-single",8,1,0,false,0}, {"clone-retained",8,3,0,false,0},
    {"clone-first",8,3,1,false,0}, {"clone-middle",8,3,2,false,0},
    {"clone-last",8,3,3,false,0}, {"clone-long-final",8,5,5,false,0},
    {"assign-empty-target",8,3,0,true,0}, {"assign-occupied",8,3,0,true,1},
    {"assign-first",8,3,1,true,1}, {"assign-middle",8,3,2,true,1},
    {"assign-last",8,3,3,true,1}, {"assign-empty-first",8,1,1,true,0},
  };
  for (const Spec& spec : cases) { run(spec); if (resources != 0) return 1; }
}
