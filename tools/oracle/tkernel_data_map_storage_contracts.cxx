#include <NCollection_DataMap.hxx>
#include <iostream>
#include <string>
struct CollisionHash {
  static std::string calls;
  size_t operator()(int key) const { calls+=std::to_string(key)+",";return 0; }
  bool operator()(int a,int b) const { return a==b; }
};
std::string CollisionHash::calls;
using Map=NCollection_DataMap<int,int,CollisionHash>;
std::string keys(const Map& map) {
  std::string result;
  for(auto it=map.cbegin();it!=map.cend();++it) result+=std::to_string(it.Key())+",";
  return result;
}
void topology() {
  Map map(size_t(5));
  std::cout<<"initial "<<map.Size()<<' '<<map.NbBuckets()<<' '<<(map.Buckets()==nullptr)<<'\n';
  map.Bind(1,10);map.Bind(2,20);map.Bind(3,30);
  const int* retained=&map.Find(2);CollisionHash::calls.clear();
  std::cout<<"head_order "<<keys(map)<<' '<<map.Size()<<' '<<map.NbBuckets()<<'\n';
  map.ReSize(size_t(200));
  std::string rehash_calls=CollisionHash::calls;
  std::cout<<"rehash "<<keys(map)<<' '<<rehash_calls<<' '<<map.NbBuckets()<<' '<<(retained==&map.Find(2))<<'\n';
  auto* table=map.Buckets();CollisionHash::calls.clear();map.ReSize(size_t(0));
  std::cout<<"resize_noop "<<(table==map.Buckets())<<' '<<CollisionHash::calls.size()<<'\n';
  map.Clear(false);CollisionHash::calls.clear();bool found=map.IsBound(2);
  std::cout<<"retained_empty "<<map.Size()<<' '<<map.NbBuckets()<<' '<<(table==map.Buckets())<<' '<<found<<' '<<CollisionHash::calls.size()<<'\n';
  map.Bind(7,70);std::cout<<"reused_insert "<<keys(map)<<' '<<(table==map.Buckets())<<' '<<CollisionHash::calls<<'\n';
}
void copy_assignment() {
  Map source(size_t(1));source.Bind(1,10);source.Bind(2,20);source.Bind(3,30);
  source.ReSize(size_t(200));
  Map copy(source);
  std::cout<<"copy "<<keys(copy)<<' '<<copy.NbBuckets()<<' '<<copy.Size()<<'\n';
  Map target(size_t(2003));target.Bind(9,90);auto* table=target.Buckets();target.Assign(source);
  std::cout<<"assign "<<keys(target)<<' '<<target.NbBuckets()<<' '<<(target.Buckets()==table)<<'\n';
  source.Clear(false);Map empty_copy(source);
  std::cout<<"empty_copy "<<empty_copy.NbBuckets()<<' '<<(empty_copy.Buckets()==nullptr)<<'\n';
  target.Assign(source);
  std::cout<<"empty_assign "<<target.NbBuckets()<<' '<<target.Size()<<' '<<(target.Buckets()==table)<<'\n';
}
void erase_links() {
  Map map;map.Bind(1,10);map.Bind(2,20);map.Bind(3,30);
  map.UnBind(2);std::cout<<"erase_middle "<<keys(map)<<' '<<map.Size()<<'\n';
  map.UnBind(3);std::cout<<"erase_head "<<keys(map)<<' '<<map.Size()<<'\n';
  map.UnBind(1);std::cout<<"erase_tail "<<map.Size()<<' '<<(map.Buckets()!=nullptr)<<'\n';
}
int main(){topology();copy_assignment();erase_links();}
