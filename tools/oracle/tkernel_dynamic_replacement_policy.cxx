#include "tkernel_dynamic_replacement_items.hxx"
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
