// Copy assignment retains unpublished tail objects. Non-owning payloads expose omitted drops.
#include <NCollection_LinearVector.hxx>
#include <cassert>
#include <functional>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

static bool active = false;
static size_t copies = 0, assignments = 0, failCopy = 0, failAssign = 0, expectedSize = 0;
static std::string events;
static std::vector<const void*> originalSlots;
static std::function<size_t()> observedSize;

void copied(int value)
{
  if (!active) return;
  assert(observedSize() == expectedSize);
  events += "C" + std::to_string(value) + ',';
  if (++copies == failCopy) throw std::runtime_error("injected tail copy failure");
}

void assigned(int previous, int value)
{
  if (!active) return;
  assert(observedSize() == expectedSize);
  events += "A" + std::to_string(previous) + '>' + std::to_string(value) + ',';
  if (++assignments == failAssign) throw std::runtime_error("injected common assignment failure");
}

void destroyed(const void* address, int value)
{
  if (!active) return;
  size_t index = 0;
  while (index < originalSlots.size() && originalSlots[index] != address) ++index;
  events += "D" + std::to_string(value) + '@'
          + (index == originalSlots.size() ? "X" : std::to_string(index)) + ',';
}

struct AssignmentItem
{
  int value;
  explicit AssignmentItem(int initial) : value(initial) {}
  AssignmentItem(const AssignmentItem& source) : value(source.value) { copied(value); }
  AssignmentItem(AssignmentItem&& source) noexcept : value(source.value)
  {
    source.value = -1;
    if (active) events += "M" + std::to_string(value) + ',';
  }
  AssignmentItem& operator=(const AssignmentItem& source)
  { int previous = value; value = source.value; assigned(previous, value); return *this; }
  ~AssignmentItem() { destroyed(this, value); }
};
using Vector = NCollection_LinearVector<AssignmentItem>;

bool attempt(Vector& target, const Vector& source, size_t copyFailure, size_t assignmentFailure)
{
  failCopy = copyFailure == 0 ? 0 : copies + copyFailure;
  failAssign = assignmentFailure == 0 ? 0 : assignments + assignmentFailure;
  expectedSize = target.Size();
  try { assert(&(target = source) == &target); return true; }
  catch (const std::runtime_error&) { return false; }
}

void emit(const std::string& name, const char* phase, const Vector& target,
          const Vector& source, const AssignmentItem* original, bool success)
{
  std::cout << name << ':' << phase << ':' << (success ? "ok" : "throw") << ':'
            << target.Size() << ':' << target.Capacity() << ':' << (original == target.Data())
            << ':' << events << ':';
  for (const AssignmentItem& item : target) std::cout << item.value << ',';
  std::cout << ':';
  for (const AssignmentItem& item : source) std::cout << item.value << ',';
  std::cout << '\n';
}

void followup(const std::string& name, Vector& target, const Vector& source,
              const AssignmentItem* original, const std::string& action)
{
  if (action == "clear-keep" || action == "clear-free")
  {
    target.Clear(action == "clear-free"); emit(name, "cleared", target, source, original, true);
    if (action == "clear-keep")
      emit(name, "published", target, source, original, attempt(target, source, 0, 0));
  }
  else if (action == "reserve")
  { target.Reserve(9); emit(name, "reserved", target, source, original, true); }
  else if (action == "retry-success")
    emit(name, "published", target, source, original, attempt(target, source, 0, 0));
  else if (action == "retry-first" || action == "retry-second")
  {
    bool success = attempt(target, source, action == "retry-first" ? 1 : 2, 0);
    emit(name, "retry", target, source, original, success);
    emit(name, "published", target, source, original, attempt(target, source, 0, 0));
  }
}

void run(const std::string& name, size_t capacity, size_t count, size_t sourceCount,
         size_t copyFailure, size_t assignmentFailure, const std::string& action)
{
  Vector source(sourceCount);
  for (size_t i = 0; i < sourceCount; ++i) source.EmplaceAppend(10 + i);
  {
    Vector target(capacity);
    for (size_t i = 0; i < count; ++i) target.EmplaceAppend(100 + i);
    const AssignmentItem* original = target.Data(); originalSlots.clear();
    for (size_t i = 0; i < capacity; ++i) originalSlots.push_back(original + i);
    copies = assignments = 0; events.clear(); active = true;
    observedSize = [&target] { return target.Size(); };
    bool success = attempt(target, source, copyFailure, assignmentFailure);
    emit(name, "assigned", target, source, original, success);
    followup(name, target, source, original, action);
  }
  std::cout << name << ":freed:" << events << '\n';
  active = false; observedSize = {};
}

int main()
{
  for (size_t count : {size_t(0), size_t(2)})
    for (size_t fail : {size_t(1), size_t(2), size_t(3)})
      for (const char* action : {"clear-keep", "clear-free", "reserve", "drop",
                                "retry-success", "retry-first", "retry-second"})
        run(std::to_string(count) + '-' + std::to_string(fail) + '-' + action,
            8, count, count + 3, fail, 0, action);
  for (size_t fail : {size_t(1), size_t(2)})
    for (const char* action : {"drop", "retry-success"})
      run("common-" + std::to_string(fail) + '-' + action, 8, 2, 5, 0, fail, action);
  run("equal", 8, 3, 3, 0, 0, "drop");
  run("shrink", 8, 5, 3, 0, 0, "drop");
  run("empty-source", 8, 3, 0, 0, 0, "drop");
  run("empty-both", 0, 0, 0, 0, 0, "drop");
  run("grow-success", 1, 1, 5, 0, 0, "drop");
}
