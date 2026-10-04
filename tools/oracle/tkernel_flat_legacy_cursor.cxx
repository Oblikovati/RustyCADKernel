// Native non-consuming cursor behavior at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <NCollection_FlatMap.hxx>
#include <NCollection_FlatDataMap.hxx>
#include <Standard_Failure.hxx>
#include <iostream>

template <typename Accessor>
void printFailure(const Accessor& access)
{
  try { access(); }
  catch (const Standard_Failure& error) {
    std::cout << error.ExceptionType() << ':' << error.what() << '\n';
  }
}

void printSetCursor()
{
  NCollection_FlatMap<int> map;
  for (const int key : {3, 11, 4}) map.Add(key);
  NCollection_FlatMap<int>::Iterator cursor(map), empty;
  const auto copy = cursor;
  std::cout << cursor.More() << ' ' << cursor.IsEqual(copy) << '\n';
  while (cursor.More()) {
    std::cout << cursor.Key() << ' ' << cursor.Key() << ' ' << cursor.Value() << '\n';
    cursor.Next();
  }
  cursor.Next();
  std::cout << cursor.More() << ' ' << cursor.IsEqual(empty) << '\n';
  printFailure([&] { cursor.Key(); });
  printFailure([&] { cursor.Value(); });
}

void printDataCursor()
{
  NCollection_FlatDataMap<int, int> map;
  for (const int key : {3, 11, 4}) map.Bind(key, key * 10);
  NCollection_FlatDataMap<int, int>::Iterator cursor(map), empty;
  const auto copy = cursor;
  std::cout << cursor.More() << ' ' << cursor.IsEqual(copy) << '\n';
  while (cursor.More()) {
    std::cout << cursor.Key() << ' ' << cursor.Value() << ' ' << cursor.Value() << '\n';
    cursor.Next();
  }
  cursor.Next();
  std::cout << cursor.More() << ' ' << cursor.IsEqual(empty) << '\n';
  printFailure([&] { cursor.Key(); });
  printFailure([&] { cursor.Value(); });
}

int main()
{
  printSetCursor();
  printDataCursor();
}
