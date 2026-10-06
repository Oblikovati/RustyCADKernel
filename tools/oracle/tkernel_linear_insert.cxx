#include <NCollection_LinearVector.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <memory>
#include <set>
#include <stdexcept>
#include <string>

struct InsertItem;
static std::set<const InsertItem*> live;
static InsertItem* donorAddress = nullptr;
static bool active = false, failInsert = false;
static int failShift = -99;
static std::string events;

struct InsertItem
{
  int value;
  std::unique_ptr<int> resource;
  explicit InsertItem(int initial) : value(initial), resource(new int(initial)) { live.insert(this); }
  InsertItem(const InsertItem& source);
  InsertItem(InsertItem&& source);
  ~InsertItem();
};

InsertItem::InsertItem(const InsertItem& source)
  : value(source.value), resource(new int(*source.resource))
{
  const bool inserting = &source == donorAddress;
  if (active) events += std::string(inserting ? "I" : "G") + std::to_string(value) + ",";
  if (active && inserting && failInsert) throw std::runtime_error("insert copy failure");
  live.insert(this);
}

InsertItem::InsertItem(InsertItem&& source)
  : value(source.value), resource(std::move(source.resource))
{
  const bool inserting = &source == donorAddress;
  if (active) events += std::string(inserting ? "V" : "M") + std::to_string(value) + ",";
  source.value = -1;
  if (active && ((inserting && failInsert) || (!inserting && value == failShift)))
    throw std::runtime_error("move construction failure");
  live.insert(this);
}

InsertItem::~InsertItem()
{
  if (active) events += "D" + std::to_string(value) + ",";
  live.erase(this);
}

using InsertVector = NCollection_LinearVector<InsertItem>;

std::string perform(InsertVector& vector, size_t index, bool after, bool move, InsertItem& donor)
{
  try
  {
    if (after && move) vector.InsertAfter(index, std::move(donor));
    else if (after) vector.InsertAfter(index, donor);
    else if (move) vector.InsertBefore(index, std::move(donor));
    else vector.InsertBefore(index, donor);
    return "ok";
  }
  catch (const Standard_OutOfRange&) { return "range"; }
  catch (const std::runtime_error&) { return "throw"; }
}

void print_slots(const InsertVector& vector, size_t oldSize)
{
  for (size_t index = 0; index <= oldSize; ++index)
  {
    const auto* slot = vector.Data() == nullptr ? nullptr : vector.Data() + index;
    if (live.count(slot)) std::cout << slot->value << '/' << (slot->resource != nullptr) << ',';
    else std::cout << "X,";
  }
}

void repair(InsertVector& vector, size_t oldSize)
{
  for (size_t index = 0; index < vector.Size(); ++index)
    if (!live.count(vector.Data() + index)) new (vector.Data() + index) InsertItem(90 + index);
  if (vector.Size() == oldSize && vector.Data() != nullptr && live.count(vector.Data() + oldSize))
    vector.Data()[oldSize].~InsertItem();
}

struct InsertCase
{
  const char* name;
  size_t capacity, count, index;
  bool after, move;
  int fail;
  bool failValue;
};

void emit(const InsertCase& test, const InsertVector& vector, const InsertItem& donor,
          const InsertItem* before, const int* payload, const std::string& outcome)
{
  const size_t insertion = test.index + (test.after ? 1 : 0);
  const bool payloadSame = outcome == "ok" && vector.Data()[insertion].resource.get() == payload;
  std::cout << test.name << ':' << outcome << ':' << vector.Size() << ':' << vector.Capacity()
            << ':' << (before == vector.Data()) << ':' << events << ':';
  print_slots(vector, test.count);
  std::cout << ':' << donor.value << '/' << (donor.resource != nullptr) << ':' << payloadSame << '\n';
}

void run_case(const InsertCase& test)
{
  InsertVector vector(test.capacity);
  for (size_t i = 0; i < test.count; ++i) vector.EmplaceAppend(10 + static_cast<int>(i));
  InsertItem donor(99);
  donorAddress = &donor;
  auto* before = vector.Data();
  const int* payload = donor.resource.get();
  failShift = test.fail;
  failInsert = test.failValue;
  events.clear();
  active = true;
  const std::string outcome = perform(vector, test.index, test.after, test.move, donor);
  active = false;
  emit(test, vector, donor, before, payload, outcome);
  repair(vector, test.count);
}

int main()
{
  const InsertCase cases[] = {
    {"head-copy",8,3,0,false,false,-99,false}, {"middle-copy",8,3,1,false,false,-99,false},
    {"tail-copy",8,3,3,false,false,-99,false}, {"after-head",8,3,0,true,false,-99,false},
    {"after-tail",8,3,2,true,true,-99,false}, {"middle-move",8,3,1,false,true,-99,false},
    {"grow-copy",3,3,1,false,false,-99,false}, {"grow-move",3,3,1,false,true,-99,false},
    {"empty-copy",0,0,0,false,false,-99,false}, {"bad-before",8,3,4,false,false,-99,false},
    {"bad-after",8,3,3,true,false,-99,false}, {"empty-after",0,0,0,true,false,-99,false},
    {"shift-first",8,3,0,false,false,12,false}, {"shift-second",8,3,0,false,false,11,false},
    {"shift-last",8,3,0,false,false,10,false}, {"insert-copy-fail",8,3,1,false,false,-99,true},
    {"insert-move-fail",8,3,1,false,true,-99,true}, {"tail-fail",8,3,3,false,false,-99,true},
    {"empty-fail",0,0,0,false,false,-99,true}, {"grow-insert-fail",3,3,1,false,false,-99,true},
  };
  for (const auto& test : cases) run_case(test);
  return live.empty() ? 0 : 1;
}
