#include <NCollection_Array1.hxx>
#include <iostream>
#include <stdexcept>
#include <vector>

struct AssignmentProbe
{
  int slot = 0;
  int value = 0;
  std::vector<int>* assignments = nullptr;
  AssignmentProbe() = default;
  AssignmentProbe(int id, std::vector<int>& trace) : slot(id), value(id), assignments(&trace) {}
  AssignmentProbe(const AssignmentProbe&) { throw std::runtime_error("unexpected copy construction"); }
  AssignmentProbe& operator=(const AssignmentProbe& source)
  {
    assignments->push_back(slot);
    value = source.value;
    return *this;
  }
};

int main()
{
  std::vector<int> assignments;
  AssignmentProbe buffer[] = {{1, assignments}, {2, assignments}, {3, assignments}};
  const AssignmentProbe source(9, assignments);
  NCollection_Array1<AssignmentProbe> array(buffer[0], -1, 1);
  array.Init(source);
  for (int slot : assignments) std::cout << slot << ' ';
  std::cout << '\n';
  for (const auto& element : array) std::cout << element.value << ' ';
  std::cout << '\n' << array.Lower() << ' ' << array.Upper() << ' ' << source.value << '\n';
  assignments.clear();
  NCollection_Array1<AssignmentProbe> empty;
  empty.Init(source);
  std::cout << assignments.size() << ' ' << empty.Lower() << ' ' << empty.Upper()
            << ' ' << source.value << '\n';
}
