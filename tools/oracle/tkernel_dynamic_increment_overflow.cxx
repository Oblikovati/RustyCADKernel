// -fno-access-control observes private metadata without editing pinned OCCT 3d097a0.
// All arithmetic is unsigned; never allocate or dereference elements with overflowed zero stride.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <climits>
#include <limits>
#include <string>
using Array=NCollection_DynamicArray<int>;
static void row(const std::string& name,const Array& array) {
  std::cout<<name<<' '<<array.myInternalSize<<' '<<array.myBlockShift<<' '<<array.myBlockMask
           <<' '<<array.Size()<<' '<<array.Length()<<' '<<array.Lower()<<' '<<array.Upper()
           <<' '<<array.IsEmpty()<<' '<<array.myContainer.Size()<<' '<<array.availableSize()
           <<' '<<!array.Allocator().IsNull()<<' '<<(array.begin()==array.end())<<'\n';
}
static void constructors() {
  const size_t high=size_t(1)<<(std::numeric_limits<size_t>::digits-1);
  const size_t values[]={0,1,2,3,high-1,high,high+1,SIZE_MAX-1,SIZE_MAX};
  occ::handle<NCollection_BaseAllocator> allocator=new NCollection_BaseAllocator;
  NCollection_OccAllocator<int> typed(allocator);
  for(int i=0;i<9;++i) {
    Array first(values[i]), second(values[i],allocator), third(values[i],typed);
    row("usize_plain_"+std::to_string(i),first);row("usize_handle_"+std::to_string(i),second);row("usize_typed_"+std::to_string(i),third);
    Array changed(3);changed.SetIncrement(values[i]);row("set_"+std::to_string(i),changed);
  }
  const int signedValues[]={INT_MIN,-1,0,1,3,INT_MAX};
  for(int i=0;i<6;++i) {
    Array first(signedValues[i]), second(signedValues[i],allocator), third(signedValues[i],typed);
    row("signed_plain_"+std::to_string(i),first);row("signed_handle_"+std::to_string(i),second);row("signed_typed_"+std::to_string(i),third);
  }
}
static void transfers() {
  occ::handle<NCollection_BaseAllocator> allocator=new NCollection_BaseAllocator;
  Array source(SIZE_MAX,allocator), copied(source); row("copy_zero",copied);
  Array moved(std::move(source));row("move_zero",moved);row("moved_source_zero",source);
  Array assigned(2);assigned.Assign(copied,false);row("assign_zero",assigned);
  Array receiver(2);receiver.Assign(std::move(assigned));row("move_assign_zero",receiver);row("move_assign_source_zero",assigned);
}
static void retained() {
  Array array(2);array.Append(1);array.Append(2);array.Append(3);array.Clear(false);
  array.SetIncrement(SIZE_MAX);row("retained_zero",array);
  array.SetIncrement(2);array.Append(77);row("retained_reset",array);
  array.SetIncrement(SIZE_MAX);row("live_unchanged",array);
  Array empty(SIZE_MAX);empty.SetIncrement(size_t(0));empty.Append(9);row("zero_reset",empty);
}
int main(){constructors();transfers();retained();}
