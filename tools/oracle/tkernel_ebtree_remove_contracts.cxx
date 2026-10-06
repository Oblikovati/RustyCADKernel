// OCCT 3d097a0328 Remove: retained ancestor assignments and prior index rebinding.
#include <NCollection_EBTree.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
struct Bounds;
using Editable=NCollection_EBTree<int,Bounds>;
struct RemoveTrace {
  static std::string events;
  static char reject;
  static int countdown;
  static std::vector<const void*> assignments,additions;
  static const Editable* tree;
  static const void* absorbed;
  static bool rebound;
  static void event(char phase,int value,const void* address);
  static void reset(char phase,int count) {
    events.clear();assignments.clear();additions.clear();reject=phase;countdown=count;rebound=true;
  }
};
struct Bounds {
  int value=0;
  Bounds()=default;explicit Bounds(int bits):value(bits) {}
  Bounds(const Bounds& source):value(source.value) {RemoveTrace::event('C',value,this);}
  Bounds& operator=(const Bounds& source) {value=source.value;RemoveTrace::event('A',value,this);return *this;}
  ~Bounds() {RemoveTrace::event('D',value,this);}
  void Add(const Bounds& other) {value|=other.value;RemoveTrace::event('G',other.value,this);}
  bool IsOut(const Bounds&)const {return false;}
  double SquareExtent()const {return 0;}
};
std::string RemoveTrace::events;char RemoveTrace::reject=0;int RemoveTrace::countdown=0;
std::vector<const void*> RemoveTrace::assignments,RemoveTrace::additions;
const Editable* RemoveTrace::tree=nullptr;const void* RemoveTrace::absorbed=nullptr;bool RemoveTrace::rebound=true;
void RemoveTrace::event(char phase,int value,const void* address) {
  events+=std::string(1,phase)+std::to_string(value)+",";
  if(phase=='A')assignments.push_back(address);
  if(phase=='G')additions.push_back(address);
  if(tree && (phase=='A'||phase=='G') && address!=absorbed)
    rebound &= &tree->FindNode(4).Bnd()==absorbed;
  if(phase==reject && --countdown==0) {reject=0;throw std::runtime_error("rejected bounds callback");}
}
static void populate(Editable& tree,int count) {
  for(int value=1;value<=count;++value)tree.Add(value,Bounds(1<<value));
}
static bool prefix(const std::vector<const void*>& actual,const std::vector<const void*>& expected) {
  return actual.size()<=expected.size() && std::equal(actual.begin(),actual.end(),expected.begin());
}
static void remove_case(char phase,int count) {
  Editable tree;populate(tree,4);const auto& parent=tree.FindNode(1).Parent();
  const std::vector<const void*> expected{&parent.Bnd(),&parent.Parent().Bnd(),&tree.Root().Bnd()};
  const std::vector<const void*> additions{expected[1],expected[2]};
  RemoveTrace::reset(phase,count);RemoveTrace::tree=&tree;RemoveTrace::absorbed=expected[0];
  bool failed=false;try {tree.Remove(1);}catch(const std::runtime_error&) {failed=true;}
  RemoveTrace::tree=nullptr;const bool bound=&tree.FindNode(4)==&parent;
  std::cout<<"remove "<<phase<<count<<' '<<RemoveTrace::events<<' '<<failed<<' '<<bound<<' '
    <<RemoveTrace::rebound<<' '<<prefix(RemoveTrace::assignments,expected)<<' '
    <<prefix(RemoveTrace::additions,additions)<<' '<<!tree.Contains(1)<<' '<<tree.Root().Bnd().value<<'\n';
  RemoveTrace::reset(0,0);
}
static void edge_case(int count,int removed) {
  Editable tree;populate(tree,count);RemoveTrace::reset(0,0);bool result=tree.Remove(removed);
  std::cout<<"edge "<<count<<' '<<removed<<' '<<result<<' '<<tree.IsEmpty()<<' '
    <<(RemoveTrace::events.empty()?"empty":RemoveTrace::events)<<'\n';RemoveTrace::reset(0,0);
}
int main() {
  remove_case('N',0);for(int count:{1,2,3})remove_case('A',count);
  for(int count:{1,2})remove_case('G',count);
  edge_case(0,1);edge_case(1,1);edge_case(4,99);edge_case(2,2);edge_case(4,2);
}
