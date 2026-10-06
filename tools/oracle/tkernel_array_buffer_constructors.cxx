#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <iostream>

struct BufferItem
{
  inline static int defaults = 0;
  inline static int copies = 0;
  inline static int drops = 0;
  int value;
  BufferItem() : value(37) { ++defaults; }
  explicit BufferItem(int item) : value(item) {}
  BufferItem(const BufferItem& other) : value(other.value) { ++copies; }
  ~BufferItem() { ++drops; }
};

static void resetCounters()
{
  BufferItem::defaults = 0;
  BufferItem::copies = 0;
  BufferItem::drops = 0;
}

static void counters()
{
  std::cout << ";" << BufferItem::defaults << "," << BufferItem::copies << "," << BufferItem::drops;
}

static void bounded1(bool reuse)
{
  BufferItem buffer[] = {BufferItem(1), BufferItem(2), BufferItem(3), BufferItem(4)};
  resetCounters();
  {
    NCollection_Array1<BufferItem> array(buffer[0], -2, 0, reuse);
    std::cout << "array1_bounds_" << reuse << "=" << array.Lower() << "," << array.Upper()
              << ";" << array.Size() << ";" << array.IsDeletable() << ";" << (array.Data() == buffer)
              << ";" << array.First().value << "," << array.Last().value;
    array.ChangeValue(-1).value = 71;
    std::cout << ";" << buffer[0].value << "," << buffer[1].value << "," << buffer[2].value << "," << buffer[3].value;
    counters();
    std::cout << "\n";
  }
  std::cout << "array1_bounds_drop_" << reuse << "=" << BufferItem::drops << "\n";
}

static void sized1(bool reuse)
{
  BufferItem buffer[] = {BufferItem(1), BufferItem(2), BufferItem(3), BufferItem(4)};
  resetCounters();
  {
    NCollection_Array1<BufferItem> array(buffer[0], size_t(3), reuse);
    std::cout << "array1_size_" << reuse << "=" << array.Lower() << "," << array.Upper()
              << ";" << array.Size() << ";" << array.IsDeletable() << ";" << (array.Data() == buffer)
              << ";" << array.First().value << "," << array.Last().value;
    counters();
    std::cout << "\n";
  }
  std::cout << "array1_size_drop_" << reuse << "=" << BufferItem::drops << "\n";
}

static void ignoredShort1()
{
  BufferItem item(91), other(91);
  resetCounters();
  NCollection_Array1<BufferItem> bounded(item, -1, 2, false);
  NCollection_Array1<BufferItem> sized(other, size_t(5), false);
  std::cout << "array1_ignored_short=" << bounded.Size() << "," << sized.Size() << ";" << item.value << "," << other.value;
  counters();
  std::cout << "\n";
}

static void empty1(bool reuse)
{
  BufferItem item(91);
  resetCounters();
  NCollection_Array1<BufferItem> array(item, 7, 6, reuse);
  std::cout << "array1_empty_" << reuse << "=" << array.Lower() << "," << array.Upper()
            << ";" << array.Size() << ";" << array.IsDeletable() << ";" << (array.Data() == &item);
  counters();
  std::cout << "\n";
}

static void bounded2()
{
  BufferItem buffer[] = {BufferItem(1), BufferItem(2), BufferItem(3), BufferItem(4), BufferItem(5)};
  resetCounters();
  {
    NCollection_Array2<BufferItem> array(buffer[0], -1, 0, 3, 4);
    std::cout << "array2_bounds=" << array.LowerRow() << "," << array.UpperRow() << ","
              << array.LowerCol() << "," << array.UpperCol() << ";" << array.NbRows() << "," << array.NbColumns()
              << ";" << array.Lower() << "," << array.Upper() << ";" << array.IsDeletable() << ";" << (array.Data() == buffer);
    array.ChangeValue(-1, 4).value = 71;
    std::cout << ";" << buffer[0].value << "," << buffer[1].value << "," << buffer[2].value << "," << buffer[3].value << "," << buffer[4].value;
    counters();
    std::cout << "\n";
  }
  std::cout << "array2_bounds_drop=" << BufferItem::drops << "\n";
}

static void empty2(bool emptyRows)
{
  BufferItem item(91);
  resetCounters();
  NCollection_Array2<BufferItem> array(item, -1, emptyRows ? -2 : 0, 3, emptyRows ? 4 : 2);
  std::cout << "array2_empty_" << emptyRows << "=" << array.NbRows() << "," << array.NbColumns()
            << ";" << array.Lower() << "," << array.Upper() << ";" << array.IsDeletable() << ";" << (array.Data() == &item);
  counters();
  std::cout << "\n";
}

struct NoDefaultItem
{
  int value;
  explicit NoDefaultItem(int item) : value(item) {}
  NoDefaultItem() = delete;
  NoDefaultItem(const NoDefaultItem&) = delete;
};

static void pointerPrefix()
{
  NoDefaultItem buffer[] = {NoDefaultItem(1), NoDefaultItem(2), NoDefaultItem(3)};
  NCollection_Array1<NoDefaultItem> array(buffer, size_t(2));
  array.ChangeAt(1).value = 71;
  std::cout << "array1_pointer_prefix=" << array.Size() << ";" << array.IsDeletable()
            << ";" << buffer[0].value << "," << buffer[1].value << "," << buffer[2].value << "\n";
}

int main()
{
  bounded1(true);
  bounded1(false);
  sized1(true);
  sized1(false);
  ignoredShort1();
  empty1(true);
  empty1(false);
  bounded2();
  empty2(true);
  empty2(false);
  pointerPrefix();
}
