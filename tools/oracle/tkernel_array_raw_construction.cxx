#include <NCollection_Array1.hxx>
#include <iostream>
#include <sstream>
#include <stdexcept>
static std::ostringstream trace;
static int calls = 0, completed = 0, deaths = 0, failAt = -1;
struct ConstructionCell
{
  int value;
  explicit ConstructionCell(int v) : value(v) {}
  ConstructionCell() = delete;
  ConstructionCell& operator=(const ConstructionCell&) = delete;
  ConstructionCell(const ConstructionCell& source) : value(source.value + 10)
  {
    trace << 'C' << source.value << ',';
    if (calls++ == failAt) throw std::runtime_error("construction");
    ++completed;
  }
  ~ConstructionCell() { trace << 'D' << value << ','; ++deaths; }
};
struct ConstructionArray : NCollection_Array1<ConstructionCell>
{
  using NCollection_Array1<ConstructionCell>::copyConstruct;
};
static void rawCase(const char* name, int failure, size_t count)
{
  ConstructionCell source[] = {ConstructionCell(1), ConstructionCell(2), ConstructionCell(3)};
  alignas(ConstructionCell) unsigned char bytes[3 * sizeof(ConstructionCell)];
  auto* target = reinterpret_cast<ConstructionCell*>(bytes);
  ConstructionArray array;
  calls = 0; completed = 0; deaths = 0; failAt = failure; trace.str("");
  bool failed = false;
  try { array.copyConstruct(target, source, count); }
  catch (const std::runtime_error&) { failed = true; }
  std::cout << name << '=';
  for (int i = 0; i < completed; ++i) std::cout << (i ? "," : "") << target[i].value;
  std::cout << ';' << trace.str() << ';' << failed << ';' << completed << ';' << deaths << ';';
  trace.str("");
  for (int i = 0; i < completed; ++i) target[i].~ConstructionCell();
  std::cout << trace.str() << '\n';
}
static void zeroCase()
{
  ConstructionArray array;
  calls = 0;
  array.copyConstruct(static_cast<ConstructionCell*>(nullptr),
                      static_cast<const ConstructionCell*>(nullptr), size_t(0));
  std::cout << "zero_null=" << calls << '\n';
}
int main()
{
  rawCase("success", -1, 3);
  rawCase("fail_first", 0, 3);
  rawCase("fail_late", 2, 3);
  rawCase("empty", -1, 0);
  zeroCase();
}
