#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <Standard_OutOfRange.hxx>

#include <iostream>

struct Probe {
  int identity = 0;
  int value = 0;
  int* assignments = nullptr;
  int* constructions = nullptr;
  Probe() = default;
  Probe(const Probe& other)
      : identity(other.identity), value(other.value), assignments(other.assignments),
        constructions(other.constructions) {
    if (constructions) ++*constructions;
  }
  Probe& operator=(const Probe& other) {
    if (other.assignments) ++*other.assignments;
    value = other.value;
    assignments = other.assignments;
    constructions = other.constructions;
    return *this;
  }
};

void initialize(Probe& probe, int identity, int value, int* assignments, int* constructions) {
  probe.identity = identity;
  probe.value = value;
  probe.assignments = assignments;
  probe.constructions = constructions;
}

void probeArray1(int& assignments, int& constructions, Probe& source) {
  NCollection_Array1<Probe> array1(0, 1);
  initialize(array1.ChangeValue(0), 1, 10, &assignments, &constructions);
  initialize(array1.ChangeValue(1), 2, 20, &assignments, &constructions);
  const Probe* target1 = &array1.ChangeValue(0);
  array1.SetValue(0, source);
  std::cout << "array1=" << array1.Value(0).identity << ',' << array1.Value(0).value
            << ",source=" << source.value << ",same=" << (target1 == &array1.Value(0)) << '\n';
  bool array1OutOfRange = false;
  try { array1.SetValue(2, source); } catch (const Standard_OutOfRange&) { array1OutOfRange = true; }
  std::cout << "array1_out_of_range=" << array1OutOfRange << '\n';
}

void probeArray2(int& assignments, int& constructions, Probe& source) {
  NCollection_Array2<Probe> array2(0, 0, 0, 1);
  initialize(array2.ChangeValue(0, 0), 4, 40, &assignments, &constructions);
  initialize(array2.ChangeValue(0, 1), 5, 50, &assignments, &constructions);
  const Probe* target2 = &array2.ChangeValue(0, 1);
  array2.SetValue(0, 1, source);
  std::cout << "array2=" << array2.Value(0, 1).identity << ',' << array2.Value(0, 1).value
            << ",same=" << (target2 == &array2.Value(0, 1)) << '\n';
  bool array2OutOfRange = false;
  try { array2.SetValue(1, 0, source); } catch (const Standard_OutOfRange&) { array2OutOfRange = true; }
  std::cout << "array2_out_of_range=" << array2OutOfRange << '\n';
  bool array2BadColumn = false;
  try { array2.SetValue(0, 2, source); } catch (const Standard_OutOfRange&) { array2BadColumn = true; }
  std::cout << "array2_bad_column=" << array2BadColumn << '\n';
}

int main() {
  int assignments = 0;
  int constructions = 0;
  Probe source;
  initialize(source, 3, 99, &assignments, &constructions);
  probeArray1(assignments, constructions, source);
  probeArray2(assignments, constructions, source);
  std::cout << "assignments=" << assignments << '\n';
  std::cout << "copy_constructions=" << constructions << '\n';
}
