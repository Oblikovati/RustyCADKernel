// Native BaseList subclass exposes protected methods without relying on typed List storage.
#include <NCollection_BaseList.hxx>
#include <NCollection_TListNode.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <vector>
class ExposedList : public NCollection_BaseList {
public:
  explicit ExposedList(const occ::handle<NCollection_BaseAllocator>& a=nullptr):NCollection_BaseList(a) {}
  using NCollection_BaseList::PAppend; using NCollection_BaseList::PPrepend;
  using NCollection_BaseList::PInsertBefore;using NCollection_BaseList::PInsertAfter;
  using NCollection_BaseList::PRemove;using NCollection_BaseList::PRemoveFirst;
  using NCollection_BaseList::PReverse;using NCollection_BaseList::PClear;
  using NCollection_BaseList::PExchange;using NCollection_BaseList::PFirst;using NCollection_BaseList::PLast;
  NCollection_ListNode* make(int value) {return new(myAllocator) NCollection_TListNode<int>(value);}
  ~ExposedList() {PClear(NCollection_TListNode<int>::delNode);}
};
class ExposedCursor : public NCollection_BaseList::Iterator {
public:
  using NCollection_BaseList::Iterator::Iterator;
  using NCollection_BaseList::Iterator::init;
};
static int item(const NCollection_ListNode* node) {return static_cast<const NCollection_TListNode<int>*>(node)->Value();}
static void dump(const char* name,const ExposedList& list) {
  std::cout<<name<<' '<<list.Size();
  for(auto* node=list.PFirst();node;node=node->Next())std::cout<<' '<<item(node);
  std::cout<<'\n';
}
static std::vector<int> removed;
static void record_delete(NCollection_ListNode* node,occ::handle<NCollection_BaseAllocator>& allocator) {
  removed.push_back(item(node));NCollection_TListNode<int>::delNode(node,allocator);
}
static void individual() {
  ExposedList list;ExposedCursor cursor;
  list.PAppend(list.make(1),cursor);auto* first=cursor.myCurrent;
  std::cout<<"first_append "<<list.Extent()<<' '<<list.Length()<<' '<<(cursor.myPrevious==nullptr)<<' '<<(list.PLast()==first)<<'\n';
  list.PAppend(list.make(2),cursor);list.PInsertBefore(list.make(3),cursor);
  dump("before",list);auto* predecessor=cursor.myPrevious;
  list.PRemove(cursor,record_delete);
  std::cout<<"remove "<<cursor.More()<<' '<<(cursor.myPrevious==predecessor)<<' '<<item(list.PLast())<<'\n';
  cursor.Init(list);list.PInsertAfter(list.make(4),cursor);list.PReverse();dump("reverse",list);
  removed.clear();list.PClear(record_delete);
  std::cout<<"clear "<<list.IsEmpty()<<' '<<list.Size()<<' '<<(list.PFirst()==nullptr)<<' '<<(list.PLast()==nullptr);
  for(int value:removed)std::cout<<' '<<value;std::cout<<'\n';
}
static void splicing() {
  ExposedList destination,source;
  destination.PAppend(destination.make(1));destination.PAppend(destination.make(4));
  source.PAppend(source.make(2));source.PAppend(source.make(3));
  ExposedCursor cursor(destination);destination.PInsertAfter(source,cursor);dump("after_list",destination);
  std::cout<<"donor_empty "<<source.IsEmpty()<<' '<<source.Size()<<' '<<(source.PLast()==nullptr)<<'\n';
  source.PAppend(source.make(0));destination.PInsertBefore(source,cursor);dump("before_head_list",destination);
  std::cout<<"before_head_cursor "<<item(cursor.myCurrent)<<' '<<item(cursor.myPrevious)<<'\n';
  source.PAppend(source.make(5));destination.PAppend(source);destination.PPrepend(destination);dump("append_self_prepend",destination);
  destination.PAppend(destination);source.PAppend(source.make(-1));destination.PPrepend(source);dump("prepend_list",destination);
}
static void middle_and_empty() {
  ExposedList destination,source;
  destination.PAppend(destination.make(1));destination.PAppend(destination.make(4));
  ExposedCursor cursor;cursor.init(destination,const_cast<NCollection_ListNode*>(destination.PFirst()));
  source.PAppend(source.make(2));source.PAppend(source.make(3));destination.PInsertBefore(source,cursor);
  dump("middle_before",destination);auto* previous=cursor.myPrevious;destination.PInsertBefore(source,cursor);
  std::cout<<"empty_before "<<(cursor.myPrevious==previous)<<' '<<cursor.More()<<'\n';
  source.PAppend(source.make(5));destination.PInsertAfter(source,cursor);dump("tail_after",destination);
  cursor.init(destination,nullptr);std::cout<<"init_null_prev "<<item(cursor.myCurrent)<<' '<<(cursor.myPrevious==nullptr)<<'\n';
  ExposedList empty;empty.PAppend(destination);dump("append_into_empty",empty);empty.PRemoveFirst(record_delete);dump("remove_first",empty);
}
static void failures_exchange() {
  ExposedList list,source;ExposedCursor cursor;int errors=0;
  try{list.PRemoveFirst(record_delete);}catch(const Standard_NoSuchObject&){++errors;}
  try{list.PRemove(cursor,record_delete);}catch(const Standard_NoSuchObject&){++errors;}
  try{list.PInsertBefore(source,cursor);}catch(const Standard_NoSuchObject&){++errors;}
  try{list.PInsertAfter(source,cursor);}catch(const Standard_NoSuchObject&){++errors;}
  std::cout<<"empty_errors "<<errors<<'\n';
  occ::handle<NCollection_BaseAllocator> arena=new NCollection_IncAllocator;
  ExposedList foreign(arena);foreign.PAppend(foreign.make(7));list.PExchange(foreign);
  std::cout<<"exchange "<<item(list.PFirst())<<' '<<(list.Allocator()==arena)<<' '<<foreign.IsEmpty()<<'\n';
}
int main(){individual();splicing();middle_and_empty();failures_exchange();}
