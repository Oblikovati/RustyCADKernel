// Protected BaseSequence engine exercised through a native subclass.
#include <NCollection_Sequence.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
struct RawSequence : NCollection_BaseSequence {
  using Node=NCollection_Sequence<int>::Node;
  RawSequence():NCollection_BaseSequence(nullptr){}
  ~RawSequence(){ClearSeq(NCollection_Sequence<int>::delNode);}
  Node* node(int value){return new(myAllocator) Node(value);}
  void append(int value){PAppend(node(value));}
  void prepend(int value){PPrepend(node(value));}
  void after(size_t index,int value){PInsertAfter(index,node(value));}
  void after(Iterator& position,int value){PInsertAfter(position,node(value));}
  void cache(size_t index){myCurrentItem=Find(index);myCurrentIndex=index;}
  void show(const char* name){
    std::cout<<name<<' '<<Size()<<' '<<myCurrentIndex<<' '<<(myCurrentItem?((Node*)myCurrentItem)->Value():-1)<<" :";
    for(auto node=myFirstItem;node;node=node->Next())std::cout<<' '<<((Node*)node)->Value();
    std::cout<<'\n';
  }
  using NCollection_BaseSequence::PAppend;
  using NCollection_BaseSequence::PPrepend;
  using NCollection_BaseSequence::PInsertAfter;
  using NCollection_BaseSequence::PSplit;
  using NCollection_BaseSequence::PReverse;
  using NCollection_BaseSequence::PExchange;
  using NCollection_BaseSequence::RemoveSeq;
  using NCollection_BaseSequence::ClearSeq;
};
void edits(){
  RawSequence s;s.show("empty");s.append(1);s.append(2);s.append(3);s.cache(2);
  s.prepend(0);s.show("prepend_cache");s.after(1,8);s.show("index_insert_cache");
  NCollection_BaseSequence::Iterator cursor(s,true);s.after(cursor,9);s.show("cursor_insert_cache");
  s.cache(4);s.PReverse();s.show("reverse_cache");s.PExchange(1,6);s.show("exchange_edges");
  s.PExchange(2,3);s.show("exchange_neighbors");s.PExchange(3,3);s.show("exchange_equal");
}
void transfers(){
  RawSequence a,b;a.append(1);a.append(2);a.cache(2);b.append(7);b.append(8);
  a.PInsertAfter(1,b);a.show("insert_sequence");b.show("source_empty");
  b.append(9);a.PPrepend(b);a.show("prepend_sequence");b.append(10);a.PAppend(b);a.show("append_sequence");
  a.cache(4);a.PSplit(3,b);a.show("split_head");b.show("split_tail");
  RawSequence empty;try{empty.PInsertAfter(1,b);}catch(const Standard_Failure&){std::cout<<"invalid_insert 1\n";}
  try{b.PSplit(1,b);}catch(const Standard_Failure&){std::cout<<"self_split 1\n";}
}
void removals(){
  RawSequence a;for(int i=1;i<=6;i++)a.append(i);a.cache(4);
  a.RemoveSeq(2,NCollection_Sequence<int>::delNode);a.show("remove_before_cache");
  a.RemoveSeq(3,NCollection_Sequence<int>::delNode);a.show("remove_at_cache");
  a.RemoveSeq(2,3,NCollection_Sequence<int>::delNode);a.show("remove_range_cache");
  NCollection_BaseSequence::Iterator cursor(a,false);cursor.Previous();
  a.RemoveSeq(cursor,NCollection_Sequence<int>::delNode);a.show("remove_cursor_cache");
  a.RemoveSeq(cursor,NCollection_Sequence<int>::delNode);a.show("remove_end_noop");
  a.ClearSeq(NCollection_Sequence<int>::delNode);a.show("clear_cache");
}
int main(){edits();transfers();removals();}
