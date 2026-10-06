// OCCT 3d097a0: assignment copies cached snapshots, Init captures current storage and end.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <limits>
#include <utility>
using Array=NCollection_DynamicArray<int>;
static Array sourceArray() {
  Array source(2);
  for(int value=1;value<=4;++value) source.Append(value);
  return source;
}
template<class Left,class Right> static void comparisonRow(const char* label,const Left& left,const Right& right) {
  std::cout<<label<<" eq="<<(left==right)<<" ne="<<(left!=right)<<" lt="<<(left<right)
           <<" gt="<<(left>right)<<" le="<<(left<=right)<<" ge="<<(left>=right)<<'\n';
}
static void assignmentRows(Array& source,Array& target,const Array::iterator& saved) {
  Array::iterator assigned;const bool returned=&(assigned=saved)==&assigned;
  Array::const_iterator constant;const bool converted=&(constant=saved)==&constant;
  const auto pointer=assigned.operator->();const bool self=&(assigned=assigned)==&assigned;
  std::cout<<"assign value="<<*assigned<<" target="<<(pointer==&target.Value(1))
           <<" returned="<<returned<<" self="<<self<<" unchanged="<<(pointer==assigned.operator->())<<'\n';
  std::cout<<"const_assign value="<<*constant<<" same="<<(constant.operator->()==pointer)
           <<" returned="<<converted<<'\n';
  for(int value=101;value<=106;++value) source.Append(value);
  comparisonRow("same_owner_new_pointer",assigned,source.cbegin()+1);
  comparisonRow("different_owner_same_pointer",assigned,target.cbegin()+1);
  assigned+=3;constant+=3;
  std::cout<<"assigned_end mutable="<<assigned.More()<<" constant="<<constant.More()
           <<" difference="<<(assigned-source.cend())<<'\n';
}
static void initializationRows(Array& source,Array& target) {
  Array::Iterator cursor(target);cursor.Init(source);
  std::cout<<"init_mutable value="<<cursor.Value()<<" source="<<(cursor.operator->()==&source.Value(0))<<'\n';
  cursor.Initialize(target);
  std::cout<<"initialize_other value="<<cursor.Value()<<" target="<<(cursor.operator->()==&target.Value(0))<<'\n';
  const Array& constantSource=source;cursor.Init(constantSource);cursor.ChangeValue()=201;
  std::cout<<"init_const_argument write="<<source.Value(0)<<'\n';
  cursor.Initialize(constantSource);cursor.ChangeValue()=301;
  std::cout<<"initialize_const_argument write="<<source.Value(0)<<'\n';
  Array::ConstIterator readOnly(target);readOnly.Init(source);
  std::cout<<"const_init value="<<readOnly.Value()<<'\n';
  readOnly.Initialize(target);
  std::cout<<"const_initialize value="<<readOnly.Value()<<'\n';
}
static void endSnapshotRows(Array& source) {
  Array::ConstIterator cursor;cursor.Init(source);source.Append(107);cursor+=6;
  std::cout<<"initialized_end more="<<cursor.More()<<" difference="<<(cursor-source.cend())<<'\n';
  cursor.Initialize(source);cursor+=6;
  std::cout<<"reinitialized_end more="<<cursor.More()<<" value="<<cursor.Value()<<'\n';
  Array empty(2);cursor.Init(empty);
  std::cout<<"empty_init more="<<cursor.More()<<" same="<<(cursor==empty.cbegin())<<'\n';
}
static void comparisonRows(Array& source,Array& target) {
  comparisonRow("cross_owner_zero",source.begin(),target.cbegin());
  comparisonRow("cross_const_before",source.begin(),source.cend());
  comparisonRow("cross_const_after",source.cend(),source.begin());
  const size_t minimum=static_cast<size_t>(std::numeric_limits<std::ptrdiff_t>::min());
  Array::const_iterator negative(source,minimum);const auto zero=source.cbegin();
  std::cout<<"signed_min_left lt="<<(negative<zero)<<" ge="<<(negative>=zero)<<'\n';
  std::cout<<"signed_min_right gt="<<(zero>negative)<<" le="<<(zero<=negative)<<'\n';
}
int main() {
  auto source=sourceArray();auto saved=source.begin();++saved;Array target(std::move(source));
  Array::Iterator emptyCapture(target);emptyCapture.Init(source);
  std::cout<<"moved_empty_init more="<<emptyCapture.More()<<'\n';
  assignmentRows(source,target,saved);initializationRows(source,target);
  endSnapshotRows(source);comparisonRows(source,target);
}
