#include <NCollection_Sequence.hxx>
#include <iostream>
#include <vector>
struct Item;
struct Sequence:NCollection_Sequence<Item>{
 const NCollection_SeqNode* head() const{return myFirstItem;}
 const NCollection_SeqNode* tail() const{return myLastItem;}
 const NCollection_SeqNode* cached() const{return myCurrentItem;}
 size_t cache_index() const{return myCurrentIndex;}
};
static const Sequence* observed=nullptr;
static const Sequence::Iterator* cursor_view=nullptr;
static bool recording=false;
static std::vector<const NCollection_SeqNode*> identities;
struct Item {int value;explicit Item(int input):value(input){}~Item();};
int identity(const NCollection_SeqNode* node){if(!node)return 0;for(size_t i=0;i<identities.size();++i)if(identities[i]==node)return int(i+1);return -1;}
void fields(){
 std::cout<<' '<<observed->Size()<<' '<<observed->IsEmpty()<<' '<<identity(observed->head())<<' '<<identity(observed->tail());
 std::cout<<' '<<identity(observed->cached())<<' '<<observed->cache_index()<<' '<<(cursor_view&&cursor_view->More());
 std::cout<<' '<<identity(cursor_view?cursor_view->CurrentNode():nullptr)<<' '<<identity(cursor_view?cursor_view->PreviousNode():nullptr)<<'\n';
}
Item::~Item(){if(recording){std::cout<<"drop "<<value;fields();}}
void fill(Sequence& sequence,int count,int cache){
 identities.clear();for(int i=1;i<=count;++i)sequence.EmplaceAppend(i);
 for(Sequence::Iterator cursor(sequence);cursor.More();cursor.Next())identities.push_back(cursor.CurrentNode());
 if(cache)(void)sequence.Value(cache);
}
void run(int kind,int count,int cache,int first,int last){
 Sequence sequence;fill(sequence,count,cache);Sequence::Iterator cursor(sequence);
 if(kind==1)for(int i=0;i<first;++i)cursor.Next();
 observed=&sequence;cursor_view=kind==1?&cursor:nullptr;
 std::cout<<"case "<<kind<<' '<<count<<' '<<cache<<' '<<first<<' '<<last<<'\n';recording=true;
 if(kind==0)sequence.Remove(first);else if(kind==1)sequence.Remove(cursor);else if(kind==2)sequence.Remove(first,last);else sequence.Clear();
 recording=false;std::cout<<"after";fields();
}
int main(){
 for(int count=0;count<=3;++count)for(int cache=0;cache<=count;++cache){
  for(int index=1;index<=count;++index)run(0,count,cache,index,0);
  for(int pos=0;pos<=count;++pos)run(1,count,cache,pos,0);
  for(int from=1;from<=count;++from)for(int to=from;to<=count;++to)run(2,count,cache,from,to);
  run(3,count,cache,0,0);
 }
}
