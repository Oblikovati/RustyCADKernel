#include <NCollection_Array2.hxx>

#include <array>
#include <iostream>

struct CopyProbe
{
  int   value;
  int*  assignments;
  int*  constructions;

  CopyProbe() : value(0), assignments(nullptr), constructions(nullptr) {}

  CopyProbe(const CopyProbe& theOther)
      : value(theOther.value), assignments(theOther.assignments), constructions(theOther.constructions)
  {
    if (constructions != nullptr)
    {
      ++*constructions;
    }
  }

  CopyProbe& operator=(const CopyProbe& theOther)
  {
    ++*theOther.assignments;
    value         = theOther.value;
    assignments   = theOther.assignments;
    constructions = theOther.constructions;
    return *this;
  }
};

int main()
{
  int assignments = 0;
  int constructions = 0;
  std::array<CopyProbe, 6> buffer{};
  for (int index = 0; index < 6; ++index)
  {
    buffer[index].value = 100 + index;
    buffer[index].assignments = &assignments;
    buffer[index].constructions = &constructions;
  }
  NCollection_Array2<CopyProbe> array(buffer.data(), 2, 2);
  CopyProbe fill;
  fill.value = 7;
  fill.assignments = &assignments;
  fill.constructions = &constructions;
  array.Init(fill);
  array.ChangeAt(0, 0).value = 20;
  std::cout << "init_assignments=" << assignments << '\n';
  std::cout << "copy_constructions=" << constructions << '\n';
  std::cout << "values=" << array.At(0).value << ',' << array.At(1).value << ','
            << array.At(2).value << ',' << array.At(3).value << '\n';
  std::cout << "tail=" << buffer[4].value << ',' << buffer[5].value << '\n';
  NCollection_Array2<CopyProbe> anEmptyArray(buffer.data(), 0, 5);
  std::cout << "empty_size=" << anEmptyArray.Size() << '\n';
}
