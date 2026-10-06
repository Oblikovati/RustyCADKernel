#include <NCollection_BaseSequence.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <type_traits>
#include <cstdint>
struct Value { int number; };
struct ForeignPrefix { uint64_t marker=42; };
struct CustomNode : ForeignPrefix, NCollection_SeqNode {
  ::Value value; mutable int reads=0;int writes=0;
  CustomNode(int n):value{n}{}
  const ::Value& ValueRef()const {++reads;return value;}
  const ::Value& Value()const {return ValueRef();}
  ::Value& ChangeValue(){++writes;return value;}
};
struct CustomSequence : NCollection_BaseSequence {
  template<bool Constant>using Cursor=BasicIterator<CustomNode,::Value,Constant>;
  CustomNode first{1},second{2};
  CustomSequence():NCollection_BaseSequence(nullptr){PAppend(&first);PAppend(&second);}
  void fields(){
    mySize=9;myCurrentIndex=7;myCurrentItem=&second;
    std::cout<<"raw_fields "<<Size()<<' '<<Length()<<' '<<IsEmpty()<<' '
      <<myCurrentIndex<<' '<<(myCurrentItem==&second)<<'\n';
    mySize=2;myCurrentIndex=1;myCurrentItem=&first;
  }
};
int main(){
  CustomSequence sequence;
  CustomSequence::Cursor<false> writable(sequence);
  auto previous=writable++;CustomSequence::Cursor<true> readonly(writable);
  CustomSequence::Cursor<false> legacy(readonly),assigned;assigned=readonly;
  NCollection_BaseSequence::Iterator base;readonly.CopyTo(base);
  CustomSequence::Cursor<true> restored(base);
  std::cout<<"converted "<<(writable==readonly)<<' '<<(legacy==readonly)<<' '
    <<(assigned==readonly)<<' '<<(restored==readonly)<<' '
    <<(base.PreviousNode()==previous.Node())<<'\n';
  (*legacy).number=8;legacy->number=9;legacy.ChangeValue().number=10;
  const int snapshot=legacy.Value().number;
  const int read=(*readonly).number+readonly->number;
  std::cout<<"dispatch "<<sequence.second.reads<<' '<<sequence.second.writes<<' '<<snapshot<<' '<<read<<'\n';
  auto end=CustomSequence::Cursor<false>(sequence,false);auto old=end--;
  std::cout<<"movement "<<old.More()<<' '<<(end.Node()==&sequence.second)<<' '
    <<(end.MutablePreviousNode()==&sequence.first)<<' '<<((&--end)==&end)<<' '
    <<(end.Node()==&sequence.first)<<'\n';
  CustomSequence other;CustomSequence::Cursor<true> otherEnd(other,false);
  end.Initialize(sequence,false);
  std::cout<<"ends "<<(end==otherEnd)<<' '<<(end!=otherEnd)<<' '<<(end.CurrentNode()==nullptr)<<'\n';
  std::cout<<"adjusted_base "<<(reinterpret_cast<char*>(static_cast<NCollection_SeqNode*>(&sequence.first))!=reinterpret_cast<char*>(&sequence.first))<<' '<<sequence.first.marker<<'\n';
  sequence.fields();
}
