// OCCT 3d097a0: cached pointer lifetime follows transferred allocation, reload follows original owner.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <utility>
using Array=NCollection_DynamicArray<int>;
static Array sourceArray() {
  Array source(2);for(int value=1;value<=6;++value) source.Append(value);return source;
}
static void row(const char* label,const Array::iterator& cursor,Array& source,Array& target) {
  Array::const_iterator constant(cursor);
  std::cout<<label<<" value="<<*cursor<<" same="<<(cursor.operator->()==&target.Value(0))
           <<" more="<<cursor.More()<<" source="<<source.Size()<<" target="<<target.Size()
           <<" equal="<<(cursor==target.begin())<<" converted="<<(constant.operator->()==cursor.operator->())<<'\n';
}
static void trace(Array& source,Array& target,Array::iterator cursor,const char* mode) {
  row(mode,cursor,source,target);*cursor=11;
  std::cout<<"write="<<target.Value(0)<<'\n';
  ++cursor;std::cout<<"within="<<*cursor<<" target="<<(cursor.operator->()==&target.Value(1))<<'\n';
  for(int value=101;value<=106;++value) source.Append(value);
  auto reloaded=cursor;reloaded+=0;
  std::cout<<"cached="<<*cursor<<" offset_zero="<<*reloaded<<'\n';
  ++cursor;std::cout<<"boundary="<<*cursor<<" source="<<(cursor.operator->()==&source.Value(2))<<'\n';
  --cursor;std::cout<<"retreat="<<*cursor<<" source="<<(cursor.operator->()==&source.Value(1))<<'\n';
}
static void constructorCase() {
  auto source=sourceArray();auto cursor=source.begin();Array target(std::move(source));
  trace(source,target,cursor,"constructor");
}
static void assignmentCase() {
  auto source=sourceArray();auto cursor=source.begin();Array target(4);target.Append(90);
  target.Assign(std::move(source));trace(source,target,cursor,"assignment");
}
static void originalOwnerDestroyed() {
  Array target(2);Array::iterator cursor;
  { auto source=sourceArray();cursor=source.begin();target.Assign(std::move(source)); }
  std::cout<<"owner_destroyed value="<<*cursor<<" same="<<(cursor.operator->()==&target.Value(0))<<'\n';
}
int main() { constructorCase();assignmentCase();originalOwnerDestroyed(); }
