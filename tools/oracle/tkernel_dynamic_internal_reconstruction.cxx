// OCCT 3d097a0: SetValue replacement for deleted assignment and internal source aliases.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <new>
#include <stdexcept>
#include <string>
#include <utility>
static std::string trace, failure;
static int constructions, live;
static void event(char kind, int value) { trace += kind + std::to_string(value) + ";"; }
static void attempt(char kind, int value) {
  event(kind, value);
  ++constructions;
  if ((failure == "construct" && constructions == 1) || (failure == "transfer" && constructions == 2))
    throw std::runtime_error("selected constructor");
  ++live;
}
template<bool NoThrow> struct CopyPreferred {
  int value;
  explicit CopyPreferred(int number):value(number) { ++live; }
  CopyPreferred():value(0) {
    event('D',0); if (failure == "default") throw std::runtime_error("default"); ++live;
  }
  CopyPreferred(const CopyPreferred& source):value(source.value) { attempt('K', value); }
  CopyPreferred(CopyPreferred&& source) noexcept(NoThrow):value(source.value) {
    source.value=-7; attempt('M',value);
  }
  CopyPreferred& operator=(const CopyPreferred&)=delete;
  CopyPreferred& operator=(CopyPreferred&&)=delete;
  ~CopyPreferred() { event('X',value); --live; }
};
struct MoveOnly {
  int value;
  explicit MoveOnly(int number):value(number) { ++live; }
  MoveOnly():value(0) {
    event('D',0); if (failure == "default") throw std::runtime_error("default"); ++live;
  }
  MoveOnly(const MoveOnly&)=delete;
  MoveOnly(MoveOnly&& source) noexcept(false):value(source.value) {
    source.value=-7; attempt('M',value);
  }
  MoveOnly& operator=(MoveOnly&&)=delete;
  ~MoveOnly() { event('X',value); --live; }
};
template<int Mode, class T, class Index>
static T& set(NCollection_DynamicArray<T>& array, Index index, size_t source) {
  if constexpr (Mode == 0 || Mode == 3) return array.SetValue(index, std::as_const(array[source]));
  else return array.SetValue(index, std::move(array[source]));
}
template<int Mode> static void run(const std::string& name, int index, size_t source, const char* fail) {
  using T=typename std::conditional<Mode==2, MoveOnly, CopyPreferred<Mode==3>>::type;
  NCollection_DynamicArray<T> array(2);
  for(int i=0;i<4;++i) array.EmplaceAppend((i+1)*10);
  const T* original=&array[source]; trace.clear();failure=fail;constructions=0;
  bool threw=false, same=false;
  try { T& result=index>=0 && index%2 ? set<Mode>(array,size_t(index),source) : set<Mode>(array,index,source);
        same=&result==&array[size_t(index)]; }
  catch(...) { threw=true; }
  const bool dead=failure=="transfer";
  std::cout<<name<<' '<<(trace.empty()?"none":trace)<<" size="<<array.Size()<<" live="<<live
           <<" threw="<<threw<<" same="<<same<<" stable="<<(original==&array[source]);
  for(size_t i=0;i<array.Size();++i) {
    if(dead && i==size_t(index)) std::cout<<" dead";
    else std::cout<<' '<<array[i].value;
  }
  std::cout<<'\n';
  failure.clear();
  if(dead) ::new(static_cast<void*>(&array[size_t(index)])) T(55);
  array.Clear(true);
  if(live!=0) throw std::logic_error("cleanup live count: expected zero");
}
template<int Mode> static void cases() {
  for(int index : {0,1,2,3,7}) for(size_t source=0;source<4;++source)
    run<Mode>("reconstruct_m"+std::to_string(Mode)+"_i"+std::to_string(index)+"_s"+std::to_string(source),index,source,"");
  for(int index : {0,1}) for(const char* fail : {"construct","transfer"})
    if (Mode != 3 || std::string(fail) != "transfer") run<Mode>("reconstruct_m"+std::to_string(Mode)+"_i"+std::to_string(index)+"_"+fail,index,1,fail);
  run<Mode>("reconstruct_m"+std::to_string(Mode)+"_default",7,1,"default");
  run<Mode>("reconstruct_m"+std::to_string(Mode)+"_negative",-1,1,"");
}
int main(){cases<0>();cases<1>();cases<2>();cases<3>();}
