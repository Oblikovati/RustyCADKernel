// Defined nonthrowing observations of the canonical owner and cursor, including endpoints.
#include <NCollection_List.hxx>
#include <iostream>
#include <vector>
struct Item;
struct List:NCollection_List<Item>{
 const NCollection_ListNode* head() const{return pFirst();}
 const NCollection_ListNode* tail() const{return pLast();}
};
static const List* observed=nullptr;
static const List::Iterator* observed_cursor=nullptr;
static bool recording=false;
static std::vector<const NCollection_ListNode*> identities;
struct Item {int value;explicit Item(int input):value(input){}~Item();};
int identity(const NCollection_ListNode* node){if(!node)return 0;for(size_t i=0;i<identities.size();++i)if(identities[i]==node)return int(i+1);return -1;}
void fields(){
 std::cout<<' '<<observed->Size()<<' '<<observed->IsEmpty()<<' '<<(observed_cursor&&observed_cursor->More());
 std::cout<<' '<<identity(observed->head())<<' '<<identity(observed->tail());
 std::cout<<' '<<identity(observed_cursor?observed_cursor->myCurrent:nullptr)<<' '<<identity(observed_cursor?observed_cursor->myPrevious:nullptr)<<'\n';
}
Item::~Item(){if(recording){std::cout<<"drop "<<value;fields();}}
void fill(List& list,int count){
 identities.clear();for(int i=1;i<=count;++i)list.EmplaceAppend(i);
 for(List::Iterator cursor(list);cursor.More();cursor.Next())identities.push_back(cursor.myCurrent);
}
void removal(int count,int position){
 List list;fill(list,count);List::Iterator cursor(list);for(int i=0;i<position;++i)cursor.Next();
 std::cout<<"remove "<<count<<' '<<position<<'\n';observed=&list;observed_cursor=&cursor;recording=true;
 list.Remove(cursor);recording=false;std::cout<<"after";fields();
}
void clear(){
 List list;fill(list,3);std::cout<<"clear\n";observed=&list;observed_cursor=nullptr;recording=true;
 list.Clear();recording=false;std::cout<<"after";fields();
}
int main(){for(int count=1;count<=3;++count)for(int pos=0;pos<count;++pos)removal(count,pos);clear();}
