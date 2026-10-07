// Mutation results/failure prefixes; injected extents below are synthetic control-flow evidence.
#include <NCollection_PackedMap.hxx>
#include <NCollection_PackedMapAlgo.hxx>
#include <Standard.hxx>
#include <array>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <vector>
static_assert(sizeof(int)==4 && sizeof(size_t)==8, "pinned Linux integer widths required");
static bool fail_bucket = false;
void* Standard::AllocateOptimal(size_t bytes) {
  if (fail_bucket) throw 17;
  return std::malloc(bytes);
}
template<class T> using Map = NCollection_PackedMap<T>;
template<class T> Map<T> profile(unsigned shape) {
  Map<T> map; const uint64_t collision = Map<T>::BitsPerBlock*101;
  if (shape==1) map.ReSize(size_t(0));
  if (shape==2) for (auto key:std::array<uint64_t,5>{0,collision,32,96,UINT64_MAX}) map.Add(T(key));
  if (shape==3) for (auto key:std::array<uint64_t,4>{collision,0,32,64}) map.Add(T(key));
  if (shape==4) for (auto key:std::array<uint64_t,3>{2,63,collision*2}) map.Add(T(key));
  return map;
}
template<class T> int unary(unsigned op, bool member, Map<T>& map, const Map<T>& other) {
  switch(op) {
    case 0:return member?map.Unite(other):NCollection_PackedMapAlgo::Unite(map,other);
    case 2:return member?map.Intersect(other):NCollection_PackedMapAlgo::Intersect(map,other);
    case 4:return member?map.Subtract(other):NCollection_PackedMapAlgo::Subtract(map,other);
    default:return member?map.Differ(other):NCollection_PackedMapAlgo::Differ(map,other);
  }
}
template<class T> int binary(unsigned op, bool member, Map<T>& map, const Map<T>& left, const Map<T>& right) {
  switch(op) {
    case 1:if(member)map.Union(left,right);else NCollection_PackedMapAlgo::Union(map,left,right);break;
    case 3:if(member)map.Intersection(left,right);else NCollection_PackedMapAlgo::Intersection(map,left,right);break;
    case 5:if(member)map.Subtraction(left,right);else NCollection_PackedMapAlgo::Subtraction(map,left,right);break;
    default:if(member)map.Difference(left,right);else NCollection_PackedMapAlgo::Difference(map,left,right);break;
  }
  return -2;
}
template<class T> void snapshot(int result, const Map<T>& map) {
  std::vector<uint64_t> keys;
  for(T key:map) keys.push_back(uint64_t(typename Map<T>::BlockType(key)));
  std::cout << result << ' ' << map.NbBuckets() << ' ' << map.Size() << ' ' << map.Extent()
    << ' ' << map.IsEmpty() << ' ' << map.myNbPackedMapNodes << ' ' << (map.myData1!=nullptr)
    << ' ' << keys.size();
  for(auto key:keys) std::cout << ' ' << key;
  std::cout << '\n';
}
template<class T> void ordinary(const char* name) {
  const unsigned pairs[][2]={{0,2},{2,0},{2,3},{3,4}};
  for(unsigned op=0;op<8;++op) for(unsigned shape=0;shape<4;++shape) {
    for(unsigned mode=0;mode<4;++mode) {
      auto left=profile<T>(pairs[shape][0]),right=profile<T>(pairs[shape][1]),destination=profile<T>(4);
      int result=-1;fail_bucket=mode/2;
      try { result=op%2?binary(op,mode%2,destination,left,right):unary(op,mode%2,left,right); } catch(int n){if(n!=17)throw;}
      fail_bucket=false;
      std::cout<<name<<":ordinary:"<<op<<':'<<shape<<':'<<mode<<' ';snapshot(result,op%2?destination:left);
    }
  }
}
template<class T> void self_rows(const char* name) {
  for(unsigned op=0;op<8;op+=2) for(unsigned shape=0;shape<3;++shape) for(unsigned mode=0;mode<2;++mode) {
    auto map=profile<T>(shape);
    const int result=unary(op,mode,map,map);
    std::cout<<name<<":self:"<<op<<':'<<shape<<':'<<mode<<' ';snapshot(result,map);
  }
}
constexpr std::array<size_t,9> extents={0,1,2,2147483647,2147483648,4294967295,4294967296,4294967297,SIZE_MAX};
template<class T> void synthetic_intersect(const char* name) {
  for(unsigned a=0;a<extents.size();++a) for(unsigned shape=0;shape<3;++shape) for(unsigned mode=0;mode<2;++mode) {
    Map<T> left;left.Add(T(0));auto right=profile<T>(shape==0?0:shape==1?2:4);
    left.myExtent=extents[a];int result=unary(2,mode,left,right);
    std::cout<<name<<":synthetic_intersect:"<<a<<':'<<shape<<':'<<mode<<' ';snapshot(result,left);
  }
}
template<class T> void synthetic_intersection(const char* name) {
  for(unsigned a=0;a<extents.size();++a) for(unsigned b=0;b<extents.size();++b) for(unsigned mode=0;mode<2;++mode) {
    auto left=profile<T>(2),right=profile<T>(3),destination=profile<T>(4);
    const size_t saved_left=left.Size(),saved_right=right.Size();
    left.myExtent=extents[a];right.myExtent=extents[b];
    const int result=binary(3,mode,destination,left,right);
    left.myExtent=saved_left;right.myExtent=saved_right;
    std::cout<<name<<":synthetic_intersection:"<<a<<':'<<b<<':'<<mode<<' ';snapshot(result,destination);
  }
}
template<class T> std::vector<const void*> node_addresses(const Map<T>& map) {
  std::vector<const void*> result;
  for(size_t bucket=0;bucket<=map.NbBuckets();++bucket)
    for(auto node=map.myData1[bucket];node;node=node->Next())result.push_back(node);
  return result;
}
static bool retains_nodes(const std::vector<const void*>& before,const std::vector<const void*>& after) {
  for(auto node:before) {
    bool found=false;for(auto candidate:after)found|=candidate==node;
    if(!found)return false;
  }
  return true;
}
template<class T> void synthetic_changes(const char* name) {
  constexpr std::array<size_t,7> counts={1,2147483647,2147483648,4294967295,4294967296,4294967297,SIZE_MAX-16};
  for(unsigned op:{0u,4u,6u})for(unsigned a=0;a<counts.size();++a)for(unsigned mode=0;mode<2;++mode) {
    Map<T> map,other;map.Add(T(0));other.Add(T(op==0?33:0));if(op==6)other.Add(T(33));
    map.myExtent=counts[a];const int result=unary(op,mode,map,other);
    std::cout<<name<<":synthetic_changes:"<<op<<':'<<a<<':'<<mode<<' ';snapshot(result,map);
    map.myExtent=0;for(auto key:map){(void)key;++map.myExtent;}
  }
}
template<class T> void growth_failure(const char* name) {
  for(unsigned op: {0u,6u}) for(unsigned mode=0;mode<2;++mode) {
    Map<T> map,other;for(unsigned block=1;block<=101;++block)map.Add(T(block*Map<T>::BitsPerBlock));
    other.Add(T(0));other.Add(T(102*Map<T>::BitsPerBlock));
    const auto before=node_addresses(map);
    int result=-1;fail_bucket=true;
    try {result=unary(op,mode,map,other);}catch(int n){if(n!=17)throw;}
    fail_bucket=false;
    std::cout<<name<<":growth_failure:"<<op<<':'<<mode<<' '<<retains_nodes(before,node_addresses(map))<<' ';snapshot(result,map);
  }
}
template<class T> void width(const char* name) {ordinary<T>(name);self_rows<T>(name);}
int main() {
  width<int32_t>("i32");width<uint32_t>("u32");width<int64_t>("i64");width<uint64_t>("u64");width<int8_t>("i8");width<bool>("bool");
  synthetic_intersect<int64_t>("i64");synthetic_intersect<uint64_t>("u64");
  synthetic_intersection<int64_t>("i64");synthetic_intersection<uint64_t>("u64");
  synthetic_changes<int64_t>("i64");synthetic_changes<uint64_t>("u64");
  growth_failure<int32_t>("i32");growth_failure<uint32_t>("u32");growth_failure<int64_t>("i64");growth_failure<uint64_t>("u64");
}
