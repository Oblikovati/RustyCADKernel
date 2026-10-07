#include <NCollection_Array1.hxx>
#include <iostream>
#include <sstream>
#include <stdexcept>
static std::ostringstream trace;
static int calls = 0, completed = 0, failAt = -1;
struct DefaultMember
{
  int index;
  explicit DefaultMember(int input) : index(input)
  {
    trace << 'M' << index << ',';
  }
  ~DefaultMember()
  {
    trace << 'G' << index << ',';
  }
};
struct DefaultCell
{
  DefaultMember member;
  DefaultCell() : member(calls++)
  {
    trace << 'C' << member.index << ',';
    if (member.index == failAt) throw std::runtime_error("default construction");
    ++completed;
  }
  DefaultCell(const DefaultCell&) = delete;
  DefaultCell& operator=(const DefaultCell&) = delete;
  ~DefaultCell()
  {
    trace << 'D' << member.index << ',';
  }
};
struct DefaultArray : NCollection_Array1<DefaultCell>
{
  using NCollection_Array1<DefaultCell>::construct;
};
static void finish(DefaultCell* slots, size_t from)
{
  for (int i = 0; i < completed; ++i) std::cout << (i ? "," : "") << slots[from + i].member.index;
  std::cout << ';'; trace.str("");
  for (int i = 0; i < completed; ++i) slots[from + i].~DefaultCell();
  std::cout << trace.str() << '\n';
  ::operator delete(slots);
}
static void rangeCase(const char* name, size_t from, size_t to, int failure)
{
  auto* slots = static_cast<DefaultCell*>(::operator new(4 * sizeof(DefaultCell)));
  trace.str(""); calls = 0; completed = 0; failAt = failure;
  DefaultArray array;
  bool failed = false;
  try { array.construct(from, to, slots); }
  catch (const std::runtime_error&) { failed = true; }
  std::cout << name << '=' << trace.str() << ';' << failed << ';' << completed << ';';
  finish(slots, from);
}
static void emptyPointers()
{
  DefaultArray array;
  trace.str("");
  array.construct(size_t(0), size_t(0), static_cast<DefaultCell*>(nullptr));
  array.construct(size_t(-1), size_t(0), static_cast<DefaultCell*>(nullptr));
  std::cout << "null_empty=" << trace.str() << '\n';
}
int main()
{
  rangeCase("all", 0, 4, -1);
  rangeCase("middle", 1, 3, -1);
  rangeCase("fail_first", 1, 4, 0);
  rangeCase("fail_late", 1, 4, 2);
  rangeCase("empty", 2, 2, -1);
  rangeCase("reverse", 3, 1, -1);
  emptyPointers();
}
