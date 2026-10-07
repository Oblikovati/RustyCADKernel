#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_AccAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct BorrowEmplaceArena:NCollection_AccAllocator {
 BorrowEmplaceArena():NCollection_AccAllocator(16384){}
 int Active()const{int count=0;for(auto it=myBlocks.cbegin();it!=myBlocks.cend();++it)count+=it->allocCount;return count;}
};
struct BorrowEmplaceTrace {
 BorrowEmplaceArena* arena;std::string events;bool active=false,failValue=false,failKey=false,failHash=false,failEqual=false;
 void note(const std::string& event){if(active)events+=event+"a"+std::to_string(arena->Active())+",";}
 int construct(int id){note("V"+std::to_string(id));if(failValue)throw std::runtime_error("value constructor");return id;}
};
struct BorrowEmplaceKey {
 int id,tag;BorrowEmplaceTrace* trace;
 BorrowEmplaceKey(int value,BorrowEmplaceTrace& log):id(value),tag(10),trace(&log){}
 BorrowEmplaceKey(const BorrowEmplaceKey& other):id(other.id),tag(other.tag),trace(other.trace){trace->note("K"+std::to_string(id));if(trace->failKey)throw std::runtime_error("key copy");}
 BorrowEmplaceKey(BorrowEmplaceKey&& other):id(other.id),tag(other.tag),trace(other.trace){}
 ~BorrowEmplaceKey(){trace->note("D"+std::to_string(id));}
};
struct BorrowEmplaceArgument {
 int id;BorrowEmplaceTrace* trace;
 operator int()const{return trace->construct(id);}
};
struct BorrowEmplaceValue {
 int id;BorrowEmplaceTrace* trace;
 explicit BorrowEmplaceValue(const BorrowEmplaceArgument& arg):id(arg.trace->construct(arg.id)),trace(arg.trace){}
 BorrowEmplaceValue(BorrowEmplaceValue&& other):id(other.id),trace(other.trace){}
 BorrowEmplaceValue& operator=(BorrowEmplaceValue&& other){id=other.id;return *this;}
 ~BorrowEmplaceValue(){trace->note("W"+std::to_string(id));}
};
struct BorrowEmplacePolicy {
 size_t operator()(const BorrowEmplaceKey& key)const{key.trace->note("H"+std::to_string(key.id));if(key.trace->failHash&&key.id==999)throw std::runtime_error("hash");return size_t(key.id);}
 bool operator()(const BorrowEmplaceKey& left,const BorrowEmplaceKey& right)const{left.trace->note("E"+std::to_string(left.id)+":"+std::to_string(right.id));if(left.trace->failEqual)throw std::runtime_error("equal");return left.id==right.id;}
};
using ScalarMap=NCollection_OrderedDataMap<BorrowEmplaceKey,int,BorrowEmplacePolicy>;
using TrackedMap=NCollection_OrderedDataMap<BorrowEmplaceKey,BorrowEmplaceValue,BorrowEmplacePolicy>;
int valueId(int value){return value;}
int valueId(const BorrowEmplaceValue& value){return value.id;}
const char* borrowMethods[]={"emplace","emplaced","try_emplace","try_emplaced"};
template<class Map>std::string invokeBorrowed(Map& map,int method,const BorrowEmplaceKey& key,const BorrowEmplaceArgument& arg) {
 try{switch(method){
  case 0:return "b"+std::to_string(map.Emplace(key,arg));
  case 1:return "r"+std::to_string(valueId(map.Emplaced(key,arg)));
  case 2:return "b"+std::to_string(map.TryEmplace(key,arg));
  default:return "r"+std::to_string(valueId(map.TryEmplaced(key,arg)));
 }}catch(const std::runtime_error&){return "throw";}
}
struct BorrowEmplaceCase {const char* label;int count,key;bool value,copy,hash,equal;};
BorrowEmplaceCase borrowCases[]={
 {"empty",0,7,false,false,false,false},{"empty_value",0,7,true,false,false,false},{"empty_copy",0,7,false,true,false,false},
 {"insert",3,999,false,false,false,false},{"duplicate",3,7,false,false,false,false},{"duplicate_copy",3,7,false,true,false,false},
 {"duplicate_value",3,7,true,false,false,false},{"hash",3,999,false,false,true,false},{"equal",3,7,false,false,false,true},
 {"grow",102,999,false,false,false,false},{"grow_value",102,999,true,false,false,false},{"grow_copy",102,999,false,true,false,false}
};
void fillBorrowed(ScalarMap& map,BorrowEmplaceTrace& trace,int count) {
 if(count==3){for(int key:{4,1,7})map.Bind(BorrowEmplaceKey(key,trace),key*10);return;}
 for(int key=0;key<count;++key)map.Bind(BorrowEmplaceKey(key,trace),key*10);
}
void fillBorrowed(TrackedMap& map,BorrowEmplaceTrace& trace,int count) {
 if(count)map.Emplace(BorrowEmplaceKey(7,trace),BorrowEmplaceArgument{70,&trace});
}
template<class Map>std::string borrowOrder(const Map& map) {
 std::string result;for(const auto& item:map.Items())result+=std::to_string(item.Key.id)+"/"+std::to_string(item.Key.tag)+":"+std::to_string(valueId(item.Value))+",";
 return result.empty()?"-":result;
}
template<class Map>void borrowedCase(const char* type,int method,const BorrowEmplaceCase& test) {
 occ::handle<BorrowEmplaceArena> arena=new BorrowEmplaceArena;BorrowEmplaceTrace trace{arena.get()};Map map(BorrowEmplacePolicy(),size_t(0),arena);
 fillBorrowed(map,trace,test.count);BorrowEmplaceKey key(test.key,trace);key.tag=20;const auto* before=map.Seek(key);const auto* first=map.FirstNode();
 trace.active=true;trace.failValue=test.value;trace.failKey=test.copy;trace.failHash=test.hash;trace.failEqual=test.equal;
 std::string result=invokeBorrowed(map,method,key,BorrowEmplaceArgument{20,&trace});trace.active=false;trace.failHash=false;trace.failEqual=false;
 const auto* after=map.Seek(key);std::string label=std::string(type)+"_"+borrowMethods[method]+"_"+test.label;
 std::cout<<label<<' '<<result<<' '<<map.Size()<<' '<<map.NbBuckets()<<' '<<(map.Buckets()!=nullptr)<<' '<<arena->Active()<<' '<<(first==map.FirstNode())<<' '<<(!before||before==after)<<' '<<key.id<<'/'<<key.tag<<' '<<(after?valueId(*after):-1)<<' '<<trace.events<<' '<<borrowOrder(map)<<'\n';
}
