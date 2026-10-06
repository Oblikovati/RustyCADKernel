#include <NCollection_LocalArray.hxx>
#include <algorithm>
#include <iostream>
#include <string>
#include <utility>

using LocalMoveArray = NCollection_LocalArray<int, 4>;

void move_range(const char* name, size_t destination_size, size_t source_size)
{
  LocalMoveArray destination(destination_size), source(source_size);
  std::fill(destination.begin(), destination.end(), 11);
  std::fill(source.begin(), source.end(), 22);
  const int* old_destination = destination;
  const int* old_source = source;
  auto& returned = (destination = std::move(source));
  std::cout << name << ':' << (&returned == &destination) << ':' << destination.Size()
            << ':' << source.Size() << ':' << (destination.begin() == old_source)
            << ':' << (source.begin() == old_destination) << ':' << (source.begin() == old_source)
            << ':' << (source.begin() == source.end()) << ':' << destination.begin()[0] << '\n';
  if (destination_size > 4 && source_size > 4)
    std::cout << "retained:" << source.begin()[0] << '\n';
}

void self_move_range()
{
  LocalMoveArray array(6);
  std::fill(array.begin(), array.end(), 11);
  const int* beginning = array;
  auto& returned = (array = std::move(array));
  std::cout << "self:" << (&returned == &array) << ':' << (array.begin() == beginning)
            << ':' << array.Size() << ':' << array.begin()[0] << '\n';
}

void pointer_range()
{
  LocalMoveArray array(3);
  const LocalMoveArray& constant = array;
  const int* beginning = constant;
  int* mutable_beginning = array;
  std::cout << "pointers:" << (mutable_beginning == beginning)
            << ':' << (array.end() == beginning + 3) << ':' << (constant.end() == beginning + 3)
            << ':' << (constant.cbegin() == beginning) << ':' << (constant.cend() == beginning + 3);
  array.Deallocate();
  std::cout << ':' << (array.begin() != nullptr) << ':' << (array.begin() == array.end()) << '\n';
}

void owned_move_range(const char* name, size_t destination_size, size_t source_size)
{
  NCollection_LocalArray<std::string, 4> destination(destination_size), source(source_size);
  std::fill(source.begin(), source.end(), "source");
  auto& returned = (destination = std::move(source));
  const bool contents = std::all_of(destination.begin(), destination.end(),
                                   [](const std::string& value) { return value == "source"; });
  std::cout << name << ':' << (&returned == &destination) << ':' << destination.Size()
            << ':' << source.Size() << ':' << contents << '\n';
}

int main()
{
  move_range("heap-heap", 6, 7);
  move_range("inline-heap", 2, 7);
  move_range("heap-inline", 6, 3);
  move_range("inline-inline", 2, 3);
  self_move_range();
  pointer_range();
  owned_move_range("owned-inline-inline", 2, 3);
  owned_move_range("owned-heap-inline", 6, 3);
  owned_move_range("owned-inline-heap", 2, 7);
  owned_move_range("owned-heap-heap", 6, 7);
}
