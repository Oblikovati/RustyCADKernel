// Native nonthrowing destructors can observe removal's intermediate owner metadata.
#include <NCollection_List.hxx>
#include <iostream>
struct Item;
using List=NCollection_List<Item>;
static const List* observed=nullptr;
static const List::Iterator* observed_cursor=nullptr;
static bool recording=false;
struct Item {
 int value;
 explicit Item(int input):value(input){}
 ~Item();
};
Item::~Item(){
 if(recording)std::cout<<"drop "<<value<<' '<<observed->Size()<<' '<<observed->IsEmpty()<<' '<<(observed_cursor&&observed_cursor->More())<<'\n';
}
void removal(int count,int position){
 List list;for(int i=1;i<=count;++i)list.EmplaceAppend(i);
 List::Iterator cursor(list);for(int i=0;i<position;++i)cursor.Next();
 std::cout<<"remove "<<count<<' '<<position<<'\n';observed=&list;observed_cursor=&cursor;recording=true;
 list.Remove(cursor);recording=false;
 std::cout<<"after "<<list.Size()<<' '<<list.IsEmpty()<<' '<<cursor.More()<<'\n';
}
void clear(){
 List list;for(int i=1;i<=3;++i)list.EmplaceAppend(i);
 std::cout<<"clear\n";observed=&list;observed_cursor=nullptr;recording=true;list.Clear();recording=false;
 std::cout<<"after "<<list.Size()<<' '<<list.IsEmpty()<<'\n';
}
int main(){for(int count=1;count<=3;++count)for(int pos=0;pos<count;++pos)removal(count,pos);clear();}
