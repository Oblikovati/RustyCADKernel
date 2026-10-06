#include "tkernel_dynamic_replacement_items.hxx"
template<class T> static void cleanup(NCollection_DynamicArray<T>& array, bool live) {
  if (!live) ::new(static_cast<void*>(&array[0])) T(5);
  array.Clear(true);
  if (liveItems != 0) throw std::logic_error("live items: expected 0 after repair and clear");
}
template<class T, class Index> static void run(const char* name, int request, bool live, Index index) {
  NCollection_DynamicArray<T> array(2);
  array.EmplaceAppend(1); trace.clear();
  bool threw = false;
  bool same = false;
  try { same = &array.EmplaceValue(index, request) == &array[0]; }
  catch (...) { threw = true; }
  std::cout << name << " threw=" << threw << " size=" << array.Size();
  std::cout << " live=" << liveItems << " same=" << same << " value=";
  if (live) std::cout << array[0].value; else std::cout << "dead";
  std::cout << " trace=" << (trace.empty() ? "none" : trace) << '\n';
  if (liveItems != int(live)) throw std::logic_error("live count: expected selected branch lifetime");
  cleanup(array,live); trace.clear();
}
int main() {
  run<Assignable>("assign",2,true,0);
  run<Assignable>("assign_ctor_fail",99,true,size_t(0));
  run<Assignable>("assign_fail",77,true,0);
  run<MoveOnly>("move_success",2,true,0);
  run<MoveOnly>("move_ctor_fail",99,true,size_t(0));
  run<CopyPreferred>("copy",2,true,size_t(0));
  run<CopyPreferred>("copy_ctor_fail",99,true,0);
  run<CopyPreferred>("copy_transfer_fail",88,false,size_t(0));
  run<DirectOnly>("direct",2,true,0);
  run<DirectOnly>("direct_ctor_fail",99,false,size_t(0));
  run<DirectOnly>("direct_negative",2,true,-1);
  run<CopyPreferred>("copy_negative",2,true,-1);
  run<Assignable>("assign_negative",2,true,-1);
}
