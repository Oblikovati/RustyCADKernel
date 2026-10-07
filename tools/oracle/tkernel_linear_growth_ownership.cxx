// Native relocation failure retains replacement storage; explicit repair is fixture-only.
#include <NCollection_LinearVector.hxx>
#include <Standard.hxx>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
static bool active=false;
static size_t attempts=0, completed=0, failing=0;
static int resources=0;
static std::string events;
static void* firstAttempt=nullptr;
struct Payload { Payload(){++resources;} ~Payload(){--resources;} };
struct GrowthBase {
  int value;
  std::unique_ptr<Payload> payload;
  explicit GrowthBase(int initial=99):value(initial),payload(new Payload) {}
  GrowthBase(const GrowthBase& source):value(source.value),payload(new Payload) { record('C'); }
  GrowthBase(GrowthBase&& source):value(source.value),payload(std::move(source.payload))
  { source.value=-1; record('M'); }
  GrowthBase& operator=(const GrowthBase& source) {value=source.value;return *this;}
  void record(char mode) {
    if(!active)return;
    if(firstAttempt==nullptr)firstAttempt=this;
    events+=std::string(1,mode)+std::to_string(value)+',';
    if(++attempts==failing)throw std::runtime_error("injected growth relocation failure");
    ++completed;
  }
  ~GrowthBase(){if(active)events+="D"+std::to_string(value)+',';}
};
struct CopyGrowth:GrowthBase { using GrowthBase::GrowthBase; CopyGrowth(const CopyGrowth&)=default; CopyGrowth& operator=(const CopyGrowth&)=default; };
struct MoveGrowth:GrowthBase { using GrowthBase::GrowthBase; MoveGrowth(const MoveGrowth&)=delete; MoveGrowth(MoveGrowth&&)=default; MoveGrowth& operator=(MoveGrowth&&)=default; };
struct Spec { const char* name;size_t capacity,count,minimum;bool moving;size_t fail;char operation; };
template<class Item> void emit(const char* phase,const Spec& spec,const NCollection_LinearVector<Item>& vector,const Item* before,bool success) {
  std::cout<<phase<<':'<<spec.name<<':'<<(success?"ok":"throw")<<':'<<vector.Size()<<':'<<vector.Capacity()<<':'<<(before==vector.Data())<<':'<<events<<':';
  for(const Item& item:vector)std::cout<<item.value<<'/'<<bool(item.payload)<<',';
  std::cout<<':'<<resources<<'\n';
}
template<class Item> void repair(const Spec& spec) {
  const size_t capacity=std::max(spec.capacity==0?size_t(2):spec.capacity*2,spec.minimum);
  Item* pointer=static_cast<Item*>(firstAttempt);
  std::cout<<"unpublished:"<<spec.name<<':'<<capacity<<':'<<completed<<':';
  for(size_t i=0;i<completed;++i)std::cout<<pointer[i].value<<'/'<<bool(pointer[i].payload)<<',';
  std::cout<<':'<<resources<<'\n';
  for(size_t i=0;i<completed;++i)pointer[i].~Item();
  Standard::Free(pointer);
}
template<class Item> void operation(const Spec& spec,NCollection_LinearVector<Item>& vector) {
  if(spec.operation=='R')vector.Reserve(spec.minimum);
  if(spec.operation=='A')vector.EmplaceAppend(99);
  if(spec.operation=='Z')vector.Resize(spec.minimum);
  // Both types support rvalue assignment; native SetValue uses move for this probe.
  if(spec.operation=='S')vector.SetValue(spec.minimum-1,Item(99));
}
template<class Item> void run(const Spec& spec) {
  NCollection_LinearVector<Item> vector(spec.capacity);
  for(size_t i=0;i<spec.count;++i)vector.EmplaceAppend(int(10+i));
  const Item* before=vector.Data();
  attempts=completed=0;failing=spec.fail;firstAttempt=nullptr;events.clear();active=true;
  bool success=true;
  try {operation(spec,vector);}catch(const std::runtime_error&){success=false;}
  emit("native",spec,vector,before,success);
  if(!success){repair<Item>(spec);emit("repaired",spec,vector,before,false);}
  active=false;
}
int main(){
  const Spec cases[]={
    {"empty-noop",0,0,0,false,0,'R'},
    {"empty-grow",0,0,1,false,0,'R'},
    {"retained-noop",4,3,4,false,0,'R'},
    {"copy-double",4,3,5,false,0,'R'},
    {"copy-minimum",4,3,20,false,0,'R'},
    {"copy-fail-1",4,3,5,false,1,'R'},
    {"copy-fail-2",4,3,5,false,2,'R'},
    {"copy-fail-3",4,3,5,false,3,'R'},
    {"copy-A-fail-1",2,2,3,false,1,'A'},
    {"copy-A-fail-2",2,2,3,false,2,'A'},
    {"copy-Z-fail-1",2,2,5,false,1,'Z'},
    {"copy-Z-fail-2",2,2,5,false,2,'Z'},
    {"copy-S-fail-1",2,2,5,false,1,'S'},
    {"copy-S-fail-2",2,2,5,false,2,'S'},
    {"move-double",4,3,5,true,0,'R'},
    {"move-minimum",4,3,20,true,0,'R'},
    {"move-fail-1",4,3,5,true,1,'R'},
    {"move-fail-2",4,3,5,true,2,'R'},
    {"move-fail-3",4,3,5,true,3,'R'},
    {"move-A-fail-1",2,2,3,true,1,'A'},
    {"move-A-fail-2",2,2,3,true,2,'A'},
    {"move-Z-fail-1",2,2,5,true,1,'Z'},
    {"move-Z-fail-2",2,2,5,true,2,'Z'},
    {"move-S-fail-1",2,2,5,true,1,'S'},
    {"move-S-fail-2",2,2,5,true,2,'S'},
  };
  for(const Spec& spec:cases){if(spec.moving)run<MoveGrowth>(spec);else run<CopyGrowth>(spec);if(resources!=0)return 1;}
}
