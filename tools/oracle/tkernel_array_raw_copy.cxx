#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <Standard_DimensionMismatch.hxx>
#include <iostream>

static void printBuffer(const int* values)
{
  for (int i = 0; i < 6; ++i) std::cout << (i == 0 ? "" : ",") << values[i];
}

struct RawArray1 : NCollection_Array1<int>
{
  using NCollection_Array1<int>::copyAssign;
};

static void copyRaw(const char* label, int destinationOffset, int sourceOffset)
{
  int buffer[] = {0, 1, 2, 3, 4, 5};
  RawArray1 array;
  array.copyAssign(buffer + destinationOffset, buffer + sourceOffset, 4);
  std::cout << label << "=";
  printBuffer(buffer);
  std::cout << "\n";
}

static void sameOwned()
{
  NCollection_Array1<int> destination(-2, 1);
  for (size_t i = 0; i < 4; ++i) destination.ChangeAt(i) = int(i);
  NCollection_Array1<int> source(destination.Data(), size_t(4));
  destination.CopyValues(source);
  std::cout << "array1_same_owned=0,1,2,3;" << destination.Lower() << ","
            << destination.Upper() << ";" << destination.IsDeletable() << "\n";
}

static void reshapeSame()
{
  int buffer[] = {0, 1, 2, 3};
  NCollection_Array2<int> destination(buffer, size_t(2), size_t(2));
  NCollection_Array2<int> source(buffer[0], 7, 7, 10, 13);
  destination.Assign(source);
  std::cout << "array2_reshape_same=0,1,2,3;" << destination.LowerRow() << ","
            << destination.UpperRow() << "," << destination.LowerCol() << ","
            << destination.UpperCol() << ";" << destination.IsDeletable() << "\n";
}

static void internalAssign1()
{
  NCollection_Array1<int> destination(size_t(6));
  for (size_t i = 0; i < 6; ++i) destination.ChangeAt(i) = int(i);
  NCollection_Array1<int> source(destination.At(1), -2, 1);
  destination.Assign(source);
  std::cout << "array1_internal_assign=";
  for (size_t i = 0; i < 4; ++i) std::cout << (i == 0 ? "" : ",") << destination.At(i);
  std::cout << ";" << destination.Lower() << "," << destination.Upper()
            << ";" << destination.IsDeletable() << "\n";
}

static void internalAssign2()
{
  NCollection_Array2<int> destination(size_t(2), size_t(3));
  for (size_t i = 0; i < 6; ++i) destination.ChangeAt(i) = int(i);
  NCollection_Array2<int> source(destination.At(1), -2, -1, 3, 4);
  destination.Assign(source);
  std::cout << "array2_internal_assign=";
  for (size_t i = 0; i < 4; ++i) std::cout << (i == 0 ? "" : ",") << destination.At(i);
  std::cout << ";" << destination.LowerRow() << "," << destination.UpperRow() << ","
            << destination.LowerCol() << "," << destination.UpperCol() << ";"
            << destination.IsDeletable() << "\n";
}

static void mismatch1()
{
  int buffer[] = {0, 1, 2, 3, 4, 5};
  NCollection_Array1<int> destination(buffer, size_t(4));
  NCollection_Array1<int> source(buffer, size_t(3));
  try { destination.CopyValues(source); }
  catch (const Standard_DimensionMismatch&) { std::cout << "array1_mismatch=dimension\n"; }
}

static void mismatch2()
{
  int buffer[] = {0, 1, 2, 3, 4, 5};
  NCollection_Array2<int> destination(buffer, size_t(2), size_t(2));
  NCollection_Array2<int> source(buffer, size_t(1), size_t(4));
  try { destination.CopyValues(source); }
  catch (const Standard_DimensionMismatch&) { std::cout << "array2_mismatch=shape\n"; }
}

static void emptyCopies()
{
  NCollection_Array1<int> empty1(static_cast<int*>(nullptr), size_t(0));
  NCollection_Array1<int> source1;
  empty1.CopyValues(source1);
  std::cout << "array1_empty_copy=" << empty1.Lower() << "," << empty1.Upper()
            << ";" << (empty1.Data() == nullptr) << ";" << empty1.IsDeletable() << "\n";
  NCollection_Array2<int> empty2(static_cast<int*>(nullptr), size_t(0), size_t(3));
  NCollection_Array2<int> source2(static_cast<int*>(nullptr), size_t(0), size_t(3));
  empty2.CopyValues(source2);
  std::cout << "array2_empty_copy=" << empty2.NbRows() << "," << empty2.NbColumns()
            << ";" << (empty2.Data() == nullptr) << ";" << empty2.IsDeletable() << "\n";
}

int main()
{
  copyRaw("raw_forward", 2, 0);
  copyRaw("raw_backward", 0, 2);
  copyRaw("raw_same", 1, 1);
  sameOwned();
  reshapeSame();
  internalAssign1();
  internalAssign2();
  mismatch1();
  mismatch2();
  emptyCopies();
}
