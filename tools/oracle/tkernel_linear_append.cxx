#include <NCollection_LinearVector.hxx>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>

static std::string events;
static bool active = false, failValue = false;
static char construction = 'C';

void constructed(char phase, int value)
{
  if (!active) return;
  events += std::string(1, phase) + std::to_string(value) + ",";
  if (failValue && phase == construction && (value == 99 || value == 0 || value == 42))
    throw std::runtime_error("append constructor value " + std::to_string(value) + ": expected success");
}

template<bool NoexceptMove> struct AppendItem
{
  int value;
  std::unique_ptr<int> payload;
  explicit AppendItem(int initial) : value(initial), payload(new int(initial)) {}
  AppendItem() : value(0), payload(new int(0)) { constructed('A', value); }
  AppendItem(int initial, const std::string& suffix)
    : value(initial + static_cast<int>(suffix.size())), payload(new int(value))
  { constructed('E', value); }
  AppendItem(const AppendItem& source)
    : value(source.value), payload(new int(*source.payload)) { constructed('C', value); }
  AppendItem(AppendItem&& source) noexcept(NoexceptMove)
    : value(source.value), payload(std::move(source.payload))
  { source.value = -1; constructed('M', value); }
  ~AppendItem() { if (active) events += "D" + std::to_string(value) + ","; }
};

struct AppendCase { const char* name; size_t capacity, count; char mode; bool moving, fail; };

template<class Item> Item& append_value(NCollection_LinearVector<Item>& vector, Item& donor, char mode)
{
  if (mode == 'C') return vector.Append(donor);
  if (mode == 'M') return vector.Append(std::move(donor));
  if (mode == 'A') return vector.Appended();
  return vector.EmplaceAppend(40, std::string("ab"));
}

template<class Item> Item* attempt(NCollection_LinearVector<Item>& vector, Item& donor, char mode)
{
  try { return &append_value(vector, donor, mode); }
  catch (const std::runtime_error&) { return nullptr; }
}

template<class Item> void emit(const AppendCase& test, const NCollection_LinearVector<Item>& vector,
                               const Item& donor, const Item* before, const int* payload, Item* result)
{
  std::cout << test.name << ':' << (result ? "ok" : "throw") << ':' << vector.Size() << ':'
            << vector.Capacity() << ':' << (before == vector.Data()) << ':' << events << ':';
  for (const Item& item : vector) std::cout << item.value << '/' << bool(item.payload) << ',';
  std::cout << ':' << donor.value << '/' << bool(donor.payload) << ':'
            << (result && result == vector.Data() + vector.Size() - 1) << ':'
            << (result && result->payload.get() == payload) << '\n';
}

template<bool Moving> void run_case(const AppendCase& test)
{
  using Item = AppendItem<Moving>;
  NCollection_LinearVector<Item> vector(test.capacity);
  for (size_t index = 0; index < test.count; ++index) vector.EmplaceAppend(10 + index);
  Item donor(99);
  auto* before = vector.Data();
  auto* payload = donor.payload.get();
  construction = test.mode;
  failValue = test.fail;
  events.clear();
  active = true;
  Item* result = attempt(vector, donor, test.mode);
  emit(test, vector, donor, before, payload, result);
  active = false;
}

int main()
{
  const AppendCase cases[] = {
    {"copy-kept",4,2,'C',false,false}, {"copy-grow",2,2,'C',false,false},
    {"copy-empty",0,0,'C',false,false}, {"copy-movegrow",2,2,'C',true,false},
    {"move-kept",4,2,'M',false,false}, {"move-grow",2,2,'M',false,false},
    {"move-empty",0,0,'M',false,false}, {"move-movegrow",2,2,'M',true,false},
    {"default-kept",4,2,'A',false,false}, {"default-grow",2,2,'A',false,false},
    {"default-empty",0,0,'A',false,false}, {"default-movegrow",2,2,'A',true,false},
    {"emplace-kept",4,2,'E',false,false}, {"emplace-grow",2,2,'E',false,false},
    {"emplace-empty",0,0,'E',false,false}, {"emplace-movegrow",2,2,'E',true,false},
    {"copy-fail-kept",4,2,'C',false,true}, {"copy-fail-grow",2,2,'C',true,true},
    {"move-fail-kept",4,2,'M',false,true}, {"move-fail-grow",2,2,'M',false,true},
    {"default-fail-empty",0,0,'A',false,true}, {"default-fail-grow",2,2,'A',true,true},
    {"emplace-fail-kept",4,2,'E',false,true}, {"emplace-fail-grow",2,2,'E',false,true},
  };
  for (const auto& test : cases) test.moving ? run_case<true>(test) : run_case<false>(test);
}
