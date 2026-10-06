// Non-owning elements prove native lifetime omission without leaking test resources.
#include <NCollection_LinearVector.hxx>
#include <cassert>
#include <functional>
#include <iostream>
#include <stdexcept>
#include <string>

static bool active = false;
static size_t serial = 0, failing = 0, expectedSize = 0;
static std::string events;
static std::function<size_t()> observedSize;

void tailConstructed(char phase, int value)
{
  if (!active) return;
  assert(observedSize() == expectedSize); // Native publishes only after all constructors.
  events += std::string(1, phase) + std::to_string(value) + ',';
  if (serial == failing) throw std::runtime_error("injected tail constructor failure");
}

struct TailItem
{
  int value;
  explicit TailItem(int initial) : value(initial) {}
  TailItem() : value(100 + int(++serial)) { tailConstructed('A', value); }
  TailItem(const TailItem& source) : value(source.value)
  { ++serial; tailConstructed('C', value); }
  TailItem(TailItem&& source) noexcept : value(source.value)
  {
    source.value = -1;
    if (active) events += "M" + std::to_string(value) + ',';
  }
  ~TailItem() { if (active) events += "D" + std::to_string(value) + ','; }
};

using Vector = NCollection_LinearVector<TailItem>;

void emit(const std::string& name, const char* phase, const Vector& vector)
{
  std::cout << name << ':' << phase << ':' << vector.Size() << ':' << vector.Capacity()
            << ':' << events << ':';
  for (const TailItem& item : vector) std::cout << item.value << ',';
  std::cout << '\n';
}

void attempt(Vector& vector, const TailItem& donor, bool fill, size_t size, size_t fail)
{
  failing = fail == 0 ? 0 : serial + fail;
  expectedSize = vector.Size();
  try { if (fill) vector.Resize(size, donor); else vector.Resize(size); }
  catch (const std::runtime_error&) { return; }
  assert(fail == 0);
}

void followup(const std::string& name, Vector& vector, const TailItem& donor,
              bool fill, const std::string& action)
{
  if (action == "clear-keep")
  {
    vector.Clear(false); emit(name, "cleared", vector);
    attempt(vector, donor, fill, 4, 0); emit(name, "published", vector);
  }
  else if (action == "clear-free") { vector.Clear(true); emit(name, "cleared", vector); }
  else if (action == "reserve") { vector.Reserve(9); emit(name, "reserved", vector); }
  else if (action == "retry-success")
  { attempt(vector, donor, fill, 5, 0); emit(name, "published", vector); }
  else if (action == "retry-first" || action == "retry-second")
  {
    attempt(vector, donor, fill, 5, action == "retry-first" ? 1 : 2);
    emit(name, "retry", vector);
    attempt(vector, donor, fill, 4, 0); emit(name, "published", vector);
  }
}

void run(size_t capacity, bool fill, size_t fail, const std::string& action)
{
  const std::string name = std::to_string(capacity) + '-' + (fill ? "fill" : "default")
                        + '-' + std::to_string(fail) + '-' + action;
  TailItem donor(99);
  {
    Vector vector(capacity);
    vector.EmplaceAppend(10); vector.EmplaceAppend(11);
    serial = 0; events.clear(); active = true;
    observedSize = [&vector] { return vector.Size(); };
    attempt(vector, donor, fill, 5, fail); emit(name, "failed", vector);
    followup(name, vector, donor, fill, action);
  }
  std::cout << name << ":freed:" << events << '\n';
  active = false; observedSize = {};
}

int main()
{
  for (size_t capacity : {size_t(2), size_t(8)})
    for (bool fill : {false, true})
      for (size_t fail : {size_t(1), size_t(2), size_t(3)})
        for (const char* action : {"clear-keep", "clear-free", "reserve", "drop",
                                  "retry-success", "retry-first", "retry-second"})
          run(capacity, fill, fail, action);
}
