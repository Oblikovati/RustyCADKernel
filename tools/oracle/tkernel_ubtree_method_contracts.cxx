// UBTree and UBTreeFiller concrete callback contracts against pinned OCCT.
#include <NCollection_UBTree.hxx>
#include <NCollection_UBTreeFiller.hxx>
#include <iostream>
#include <vector>
#include <string>
#include <utility>
struct Bounds {
  double low=0,high=0;
  static std::vector<double> checks;
  Bounds()=default;
  Bounds(double a,double b):low(a),high(b) {}
  void Add(const Bounds& b) { low=std::min(low,b.low); high=std::max(high,b.high); }
  bool IsOut(const Bounds& b) const { checks.push_back(low); return high<b.low || b.high<low; }
  double SquareExtent() const { return (high-low)*(high-low); }
};
std::vector<double> Bounds::checks;
struct Tree : NCollection_UBTree<int,Bounds> {
  TreeNode& LastNode() { return ChangeLastNode(); }
  int Branch(int index,Selector& selector) const { return Select(Root().Child(index),selector); }
  using NCollection_UBTree<int,Bounds>::Select;
};
struct Collect : Tree::Selector {
  std::vector<int> values;
  explicit Collect(bool stopping=false) { myStop=stopping; }
  bool Reject(const Bounds&) const override { return false; }
  bool Accept(const int& object) override { values.push_back(object); return true; }
};
void traversal() {
  Tree tree; tree.Add(1,Bounds(0,10)); tree.Add(2,Bounds(1,2));
  Bounds::checks.clear(); tree.Add(3,Bounds(1.5,1.6));
  std::cout<<"is_out"; for(double value:Bounds::checks) std::cout<<' '<<value;
  std::cout<<'\n';
  Collect branch; const int count=tree.Branch(1,branch);
  std::cout<<"branch "<<count; for(int value:branch.values) std::cout<<' '<<value;
  std::cout<<'\n';
  Collect all; const int total=tree.Select(all);
  std::cout<<"all "<<total; for(int value:all.values) std::cout<<' '<<value;
  std::cout<<'\n';
  Collect stopped(true); std::cout<<"stop "<<tree.Select(stopped)<<' '<<stopped.values.front()<<'\n';
  tree.LastNode().ChangeBnd().high=2.5;
  const Tree::TreeNode* last=&tree.LastNode();
  Tree target; const bool returned=&(target=std::move(tree))==&target;
  const bool retained=&target.LastNode()==last;
  const bool self=&(target=std::move(target))==&target;
  std::cout<<"move "<<returned<<' '<<retained<<' '<<tree.IsEmpty()<<' '<<self<<' '
    <<target.LastNode().Object()<<' '<<target.LastNode().Bnd().high<<'\n';
}
struct TraceObject {
  int value=0;
  static std::string events;
  TraceObject()=default;
  explicit TraceObject(int n):value(n) {}
  TraceObject(const TraceObject& other):value(other.value) { events+="object-copy,"; }
  TraceObject(TraceObject&& other):value(other.value) {}
  TraceObject& operator=(const TraceObject& other) {
    if(this!=&other) { events+="object-assign,"; value=other.value; }
    return *this;
  }
};
std::string TraceObject::events;
struct TraceBounds {
  TraceBounds()=default;
  TraceBounds(const TraceBounds&) { TraceObject::events+="bounds-copy,"; }
  TraceBounds(TraceBounds&&) {}
  TraceBounds& operator=(const TraceBounds& other) {
    if(this!=&other) TraceObject::events+="bounds-assign,";
    return *this;
  }
  void Add(const TraceBounds&) {}
  bool IsOut(const TraceBounds&) const { return false; }
  double SquareExtent() const { return 0; }
};
struct RecordingTree : NCollection_UBTree<TraceObject,TraceBounds> {
  std::vector<int> visits;
  bool Add(const TraceObject& object,const TraceBounds&) override { visits.push_back(object.value); return true; }
};
void pairCopy() {
  using Pair=NCollection_UBTreeFiller<TraceObject,TraceBounds>::ObjBnd;
  TraceObject object(1); TraceBounds bounds;
  TraceObject::events.clear(); Pair first(object,bounds);
  std::cout<<"pair "<<TraceObject::events<<'\n';
  TraceObject::events.clear(); Pair copied(first);
  std::cout<<"pair_copy "<<TraceObject::events<<'\n';
  TraceObject::events.clear(); copied=first;
  std::cout<<"pair_assign "<<TraceObject::events<<'\n';
}
void filler() {
  RecordingTree recording;
  {
    NCollection_UBTreeFiller<TraceObject,TraceBounds> fill(recording);
    TraceBounds bounds;
    for(int value=1;value<=3;++value) { TraceObject object(value); fill.Add(object,bounds); }
    TraceObject::events.clear(); const int count=fill.Fill();
    std::cout<<"fill "<<count<<' '<<TraceObject::events<<' '<<fill.Fill()<<'\n';
  }
  std::cout<<"visits"; for(int value:recording.visits) std::cout<<' '<<value;
  std::cout<<'\n';
}
int main() { traversal(); pairCopy(); filler(); }
