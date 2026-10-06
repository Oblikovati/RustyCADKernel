// OCCT 3d097a0: postfix retains pointer state, displacement and subscript reload the original owner.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <utility>
using Array=NCollection_DynamicArray<int>;
static Array sourceArray() {
  Array source(2);
  for(int value=1;value<=6;++value) source.Append(value);
  return source;
}
template<class Cursor> static void postfixRows(Cursor& cursor,Array& target) {
  auto previous=cursor++;
  std::cout<<"postfix_plus old="<<*previous<<" now="<<*cursor
           <<" old_target="<<(previous.operator->()==&target.Value(0))<<'\n';
  previous=cursor--;
  std::cout<<"postfix_minus old="<<*previous<<" now="<<*cursor
           <<" old_target="<<(previous.operator->()==&target.Value(1))<<'\n';
}
template<class Cursor> static void arithmeticRows(const Cursor& cursor,Array& source,Array& target) {
  auto zero=cursor+0;auto plus=cursor+2;auto minus=plus-1;
  std::cout<<"plus_zero="<<*zero<<" original="<<*cursor
           <<" source="<<(zero.operator->()==&source.Value(0))<<'\n';
  std::cout<<"plus="<<*plus<<" minus="<<*minus<<" original="<<*cursor<<'\n';
  minus-= -1;
  std::cout<<"subtract_negative="<<*minus<<" cross_const="<<(plus-Array::const_iterator(cursor))
           <<" cross_owner="<<(cursor-target.cend())<<'\n';
  std::cout<<"subscript="<<cursor[0]<<" source="<<(&cursor[0]==&source.Value(0))
           <<" left_plus="<<*(2+cursor)<<" original="<<*cursor<<'\n';
}
template<class Cursor> static void endRows(const Cursor& cursor) {
  auto end=cursor+6;auto previous=end--;
  std::cout<<"postfix_end old_more="<<previous.More()<<" now="<<*end<<'\n';
  previous=end++;
  std::cout<<"postfix_last old="<<*previous<<" now_more="<<end.More()<<'\n';
}
template<class Cursor> static void trace(Array& source,Cursor cursor,const char* mode) {
  Array target(std::move(source));std::cout<<mode<<'\n';
  postfixRows(cursor,target);
  for(int value=101;value<=106;++value) source.Append(value);
  arithmeticRows(cursor,source,target);endRows(cursor);
}
static void mutableWrite() {
  auto source=sourceArray();auto cursor=source.begin();Array target(std::move(source));
  for(int value=101;value<=106;++value) source.Append(value);
  cursor[0]=1001;
  std::cout<<"subscript_write source="<<source.Value(0)<<" target="<<target.Value(0)
           <<" original="<<*cursor<<'\n';
}
int main() {
  auto shared=sourceArray();trace(shared,shared.cbegin(),"shared");
  auto mutableArray=sourceArray();trace(mutableArray,mutableArray.begin(),"mutable");
  mutableWrite();
}
