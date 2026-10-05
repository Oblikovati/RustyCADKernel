#include <NCollection_LocalArray.hxx>
#include <NCollection_Array1.hxx>

#include <iostream>

void probe_const_view_alias()
{
  NCollection_LocalArray<int> anArray(2);
  int* aBuffer = anArray;
  aBuffer[0] = 12;
  aBuffer[1] = 20;
  const NCollection_LocalArray<int>& aConstArray = anArray;
  NCollection_Array1<int> aView = aConstArray.ToArray1();
  aView[1] = 77;
  std::cout << aBuffer[0] << ' ' << aBuffer[1] << '\n';
}

int main()
{
  probe_const_view_alias();
}
