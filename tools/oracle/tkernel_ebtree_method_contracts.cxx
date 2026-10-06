// Native EBTree leaf-index updates must not copy keys while rebinding pointers.
#include <NCollection_EBTree.hxx>
#include <NCollection_UBTreeFiller.hxx>
#include <NCollection_IncAllocator.hxx>
#include <algorithm>
#include <functional>
#include <iostream>
#include <vector>
#include <utility>
struct Key {
  int value=0;
  static std::vector<int> copies,hashes;
  Key()=default;
  explicit Key(int n):value(n) {}
  Key(const Key& other):value(other.value) { if(value) copies.push_back(value); }
  Key& operator=(const Key& other) { value=other.value; if(value && this!=&other) copies.push_back(value); return *this; }
  bool operator==(const Key& other) const { return value==other.value; }
};
std::vector<int> Key::copies,Key::hashes;
namespace std { template<> struct hash<Key> { size_t operator()(const Key& key) const {
  Key::hashes.push_back(key.value); return key.value;
} }; }
struct Bounds {
  double low=0,high=0;
  Bounds()=default;
  Bounds(double a,double b):low(a),high(b) {}
  void Add(const Bounds& b) { low=std::min(low,b.low); high=std::max(high,b.high); }
  bool IsOut(const Bounds& b) const { return high<b.low || b.high<low; }
  double SquareExtent() const { return (high-low)*(high-low); }
};
void rebind() {
  NCollection_EBTree<Key,Bounds> tree; Key first(1),second(2);
  Key::copies.clear(); tree.Add(first,Bounds(0,1));
  std::cout<<"first_copies "<<Key::copies.size()<<'\n';
  Key::copies.clear(); tree.Add(second,Bounds(2,3));
  std::cout<<"neighbor_copies "<<std::count(Key::copies.begin(),Key::copies.end(),1)<<' '
    <<std::count(Key::copies.begin(),Key::copies.end(),2)<<'\n';
  const auto* root=&tree.Root(); Key::copies.clear(); const bool removed=tree.Remove(second);
  std::cout<<"survivor "<<removed<<' '<<Key::copies.size()<<' '<<Key::copies.front()<<' '
    <<(&tree.FindNode(first)==root)<<' '<<tree.FindNode(first).IsRoot()<<'\n';
}
void singleton() {
  NCollection_EBTree<Key,Bounds> tree; Key object(1); tree.Add(object,Bounds(0,1));
  Key::hashes.clear(); Key::copies.clear(); const bool removed=tree.Remove(object);
  std::cout<<"singleton "<<removed<<' '<<Key::hashes.size()<<' '<<Key::hashes.front()<<' '
    <<Key::copies.size()<<' '<<tree.IsEmpty()<<'\n';
}
void duplicate() {
  NCollection_EBTree<Key,Bounds> tree; Key object(1); tree.Add(object,Bounds(0,1));
  Key::copies.clear(); Key::hashes.clear(); const bool added=tree.Add(object,Bounds(10,20));
  const size_t copies=Key::copies.size(),hashes=Key::hashes.size();
  std::cout<<"duplicate "<<added<<' '<<copies<<' '<<hashes<<' '<<tree.FindNode(object).Bnd().high<<'\n';
  NCollection_UBTreeFiller<Key,Bounds> filler(tree); filler.Add(object,Bounds(10,20));
  Key::copies.clear(); const int filled=filler.Fill();
  std::cout<<"filler_duplicate "<<filled<<' '<<Key::copies.size()<<' '<<tree.FindNode(object).Bnd().high<<'\n';
}
struct Collect : NCollection_UBTree<int,Bounds>::Selector {
  std::vector<int> objects;
  bool Reject(const Bounds&) const override { return false; }
  bool Accept(const int& object) override { objects.push_back(object); return true; }
};
void moving() {
  occ::handle<NCollection_BaseAllocator> arena=new NCollection_IncAllocator(4096);
  NCollection_EBTree<int,Bounds> source(arena),target;
  source.Add(1,Bounds(0,1)); source.Add(2,Bounds(2,3)); target.Add(9,Bounds(9,10));
  const auto* first=&source.FindNode(1);
  const bool returned=&(target=std::move(source))==&target;
  const bool self=&(target=std::move(target))==&target;
  std::cout<<"move "<<returned<<' '<<self<<' '<<(&target.FindNode(1)==first)<<' '
    <<(target.Allocator()==arena)<<' '<<source.IsEmpty()<<' '<<source.Contains(1)<<' '<<target.Contains(9)<<'\n';
  Collect selected; std::cout<<"select "<<target.Select(selected);
  for(int object:selected.objects) std::cout<<' '<<object;
  std::cout<<'\n';
}
int main() { rebind(); singleton(); duplicate(); moving(); }
