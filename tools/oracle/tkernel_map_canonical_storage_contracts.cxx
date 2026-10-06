#include <NCollection_Map.hxx>
#include <iostream>
#include <string>
struct CollisionHash {
  static std::string calls;
  size_t operator()(int key) const { calls+=std::to_string(key)+",";return 0; }
  bool operator()(int a,int b) const { return a==b; }
};
std::string CollisionHash::calls;
using Map=NCollection_Map<int,CollisionHash>;
std::string keys(const Map& map) {
  std::string result;
  for(auto it=map.cbegin();it!=map.cend();++it) result+=std::to_string(it.Key())+",";
  return result;
}
void topology() {
  Map map(size_t(5));
  std::cout<<"initial "<<map.Size()<<' '<<map.NbBuckets()<<' '<<(map.Buckets()==nullptr)<<'\n';
  map.Add(1);map.Add(2);map.Add(3);
  const int* retained=&map.Added(2);CollisionHash::calls.clear();
  std::cout<<"head_order "<<keys(map)<<' '<<map.Size()<<' '<<map.NbBuckets()<<'\n';
  map.ReSize(size_t(200));
  std::string rehash_calls=CollisionHash::calls;
  std::cout<<"rehash "<<keys(map)<<' '<<rehash_calls<<' '<<map.NbBuckets()<<' '<<(retained==&map.Added(2))<<'\n';
  auto* table=map.Buckets();CollisionHash::calls.clear();map.ReSize(size_t(0));
  std::cout<<"resize_noop "<<(table==map.Buckets())<<' '<<CollisionHash::calls.size()<<'\n';
  map.Clear(false);CollisionHash::calls.clear();bool found=map.Contains(2);
  std::cout<<"retained_empty "<<map.Size()<<' '<<map.NbBuckets()<<' '<<(table==map.Buckets())<<' '<<found<<' '<<CollisionHash::calls.size()<<'\n';
  map.Add(7);std::cout<<"reused_insert "<<keys(map)<<' '<<(table==map.Buckets())<<' '<<CollisionHash::calls<<'\n';
}
void copy_assignment() {
  Map source(size_t(1));source.Add(1);source.Add(2);source.Add(3);
  source.ReSize(size_t(200));
  Map copy(source);
  std::cout<<"copy "<<keys(copy)<<' '<<copy.NbBuckets()<<' '<<copy.Size()<<'\n';
  Map target(size_t(2003));target.Add(9);auto* table=target.Buckets();target.Assign(source);
  std::cout<<"assign "<<keys(target)<<' '<<target.NbBuckets()<<' '<<(target.Buckets()==table)<<'\n';
  source.Clear(false);Map empty_copy(source);
  std::cout<<"empty_copy "<<empty_copy.NbBuckets()<<' '<<(empty_copy.Buckets()==nullptr)<<'\n';
  target.Assign(source);
  std::cout<<"empty_assign "<<target.NbBuckets()<<' '<<target.Size()<<' '<<(target.Buckets()==table)<<'\n';
}
void erase_links() {
  Map map;map.Add(1);map.Add(2);map.Add(3);
  map.Remove(2);std::cout<<"erase_middle "<<keys(map)<<' '<<map.Size()<<'\n';
  map.Remove(3);std::cout<<"erase_head "<<keys(map)<<' '<<map.Size()<<'\n';
  map.Remove(1);std::cout<<"erase_tail "<<map.Size()<<' '<<(map.Buckets()!=nullptr)<<'\n';
}
void actual_nodes() {
  Map map;map.Add(1);map.Add(2);map.Add(3);
  auto* node=static_cast<Map::MapNode*>(map.Buckets()[1]);
  auto cursor=map.begin();auto old=cursor++;
  std::cout<<"actual_node "<<(node==old.Node())<<' '<<(&node->Key()==old.operator->())<<' '<<node->Key()<<'\n';
  std::cout<<"key_iterator "<<*old<<' '<<old.Key()<<' '<<*cursor<<' '<<(old.operator->()==&old.Key())<<'\n';
}
int main(){topology();copy_assignment();erase_links();actual_nodes();}
