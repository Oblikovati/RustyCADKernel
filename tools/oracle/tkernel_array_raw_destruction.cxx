#include <NCollection_Array1.hxx>
#include <array>
#include <iostream>
#include <sstream>
#include <stdexcept>
static std::ostringstream trace;
static std::array<bool, 4> live;
static int failAt = -1;
struct DestructionMember
{
  int index;
  ~DestructionMember()
  {
    trace << 'G' << index << ',';
  }
};
struct DestructionCell
{
  int index;
  DestructionMember member;
  explicit DestructionCell(int input) : index(input), member{input}
  {
    live[input] = true;
  }
  DestructionCell(const DestructionCell&) = delete;
  DestructionCell& operator=(const DestructionCell&) = delete;
  ~DestructionCell() noexcept(false)
  {
    trace << 'D' << index << ',';
    live[index] = false;
    if (index == failAt) throw std::runtime_error("destruction");
  }
};
struct DestructionArray : NCollection_Array1<DestructionCell>
{
  using NCollection_Array1<DestructionCell>::destroy;
};
static void printSurvivors(DestructionCell* slots)
{
  bool comma = false;
  for (int i = 0; i < 4; ++i)
  {
    if (!live[i]) continue;
    std::cout << (comma ? "," : "") << slots[i].index;
    comma = true;
  }
}
static void cleanup(DestructionCell* slots)
{
  failAt = -1;
  trace.str("");
  for (int i = 0; i < 4; ++i) if (live[i]) slots[i].~DestructionCell();
  std::cout << trace.str() << '\n';
  ::operator delete(slots);
}
static void rangeCase(const char* name, size_t from, size_t to, int failure)
{
  auto* slots = static_cast<DestructionCell*>(::operator new(4 * sizeof(DestructionCell)));
  for (int i = 0; i < 4; ++i) ::new (static_cast<void*>(slots + i)) DestructionCell(i);
  trace.str(""); failAt = failure;
  DestructionArray array;
  bool failed = false;
  try { array.destroy(slots, from, to); }
  catch (const std::runtime_error&) { failed = true; }
  std::cout << name << '=' << trace.str() << ';' << failed << ';';
  for (bool alive : live) std::cout << alive;
  std::cout << ';'; printSurvivors(slots); std::cout << ';';
  cleanup(slots);
}
static void emptyPointers()
{
  DestructionArray array;
  trace.str("");
  array.destroy(static_cast<DestructionCell*>(nullptr), size_t(0), size_t(0));
  array.destroy(static_cast<DestructionCell*>(nullptr), size_t(-1), size_t(0));
  std::cout << "null_empty=" << trace.str() << '\n';
}
int main()
{
  rangeCase("all", 0, 4, -1);
  rangeCase("middle", 1, 3, -1);
  rangeCase("fail_first", 1, 4, 1);
  rangeCase("fail_late", 0, 4, 2);
  rangeCase("empty", 2, 2, -1);
  rangeCase("reverse", 3, 1, -1);
  emptyPointers();
}
