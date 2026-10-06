#include <NCollection_Array1.hxx>
#include <Standard_DimensionMismatch.hxx>
#include <iostream>
#include <string>
#include <utility>

void array_pointer_ranges()
{
  NCollection_Array1<int> empty;
  int buffer[] = {11, 22, 33};
  NCollection_Array1<int> view(buffer, size_t(0));
  std::cout << "empty:" << (empty.Data() == nullptr) << ':' << (empty.end() == nullptr)
            << ':' << (view.Data() == buffer) << ':' << (view.end() == buffer) << '\n';
  NCollection_Array1<int> owned(-3, -1);
  const auto& constant = owned;
  const int* beginning = owned.Data();
  std::cout << "range:" << (beginning == &owned.First()) << ':' << (owned.begin() == beginning)
            << ':' << (owned.end() == beginning + 3) << ':' << (constant.Data() == beginning)
            << ':' << (constant.begin() == beginning) << ':' << (constant.cbegin() == beginning)
            << ':' << (constant.end() == beginning + 3) << ':' << (constant.cend() == beginning + 3);
  owned.UpdateLowerBound(100);
  std::cout << ':' << (owned.Data() == beginning) << '\n';
}

void array_assignment()
{
  NCollection_Array1<int> source(7, 9);
  for (int i = 7; i <= 9; ++i) source(i) = 10 + i - 7;
  NCollection_Array1<int> destination(-2, 0);
  const int* beginning = destination.Data();
  auto& assigned = destination.Assign(source);
  auto& self = destination.Assign(destination);
  std::cout << "assign:" << (&assigned == &destination) << ':' << (destination.Data() == beginning)
            << ':' << destination.Lower() << ':' << destination.Upper() << ':' << destination(8)
            << ':' << (&self == &destination) << '\n';
  destination.UpdateLowerBound(-2);
  auto& copied = destination.CopyValues(source);
  std::cout << "values:" << (&copied == &destination) << ':' << destination.Lower()
            << ':' << destination.Upper() << ':' << destination(-1) << '\n';
}

void array_detach_and_clone()
{
  NCollection_Array1<int> source(7, 9);
  for (int i = 7; i <= 9; ++i) source(i) = 10 + i - 7;
  int buffer[] = {99, 99};
  NCollection_Array1<int> destination(buffer, size_t(2));
  destination.Assign(source);
  NCollection_Array1<int> copied(destination);
  std::cout << "detach:" << destination.IsDeletable() << ':' << destination(9)
            << ':' << copied.Lower() << ':' << (copied.Data() != destination.Data())
            << ':' << copied(8) << ':' << buffer[0] << '\n';
  try { destination.CopyValues(NCollection_Array1<int>(size_t(2))); }
  catch (const Standard_DimensionMismatch&) { std::cout << "mismatch:1\n"; }
}

void array_owned_returning()
{
  NCollection_Array1<std::string> source(7, 7), destination(size_t(2));
  source.Init("source");
  auto& assigned = destination.Assign(source);
  destination.UpdateLowerBound(-2);
  auto& copied = destination.CopyValues(source);
  std::cout << "owned:" << (&assigned == &destination) << ':' << (&copied == &destination)
            << ':' << destination.Lower() << ':' << (destination(-2) == "source") << '\n';
}

void array_borrowed_move()
{
  int buffer[] = {11, 22};
  NCollection_Array1<int> source(buffer, size_t(2)), destination(size_t(3));
  source.UpdateLowerBound(-3);
  auto& moved = destination.Move(source);
  std::cout << "move:" << (&moved == &destination) << ':' << (destination.Data() == buffer)
            << ':' << destination.IsDeletable() << ':' << destination.Lower()
            << ':' << (source.Data() == nullptr) << ':' << source.Lower() << ':' << source.Upper() << '\n';
}

int main()
{
  array_pointer_ranges();
  array_assignment();
  array_detach_and_clone();
  array_owned_returning();
  array_borrowed_move();
}
