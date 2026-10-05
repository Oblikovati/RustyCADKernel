// A throwing first gap default must prevent the requested value constructor.
#include <NCollection_DynamicArray.hxx>
#include <iostream>

struct FailingDefault {
  FailingDefault() { throw 17; }
  explicit FailingDefault(bool& called) { called = true; }
};
int main() {
  bool called = false;
  bool caught = false;
  NCollection_DynamicArray<FailingDefault> array(4);
  try { array.EmplaceValue(1, called); }
  catch (int value) { caught = value == 17; }
  std::cout << "caught=" << caught << " called=" << called
            << " length=" << array.Length() << '\n';
}
