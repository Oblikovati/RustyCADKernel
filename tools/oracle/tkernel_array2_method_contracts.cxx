#include <NCollection_Array2.hxx>
#include <Standard_DimensionMismatch.hxx>
#include <iostream>
#include <string>

void rectangle_pointer_ranges()
{
  NCollection_Array2<int> empty;
  int buffer[] = {11, 22};
  NCollection_Array2<int> borrowed(buffer, size_t(0), size_t(2));
  std::cout << "empty:" << (empty.Data() == nullptr) << ':' << (empty.end() == nullptr)
            << ':' << (borrowed.Data() == buffer) << ':' << (borrowed.end() == buffer) << '\n';
  NCollection_Array2<int> owned(-3, -2, 7, 9);
  const auto& constant = owned;
  const int* beginning = owned.Data();
  std::cout << "range:" << (beginning == &owned.Value(-3, 7)) << ':' << (owned.begin() == beginning)
            << ':' << (owned.end() == beginning + 6) << ':' << (constant.Data() == beginning)
            << ':' << (constant.begin() == beginning) << ':' << (constant.cbegin() == beginning)
            << ':' << (constant.end() == beginning + 6) << ':' << (constant.cend() == beginning + 6) << '\n';
}

void rectangle_assignment()
{
  NCollection_Array2<int> source(size_t(3), size_t(2)), destination(size_t(2), size_t(3));
  for (int i = 0; i < 6; ++i) source.Data()[i] = 10 + i;
  source.UpdateLowerRow(7); source.UpdateLowerCol(-3);
  const int* beginning = destination.Data();
  auto& assigned = destination.Assign(source);
  auto& self = destination.Assign(destination);
  std::cout << "assign:" << (&assigned == &destination) << ':' << (destination.Data() == beginning)
            << ':' << destination.LowerRow() << ':' << destination.UpperRow()
            << ':' << destination.LowerCol() << ':' << destination.UpperCol()
            << ':' << destination.Value(9, -2) << ':' << (&self == &destination) << '\n';
}

void rectangle_value_copy()
{
  NCollection_Array2<int> source(size_t(3), size_t(2)), destination(size_t(2), size_t(3));
  try { destination.CopyValues(source); }
  catch (const Standard_DimensionMismatch&) { std::cout << "shape:1\n"; }
  NCollection_Array2<int> matching(size_t(2), size_t(3));
  for (int i = 0; i < 6; ++i) matching.Data()[i] = 10 + i;
  auto& returned = destination.CopyValues(matching);
  NCollection_Array2<int> copied(destination);
  std::cout << "values:" << (&returned == &destination) << ':' << (copied.Data() != destination.Data())
            << ':' << copied.Value(1, 2) << '\n';
}

void rectangle_owned_returning()
{
  NCollection_Array2<std::string> source(size_t(1), size_t(2)), destination(size_t(2), size_t(3));
  source.Init("source");
  auto& assigned = destination.Assign(source);
  destination.UpdateLowerRow(-3);
  auto& copied = destination.CopyValues(source);
  std::cout << "owned:" << (&assigned == &destination) << ':' << (&copied == &destination)
            << ':' << destination.LowerRow() << ':' << (destination.Value(-3, 0) == "source") << '\n';
}

void rectangle_borrowed_move()
{
  int buffer[] = {11, 22};
  NCollection_Array2<int> source(buffer, size_t(1), size_t(2)), destination(size_t(2), size_t(3));
  source.UpdateLowerRow(-3);
  auto& returned = destination.Move(source);
  std::cout << "move:" << (&returned == &destination) << ':' << (destination.Data() == buffer)
            << ':' << destination.IsDeletable() << ':' << destination.LowerRow()
            << ':' << (source.Data() == nullptr) << ':' << source.LowerRow() << ':' << source.LowerCol()
            << ':' << source.UpperRow() << ':' << source.UpperCol() << '\n';
}

int main()
{
  rectangle_pointer_ranges();
  rectangle_assignment();
  rectangle_value_copy();
  rectangle_owned_returning();
  rectangle_borrowed_move();
}
