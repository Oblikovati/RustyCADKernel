#include <NCollection_Array1.hxx>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>

static std::ostringstream operations;
static int failAt = -1, calls = 0, deaths = 0;
struct AssignmentCell
{
  int value;
  explicit AssignmentCell(int input) : value(input) {}
  AssignmentCell(const AssignmentCell&) = delete;
  ~AssignmentCell() { ++deaths; }
  AssignmentCell& operator=(const AssignmentCell& donor)
  {
    operations << (this == &donor ? 'S' : 'A') << value << ':' << donor.value << ',';
    value = donor.value + (this == &donor ? 10 : 0);
    if (calls++ == failAt) throw std::runtime_error("assignment");
    return *this;
  }
};
struct NontrivialRawArray : NCollection_Array1<AssignmentCell>
{
  using NCollection_Array1<AssignmentCell>::copyAssign;
};

static void rawCase(const char* label, int target, int source, int failure)
{
  operations.str(""); calls = 0; deaths = 0; failAt = failure;
  AssignmentCell cells[] = {AssignmentCell(1), AssignmentCell(2), AssignmentCell(3),
                            AssignmentCell(4), AssignmentCell(5)};
  NontrivialRawArray array;
  bool failed = false;
  try { array.copyAssign(cells + target, cells + source, size_t(4)); }
  catch (const std::runtime_error&) { failed = true; }
  std::cout << label << '=';
  for (int i = 0; i < 5; ++i) std::cout << (i ? "," : "") << cells[i].value;
  std::cout << ';' << operations.str() << ';' << failed << ';' << deaths << '\n';
}

static void zeroCase()
{
  NontrivialRawArray array;
  calls = 0;
  array.copyAssign(static_cast<AssignmentCell*>(nullptr),
                   static_cast<const AssignmentCell*>(nullptr), size_t(0));
  std::cout << "zero=" << calls << '\n';
}

int main()
{
  rawCase("forward", 1, 0, -1);
  rawCase("reverse", 0, 1, -1);
  rawCase("same", 0, 0, -1);
  rawCase("forward_fail_first", 1, 0, 0);
  rawCase("forward_fail_late", 1, 0, 2);
  rawCase("reverse_fail_late", 0, 1, 2);
  rawCase("same_fail_late", 0, 0, 2);
  zeroCase();
}
