// Growth destructor exceptions leave native registered holes; never read those slots.
#include <NCollection_LinearVector.hxx>
#include <Standard.hxx>
#include <iostream>
#include <memory>
#include <string>
#include <utility>
static bool active=false;
static int resources=0, failDrop=-1, failRepair=-1;
static size_t sourceSize=0, sourceCapacity=0, deadEnd=0, repaired=0;
static void* sourcePointer=nullptr;
static void* replacementPointer=nullptr;
static std::string events;
static size_t (*ownerSize)()=nullptr;
static size_t (*ownerCapacity)()=nullptr;
static bool (*ownerUnpublished)()=nullptr;
static const void* (*sourceSlot)(size_t)=nullptr;
struct Payload {Payload(){++resources;}~Payload(){--resources;}};
struct ItemBase {
 int value;std::unique_ptr<Payload> payload;
 explicit ItemBase(int initial):value(initial),payload(new Payload){if(initial==failRepair)throw initial;}
 ItemBase(const ItemBase& source):value(source.value),payload(new Payload){constructed('C');}
 ItemBase(ItemBase&& source):value(source.value),payload(std::move(source.payload)){source.value=-1;constructed('M');}
 void constructed(char mode){if(active){if(!replacementPointer)replacementPointer=this;events+=std::string(1,mode)+std::to_string(value)+',';}}
 ~ItemBase() noexcept(false){
  if(!active)return;
  size_t index=sourceSize;
  for(size_t i=0;i<sourceSize;++i)if(this==sourceSlot(i)){index=i;break;}
  events+="D"+std::to_string(value)+"@"+(index==sourceSize?"X":std::to_string(index))+"("+std::to_string(ownerSize())+"/"+std::to_string(ownerCapacity())+"/"+std::to_string(ownerUnpublished())+"),";
  if(index<sourceSize){deadEnd=index+1;if(int(index)==failDrop)throw int(index);}
 }
};
struct CopyItem:ItemBase{using ItemBase::ItemBase;CopyItem(const CopyItem&)=default;};
struct MoveItem:ItemBase{using ItemBase::ItemBase;MoveItem(const MoveItem&)=delete;MoveItem(MoveItem&&)=default;};
template<class T> static NCollection_LinearVector<T>* owner=nullptr;
struct Spec{const char* name;size_t capacity,count,minimum;bool moving;int fail,repairFailure;};
template<class T> void values(const T* pointer,size_t count,size_t dead,size_t restored){
 for(size_t i=0;i<count;++i){if(i>=restored&&i<dead){std::cout<<"X,";continue;}std::cout<<pointer[i].value<<'/'<<bool(pointer[i].payload)<<',';}
}
template<class T> void emit(const char* phase,const Spec& spec,bool success){
 const auto& vector=*owner<T>;
 std::cout<<phase<<':'<<spec.name<<':'<<(success?"ok":"throw")<<':'<<vector.Size()<<':'<<vector.Capacity()<<':'<<(vector.Data()==sourcePointer)<<':'<<deadEnd<<':'<<repaired<<':'<<events<<':';
 values(vector.Data(),vector.Size(),success?0:deadEnd,repaired);std::cout<<':'<<resources<<'\n';
}
template<class T> void restore(const Spec& spec){
 try {for(;repaired<deadEnd;++repaired){events+="R"+std::to_string(repaired)+',';new(owner<T>->Data()+repaired)T(int(100+repaired));}}
 catch(int){emit<T>("repair-failure",spec,false);failRepair=-1;restore<T>(spec);}
}
template<class T> void reclaim(const Spec& spec){
 const size_t capacity=std::max(sourceCapacity==0?size_t(2):sourceCapacity*2,spec.minimum);
 std::cout<<"replacement:"<<spec.name<<':'<<capacity<<':'<<sourceSize<<':';values(static_cast<T*>(replacementPointer),sourceSize,0,0);std::cout<<':'<<resources<<'\n';
 restore<T>(spec);emit<T>("restored",spec,false);
 for(size_t i=0;i<sourceSize;++i)static_cast<T*>(replacementPointer)[i].~T();
 Standard::Free(replacementPointer);emit<T>("reclaimed",spec,false);
}
template<class T> void run(const Spec& spec){
 owner<T> =new NCollection_LinearVector<T>(spec.capacity);
 for(size_t i=0;i<spec.count;++i)owner<T>->EmplaceAppend(int(10+i));
 sourcePointer=owner<T>->Data();sourceSize=spec.count;sourceCapacity=spec.capacity;replacementPointer=nullptr;deadEnd=repaired=0;failDrop=spec.fail;failRepair=spec.repairFailure;events.clear();active=true;
 sourceSlot=[](size_t index)->const void*{return static_cast<ItemBase*>(static_cast<T*>(sourcePointer)+index);};
 ownerSize=[](){return owner<T>->Size();};ownerCapacity=[](){return owner<T>->Capacity();};ownerUnpublished=[](){return owner<T>->Data()==sourcePointer;};
 bool success=true;try{owner<T>->Reserve(spec.minimum);}catch(int){success=false;}
 if(success)deadEnd=0;
 emit<T>("native",spec,success);
 if(!success){failDrop=-1;reclaim<T>(spec);}
 active=false;delete owner<T>;owner<T> =nullptr;
}
int main(){
 const Spec cases[]={
 {"empty-noop",0,0,0,false,-1,-1},{"empty-grow",0,0,1,false,-1,-1},{"retained-noop",4,3,4,false,-1,-1},
 {"copy-success",4,3,5,false,-1,-1},{"copy-first",4,3,5,false,0,-1},{"copy-middle",4,3,5,false,1,-1},{"copy-final",4,3,5,false,2,-1},
 {"move-success",4,3,5,true,-1,-1},{"move-first",4,3,5,true,0,-1},{"move-middle",4,3,5,true,1,-1},{"move-final",4,3,5,true,2,-1},
 {"copy-repair-first",4,3,5,false,2,100},{"copy-repair-late",4,3,5,false,2,101},
 {"move-repair-first",4,3,5,true,2,100},{"move-repair-late",4,3,5,true,2,101},{"copy-repair-final",4,3,5,false,2,102},{"move-repair-final",4,3,5,true,2,102},
 };
 for(const auto& spec:cases){if(spec.moving)run<MoveItem>(spec);else run<CopyItem>(spec);if(resources!=0)return 1;}
}
