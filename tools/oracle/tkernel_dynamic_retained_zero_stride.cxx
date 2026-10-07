// OCCT 3d097a0: zero stride still addresses a retained first slot; Clear omits its destructor.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <limits>
#include <memory>
#include <stdexcept>
#include <utility>
struct RetainedPayload {
  int value;
  int* destroyed;
  RetainedPayload(int v,int& count):value(v),destroyed(&count) {
    if(v<0)throw std::runtime_error("negative probe construction");
  }
  ~RetainedPayload(){++*destroyed;}
};
template<class T> struct RetainedArray:NCollection_DynamicArray<T> {
  using NCollection_DynamicArray<T>::NCollection_DynamicArray;
  using NCollection_DynamicArray<T>::blockCount;
  using NCollection_DynamicArray<T>::blockAddress;
};
using Array=RetainedArray<RetainedPayload>;
static Array retainedArray(int& destroyed) {
  Array array(2);
  for(int value=1;value<=3;++value)array.EmplaceAppend(value,destroyed);
  array.Clear(false);destroyed=0;array.SetIncrement(std::numeric_limits<size_t>::max());
  return array;
}
static void retainedCase(bool release,bool erase) {
  int destroyed=0;auto array=retainedArray(destroyed);const auto original=array.blockAddress(0);
  auto* slot=&array.EmplaceAppend(11,destroyed);array.ChangeValue(size_t(0)).value=12;
  std::cout<<"retained value="<<array.Value(size_t(0)).value<<" first="<<array.First().value
           <<" last="<<array.Last().value<<" size="<<array.Size()<<" same="<<(slot==original)<<'\n';
  if(erase)array.EraseLast();else array.Clear(release);
  std::cout<<"after release="<<release<<" erase="<<erase<<" size="<<array.Size()
           <<" destroyed="<<destroyed<<" blocks="<<array.blockCount()<<'\n';
  if(!erase&&!release){slot->~RetainedPayload();std::cout<<"repair destroyed="<<destroyed<<'\n';}
}
static void resetRows() {
  int destroyed=0;auto array=retainedArray(destroyed);auto* original=array.blockAddress(0);
  auto* live=&array.EmplaceAppend(21,destroyed);array.Clear(false);
  std::cout<<"omitted_live value="<<live->value<<" destroyed="<<destroyed<<'\n';
  live->~RetainedPayload();array.SetIncrement(size_t(2));array.EmplaceAppend(22,destroyed);
  std::cout<<"reset value="<<array.First().value<<" same="<<(&array.First()==original)
           <<" blocks="<<array.blockCount()<<" destroyed="<<destroyed<<'\n';
  array.Clear(true);std::cout<<"reset_clear destroyed="<<destroyed<<" blocks="<<array.blockCount()<<'\n';
}
static RetainedArray<int> retainedIntegers() {
  RetainedArray<int> array(2);
  for(int value=1;value<=3;++value)array.Append(value);
  array.Clear(false);array.SetIncrement(std::numeric_limits<size_t>::max());return array;
}
static void appendRows() {
  auto borrowed=retainedIntegers();int donor=7;borrowed.Append(donor);
  auto moved=retainedIntegers();moved.Append(std::move(donor));
  auto appended=retainedIntegers();appended.Appended()=9;
  std::cout<<"append borrowed="<<borrowed.First()<<" moved="<<moved.Last()
           <<" default="<<appended.Value(0)<<" blocks="<<appended.blockCount()<<'\n';
  RetainedArray<int> active(2);active.Append(1);active.SetIncrement(std::numeric_limits<size_t>::max());active.Append(2);
  std::cout<<"active value="<<active.Last()<<" size="<<active.Size()<<" blocks="<<active.blockCount()<<'\n';
}
static void freedResetRows() {
  auto array=retainedIntegers();array.Append(1);array.Clear(true);
  std::cout<<"freed size="<<array.Size()<<" blocks="<<array.blockCount()<<'\n';
  array.SetIncrement(size_t(2));array.Append(33);
  std::cout<<"freed_reset value="<<array.First()<<" size="<<array.Size()<<" blocks="<<array.blockCount()<<'\n';
  RetainedArray<int> empty(std::numeric_limits<size_t>::max());empty.Clear(true);
  std::cout<<"empty size="<<empty.Size()<<" blocks="<<empty.blockCount()<<'\n';
}
static void throwingRows() {
  int destroyed=0;auto array=retainedArray(destroyed);bool caught=false;
  try {array.EmplaceAppend(-1,destroyed);}catch(const std::runtime_error&){caught=true;}
  std::cout<<"throw caught="<<caught<<" size="<<array.Size()<<" blocks="<<array.blockCount()
           <<" destroyed="<<destroyed<<'\n';
  array.EmplaceAppend(41,destroyed);array.EraseLast();
  std::cout<<"retry size="<<array.Size()<<" destroyed="<<destroyed<<'\n';
}
static void resourceRows() {
  using Resource=std::shared_ptr<int>;Resource owner=std::make_shared<int>(7);
  std::weak_ptr<int> witness(owner);RetainedArray<Resource> array(2);
  for(int index=0;index<3;++index)array.Append(owner);
  array.Clear(false);array.SetIncrement(std::numeric_limits<size_t>::max());
  auto* slot=&array.Append(std::move(owner));array.Clear(false);
  std::cout<<"omitted_resource strong="<<witness.use_count()<<" expired="<<witness.expired()<<'\n';
  slot->~Resource();
  std::cout<<"repaired_resource strong="<<witness.use_count()<<" expired="<<witness.expired()<<'\n';
}
int main(){retainedCase(false,true);retainedCase(false,false);retainedCase(true,false);
  resetRows();appendRows();freedResetRows();throwingRows();resourceRows();}
