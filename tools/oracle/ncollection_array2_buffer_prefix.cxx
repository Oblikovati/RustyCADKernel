#include <NCollection_Array2.hxx>

#include <array>
#include <iostream>

struct CopyProbe
{
  int value;
  int* assignments;

  CopyProbe& operator=(const CopyProbe& theOther)
  {
    ++*theOther.assignments;
    value       = theOther.value;
    assignments = theOther.assignments;
    return *this;
  }
};

int main()
{
  int assignments = 0;
  std::array<CopyProbe, 6> buffer{{{100, &assignments}, {101, &assignments},
                                   {102, &assignments}, {103, &assignments},
                                   {104, &assignments}, {105, &assignments}}};
  NCollection_Array2<CopyProbe> array(buffer.data(), 2, 2);
  const CopyProbe fill{7, &assignments};
  array.Init(fill);
  array.ChangeAt(0, 0).value = 20;
  std::cout << "init_assignments=" << assignments << '\n';
  std::cout << "values=" << array.At(0).value << ',' << array.At(1).value << ','
            << array.At(2).value << ',' << array.At(3).value << '\n';
  std::cout << "tail=" << buffer[4].value << ',' << buffer[5].value << '\n';
}
