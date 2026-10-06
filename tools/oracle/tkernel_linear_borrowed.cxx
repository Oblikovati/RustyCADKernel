#include <NCollection_LinearVector.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct BorrowedItem {
  static bool failCopy,failAssign;static std::string events;int value=0;
  BorrowedItem() { events+='D'; }
  explicit BorrowedItem(int input):value(input) {}
  BorrowedItem(const BorrowedItem& donor):value(donor.value) {
    events+='C';if(failCopy)throw std::runtime_error("copy");
  }
  BorrowedItem& operator=(const BorrowedItem& donor) {
    events+='A';value=donor.value+100;
    if(failAssign)throw std::runtime_error("assignment");
    value=donor.value;return *this;
  }
};
bool BorrowedItem::failCopy=false;bool BorrowedItem::failAssign=false;std::string BorrowedItem::events;
static void borrowedRow(const char* name,const NCollection_LinearVector<BorrowedItem>& target,const BorrowedItem& donor) {
  std::cout<<name<<' '<<target.Size()<<' '<<target.Capacity()<<' '<<donor.value<<' '<<BorrowedItem::events<<' ';
  for(size_t index=0;index<target.Size();++index) { if(index)std::cout<<',';std::cout<<target.Value(index).value; }
  if(target.IsEmpty())std::cout<<'-';std::cout<<'\n';
}
static void appendCase(const char* name,size_t capacity,bool fail) {
  NCollection_LinearVector<BorrowedItem> target(capacity);BorrowedItem donor(9);
  BorrowedItem::events.clear();BorrowedItem::failCopy=fail;
  try { target.Append(donor); }catch(const std::runtime_error&) {}
  borrowedRow(name,target,donor);BorrowedItem::failCopy=false;
}
static void setCase(const char* name,size_t index,bool fail) {
  NCollection_LinearVector<BorrowedItem> target(8);target.EmplaceAppend(3);BorrowedItem donor(9);
  BorrowedItem::events.clear();BorrowedItem::failAssign=fail;
  try { target.SetValue(index,donor); }catch(const std::runtime_error&) {}
  borrowedRow(name,target,donor);BorrowedItem::failAssign=false;
}
int main() {
  appendCase("append_grow_success",0,false);appendCase("append_grow_failed",0,true);
  appendCase("append_retained_success",4,false);appendCase("append_retained_failed",4,true);
  setCase("set_existing_success",0,false);setCase("set_existing_failed",0,true);
  setCase("set_gap_success",3,false);setCase("set_gap_failed",3,true);
}
