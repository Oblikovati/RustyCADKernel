#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <new>
static std::string trace;
static int liveItems=0;
static void event(char kind,int value) {
  if(!trace.empty()) trace+=',';
  trace+=kind; trace+=std::to_string(value);
}
static void construct(int value) {
  event('C',value);
  if(value==99) throw std::runtime_error("construct99");
  ++liveItems;
}
struct Assignable {
  int value;
  explicit Assignable(int n=0):value(n) { construct(n); }
  ~Assignable() { event('D',value); --liveItems; }
  Assignable& operator=(Assignable&& other) {
    event('A',other.value); value=other.value; other.value=-7;
    if(value==77) throw std::runtime_error("assign77");
    return *this;
  }
};
struct MoveOnly {
  int value;
  explicit MoveOnly(int n):value(n) { construct(n); }
  MoveOnly(MoveOnly&& other) noexcept:value(other.value) { event('M',value);other.value=-7;++liveItems; }
  MoveOnly(const MoveOnly&)=delete;
  MoveOnly& operator=(MoveOnly&&)=delete;
  ~MoveOnly() { event('D',value); --liveItems; }
};
struct CopyPreferred {
  int value;
  explicit CopyPreferred(int n):value(n) { construct(n); }
  CopyPreferred(CopyPreferred&& other) noexcept(false):value(other.value) { event('M',value);other.value=-7;++liveItems; }
  CopyPreferred(const CopyPreferred& other):value(other.value) {
    event('K',value);if(value==88) throw std::runtime_error("copy88");
    ++liveItems;
  }
  CopyPreferred& operator=(CopyPreferred&&)=delete;
  ~CopyPreferred() { event('D',value); --liveItems; }
};
struct DirectOnly {
  int value;
  explicit DirectOnly(int n):value(n) { construct(n); }
  DirectOnly(DirectOnly&&)=delete;
  DirectOnly(const DirectOnly&)=delete;
  DirectOnly& operator=(DirectOnly&&)=delete;
  ~DirectOnly() { event('D',value); --liveItems; }
};
template<class T> struct Engine : NCollection_DynamicArray<T> {
  using NCollection_DynamicArray<T>::replaceAt;
};
template<class T> static void scenario(const char* name,int request,bool expectedLive) {
  alignas(T) unsigned char storage[sizeof(T)];
  T* slot=::new(static_cast<void*>(storage)) T(1);trace.clear();
  bool threw=false;
  try { Engine<T>::replaceAt(slot,request); } catch(const std::runtime_error&) { threw=true; }
  if(liveItems!=int(expectedLive)) throw std::logic_error("live items "+std::to_string(liveItems)+": expected "+std::to_string(expectedLive));
  std::cout<<name<<' '<<threw<<' '<<liveItems<<' ';
  if(expectedLive) std::cout<<std::launder(slot)->value;else std::cout<<"dead";
  std::cout<<' '<<trace<<'\n';trace.clear();
  if(!expectedLive) slot=::new(static_cast<void*>(storage)) T(5);
  std::launder(slot)->~T();
}
static void primitive() {
  int slot=1; Engine<int>::replaceAt(&slot,7);
  std::cout<<"trivial_assign 0 1 "<<slot<<" none\n";
}
static void default_assign() {
  Assignable slot(1);trace.clear();Engine<Assignable>::replaceAt(&slot);
  std::cout<<"default_assign 0 1 "<<slot.value<<' '<<trace<<'\n';
}
int main() {
  scenario<Assignable>("assign_success",2,true);
  scenario<Assignable>("assign_constructor_failure",99,true);
  scenario<Assignable>("assignment_failure",77,true);
  scenario<MoveOnly>("move_success",2,true);
  scenario<MoveOnly>("move_constructor_failure",99,true);
  scenario<CopyPreferred>("copy_success",2,true);
  scenario<CopyPreferred>("copy_constructor_failure",99,true);
  scenario<CopyPreferred>("copy_transfer_failure",88,false);
  scenario<DirectOnly>("direct_success",2,true);
  scenario<DirectOnly>("direct_constructor_failure",99,false);
  primitive();default_assign();
}
