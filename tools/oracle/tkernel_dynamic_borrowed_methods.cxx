// -fno-access-control asserts private native descriptor capacity without changing OCCT headers.
// Unmodified OCCT 3d097a0: distinguish copy construction and both assignment operators.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <utility>
struct ProbeAllocator : NCollection_BaseAllocator {
  std::string events;
  void* AllocateOptimal(size_t size) override {
    events += "A;";
    return NCollection_BaseAllocator::AllocateOptimal(size);
  }
};
static ProbeAllocator* probe;
static std::string failure;
static int moveCount;
static void event(const std::string& text) { probe->events += text + ";"; }
struct Item {
  int value;
  explicit Item(int number) : value(number) {}
  Item() : value(0) {
    event("D");
    if (failure == "default") throw std::runtime_error("default 0");
  }
  Item(const Item& source) : value(source.value) {
    event("C" + std::to_string(value));
    if (failure == "construct") throw std::runtime_error("copy " + std::to_string(value));
  }
  Item(Item&& source) : value(source.value) {
    event("K" + std::to_string(value));
    source.value = -7;
    if (failure == "move_construct") throw std::runtime_error("move construction " + std::to_string(value));
  }
  ~Item() { event("X" + std::to_string(value)); }
  Item& operator=(Item&& source) {
    if (this == &source) event("S");
    event("M" + std::to_string(value) + "," + std::to_string(source.value));
    value = source.value;
    source.value = -7;
    if ((++moveCount == 1 && failure == "move") || (value == 99 && failure == "final_move"))
      throw std::runtime_error("move assignment");
    if (this == &source && failure == "self") throw std::runtime_error("self move assignment");
    return *this;
  }
  Item& operator=(const Item& source) {
    if (this == &source) event("Q");
    event("P" + std::to_string(value) + "," + std::to_string(source.value));
    if (failure == "assign") throw std::runtime_error("copy assignment");
    value = source.value;
    return *this;
  }
};
template<bool Moving> static decltype(auto) forwarded(Item& source) {
  if constexpr (Moving) return std::move(source);
  else return std::as_const(source);
}
template<bool Moving, class Index> static Item& operate(NCollection_DynamicArray<Item>& array,
                                            Item& source, int operation, Index index) {
  switch (operation) {
    case 0: return array.Append(forwarded<Moving>(source));
    case 1: return array.SetValue(index, forwarded<Moving>(source));
    case 2: return array.InsertBefore(index, forwarded<Moving>(source));
    default: return array.InsertAfter(index, forwarded<Moving>(source));
  }
}
static void print_row(const char* name, const NCollection_DynamicArray<Item>& array,
                      const Item& source, bool returned) {
  std::cout << name << ' ' << probe->events << ' ' << array.Size() << ' ' << returned;
  for (size_t i = 0; i < array.Size(); ++i) std::cout << ' ' << array[i].value;
  std::cout << " source=" << source.value << '\n';
}
template<bool Moving = false> static void run(const char* name, int operation, int index, const char* fail, bool wide) {
  occ::handle<ProbeAllocator> allocator = new ProbeAllocator;
  probe = allocator.get(); failure.clear(); moveCount = 0;
  NCollection_DynamicArray<Item> array(2, allocator);
  array.EmplaceAppend(10); array.EmplaceAppend(20);
  Item source(99);
  probe->events.clear(); failure = fail;
  bool returned = false;
  try {
    Item& result = wide ? operate<Moving>(array, source, operation, size_t(index))
                        : operate<Moving>(array, source, operation, index);
    returned = &result == &array[operation == 0 ? 2 : index + (operation == 3)];
  } catch (...) { event("E"); }
  print_row(name, array, source, returned);
}
static void borrowed_cases() {
  run("append",0,0,"",false); run("append_copy_throw",0,0,"construct",true);
  run("set_existing",1,0,"",false); run("set_existing_wide",1,1,"",true);
  run("set_copy_throw",1,0,"construct",false); run("set_move_throw",1,0,"move",true);
  run("set_gap",1,4,"",false); run("set_gap_wide",1,4,"",true);
  run("set_gap_copy_throw",1,4,"construct",false); run("set_gap_default_throw",1,4,"default",true);
  run("set_negative",1,-1,"",false);
  run("before",2,0,"",false); run("before_wide",2,1,"",true);
  run("after",3,0,"",false); run("after_wide",3,1,"",true);
  run("before_copy_throw",2,0,"assign",false); run("after_move_throw",3,0,"move",true);
  run("before_default_throw",2,0,"default",false);
  run("before_negative",2,-1,"",false); run("after_end",3,2,"",true);

}
static void moved_cases() {
  run<true>("moved_append",0,0,"",false);
  run<true>("moved_append_ctor_fail",0,0,"move_construct",true);
  run<true>("moved_set_existing",1,0,"",false);
  run<true>("moved_set_existing_wide",1,1,"",true);
  run<true>("moved_set_ctor_fail",1,0,"move_construct",false);
  run<true>("moved_set_assign_fail",1,0,"move",true);
  run<true>("moved_set_gap",1,4,"",false);
  run<true>("moved_set_gap_wide",1,4,"",true);
  run<true>("moved_set_gap_ctor_fail",1,4,"move_construct",false);
  run<true>("moved_set_default_fail",1,4,"default",true);
  run<true>("moved_set_negative",1,-1,"",false);
  run<true>("moved_before",2,0,"",false);
  run<true>("moved_before_wide",2,1,"",true);
  run<true>("moved_after",3,0,"",false);
  run<true>("moved_after_wide",3,1,"",true);
  run<true>("moved_before_final_fail",2,0,"final_move",false);
  run<true>("moved_after_shift_fail",3,0,"move",true);
  run<true>("moved_before_default_fail",2,0,"default",false);
  run<true>("moved_before_negative",2,-1,"",false);
  run<true>("moved_after_end",3,2,"",true);
}


template<bool Moving> static bool append_internal(NCollection_DynamicArray<Item>& array, size_t source) {
  try {
    Item& result = array.Append(forwarded<Moving>(array[source]));
    return &result == &array[array.Size() - 1];
  } catch (...) { event("E"); return false; }
}
static void print_internal(const char* name, const NCollection_DynamicArray<Item>& array,
                           bool returned, bool stable) {
  std::cout << name << ' ' << probe->events << ' ' << array.Size() << ' ' << returned << ' ' << stable;
  for (size_t i = 0; i < array.Size(); ++i) std::cout << ' ' << array[i].value;
  std::cout << '\n';
}
template<bool Moving> static void internal(const char* name, int increment, int count,
                                            size_t source, const char* fail, bool retry = false) {
  occ::handle<ProbeAllocator> allocator = new ProbeAllocator;
  probe = allocator.get(); failure.clear(); moveCount = 0;
  NCollection_DynamicArray<Item> array(increment, allocator);
  for (int i = 0; i < count; ++i) array.EmplaceAppend((i + 1) * 10);
  if (count == 4 && array.myContainer.Capacity() != 2) throw std::logic_error("table capacity: expected 2 before append");
  const Item* original = &array[source]; probe->events.clear(); failure = fail;
  bool returned = append_internal<Moving>(array, source);
  if (retry) { failure.clear(); returned = append_internal<Moving>(array, source); }
  if (count == 4 && array.myContainer.Capacity() != 4) throw std::logic_error("table capacity: expected 4 after append");
  print_internal(name, array, returned, original == &array[source]);
}
static void internal_cases() {
  internal<false>("self_copy_same",4,1,0,"");
  internal<true>("self_move_same",4,1,0,"");
  internal<false>("self_copy_new_block",2,2,0,"");
  internal<true>("self_move_new_block",2,2,1,"");
  internal<false>("self_copy_table_growth",2,4,0,"");
  internal<true>("self_move_table_growth",2,4,3,"");
  internal<false>("self_copy_later_block",2,5,3,"");
  internal<true>("self_move_later_block",2,5,3,"");
  internal<false>("self_copy_same_throw",4,1,0,"construct");
  internal<true>("self_move_same_throw",4,1,0,"move_construct");
  internal<false>("self_copy_table_throw",2,4,0,"construct");
  internal<true>("self_move_table_throw",2,4,3,"move_construct");
  internal<false>("self_copy_retry",2,4,0,"construct",true);
  internal<true>("self_move_retry",2,4,3,"move_construct",true);
}

template<bool Moving, bool After, class Index>
static bool insert_internal(NCollection_DynamicArray<Item>& array, Index index, size_t source) {
  try {
    Item& result = After ? array.InsertAfter(index, forwarded<Moving>(array[source]))
                        : array.InsertBefore(index, forwarded<Moving>(array[source]));
    return &result == &array[size_t(index) + size_t(After)];
  } catch (...) { event("E"); return false; }
}
template<bool Moving, bool After>
static void insertion_case(const char* name, int index, size_t source, const char* fail, bool wide) {
  occ::handle<ProbeAllocator> allocator = new ProbeAllocator;
  probe = allocator.get(); failure.clear(); moveCount = 0;
  NCollection_DynamicArray<Item> array(2, allocator);
  for (int i = 0; i < 4; ++i) array.EmplaceAppend((i + 1) * 10);
  if (std::string(fail) == "final_move") array[source].value = 99;
  const Item* original = &array[source]; probe->events.clear(); failure = fail;
  const bool returned = wide ? insert_internal<Moving, After>(array, size_t(index), source)
                            : insert_internal<Moving, After>(array, index, source);
  print_internal(name, array, returned, original == &array[source]);
}
template<bool Moving, bool After> static void insertion_matrix() {
  for (int index = 0; index < 4; ++index) {
    for (size_t source = 0; source < 4; ++source) {
      const std::string name = "alias_a" + std::to_string(After) + "_m" + std::to_string(Moving)
                            + "_i" + std::to_string(index) + "_s" + std::to_string(source);
      insertion_case<Moving, After>(name.c_str(), index, source, "", index % 2 == 1);
    }
  }
}
static void insertion_failures() {
  insertion_case<false,false>("alias_copy_before_throw",0,1,"assign",false);
  insertion_case<false,false>("alias_copy_self_throw",0,0,"assign",true);
  insertion_case<false,true>("alias_copy_after_throw",1,0,"assign",true);
  insertion_case<true,false>("alias_move_before_shift_throw",0,3,"move",false);
  insertion_case<true,true>("alias_move_after_shift_throw",0,1,"move",true);
  insertion_case<true,false>("alias_move_before_self_throw",0,0,"self",false);
  insertion_case<true,true>("alias_move_after_self_throw",0,1,"self",true);
  insertion_case<true,false>("alias_move_final_throw",2,0,"final_move",true);
  insertion_case<false,false>("alias_default_before_throw",0,1,"default",false);
  insertion_case<true,true>("alias_default_after_throw",0,1,"default",true);
  insertion_case<false,false>("alias_before_negative",-1,0,"",false);
  insertion_case<true,true>("alias_after_end",4,0,"",true);
}
template<bool Moving, class Index>
static bool set_internal(NCollection_DynamicArray<Item>& array, Index index, size_t source) {
  try {
    Item& result = array.SetValue(index, forwarded<Moving>(array[source]));
    return &result == &array[size_t(index)];
  } catch (...) { event("E"); return false; }
}
template<bool Moving>
static void set_case(const char* name, int index, size_t source, const char* fail, bool wide) {
  occ::handle<ProbeAllocator> allocator = new ProbeAllocator;
  probe = allocator.get(); failure.clear(); moveCount = 0;
  NCollection_DynamicArray<Item> array(2, allocator);
  for (int i = 0; i < 4; ++i) array.EmplaceAppend((i + 1) * 10);
  const Item* original = &array[source]; probe->events.clear(); failure = fail;
  const bool returned = wide ? set_internal<Moving>(array, size_t(index), source)
                            : set_internal<Moving>(array, index, source);
  print_internal(name, array, returned, original == &array[source]);
}
template<bool Moving> static void set_matrix() {
  for (int index = 0; index < 8; ++index) {
    for (size_t source = 0; source < 4; ++source) {
      const std::string name = "alias_set_m" + std::to_string(Moving)
                            + "_i" + std::to_string(index) + "_s" + std::to_string(source);
      set_case<Moving>(name.c_str(), index, source, "", index % 2 == 1);
    }
  }
}
static void set_failures() {
  set_case<false>("set_alias_copy_ctor",0,1,"construct",false);
  set_case<false>("set_alias_self_copy_ctor",1,1,"construct",false);
  set_case<true>("set_alias_move_ctor",0,1,"move_construct",false);
  set_case<true>("set_alias_self_move_ctor",1,1,"move_construct",false);
  set_case<false>("set_alias_copy_assign",0,1,"move",false);
  set_case<false>("set_alias_self_copy_assign",1,1,"move",false);
  set_case<true>("set_alias_move_assign",0,1,"move",false);
  set_case<true>("set_alias_self_move_assign",1,1,"move",false);
  set_case<false>("set_alias_gap_copy_ctor",7,1,"construct",false);
  set_case<true>("set_alias_gap_move_ctor",7,1,"move_construct",false);
  set_case<false>("set_alias_gap_copy_default",7,1,"default",false);
  set_case<true>("set_alias_gap_move_default",7,1,"default",false);
  set_case<false>("set_alias_negative_copy",-1,1,"",false);
  set_case<true>("set_alias_negative_move",-1,1,"",false);
}
int main() {
  borrowed_cases(); moved_cases(); internal_cases();
  insertion_matrix<false,false>(); insertion_matrix<false,true>();
  insertion_matrix<true,false>(); insertion_matrix<true,true>();
  insertion_failures();
  set_matrix<false>(); set_matrix<true>(); set_failures();
}
