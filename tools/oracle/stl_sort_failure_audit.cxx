// Regenerates tests/stl_algorithms.rs failure-prefix hashes with the native libstdc++.
// Every possible throwing comparison is checked, including the adversarial heap fallback.
#include <algorithm>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <vector>
struct AuditSortFailure {};
struct AuditAdversary {
  std::vector<int> values;int gas,solid=0,candidate=0;
  explicit AuditAdversary(int count):values(count,count),gas(count){}
  void freeze(int index){values[index]=solid++;}
  bool less(int first,int second){
    if(values[first]==gas && values[second]==gas)freeze(first==candidate?first:second);
    if(values[first]==gas)candidate=first;else if(values[second]==gas)candidate=second;
    return values[first]<values[second];
  }
};
struct AuditSortResult {int comparisons=0;bool failed=false;std::vector<int> values;};
AuditSortResult audit_sort_trial(int count,int failAt,bool adversarial){
  AuditSortResult result;result.values.resize(count);std::iota(result.values.begin(),result.values.end(),0);
  AuditAdversary adversary(count);
  try {std::sort(result.values.begin(),result.values.end(),[&](int first,int second){
    if(++result.comparisons==failAt)throw AuditSortFailure();
    return adversarial?adversary.less(first,second):((first*7+3)%5)<((second*7+3)%5);
  });}catch(const AuditSortFailure&){result.failed=true;}
  return result;
}
void audit_sort_case(int count,bool adversarial){
  const int comparisons=audit_sort_trial(count,0,adversarial).comparisons;
  std::uint64_t hash=1469598103934665603ULL;
  auto mix=[&](int value){hash=(hash^std::uint64_t(value))*1099511628211ULL;};
  for(int failAt=1;failAt<=comparisons+1;++failAt){
    const auto result=audit_sort_trial(count,failAt,adversarial);
    mix(result.comparisons);mix(result.failed?1:0);for(int value:result.values)mix(value);
  }
  std::cout<<count<<' '<<adversarial<<' '<<comparisons<<' '<<hash<<'\n';
}
int main(){for(int count:{0,1,2,16,17,40,257})audit_sort_case(count,false);audit_sort_case(300,true);}
