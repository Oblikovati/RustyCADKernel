// Real OCCT paired storage; constructor failures retain native leaked-block observations.
#include <NCollection_UBTree.hxx>
#include <cstdlib>
#include <iostream>
#include <map>
#include <stdexcept>
#include <string>
#include <type_traits>
struct TrackingArena;
struct PairTrace {
  static TrackingArena* arena;
  static bool active;
  static char phase,kind;
  static int remaining;
  static std::string events;
  static void event(char p,char k,int value);
  static void reset(char p='X',char k='X',int count=0) {active=true;phase=p;kind=k;remaining=count;events.clear();}
};
struct TrackingArena: NCollection_BaseAllocator {
  std::map<void*,int> blocks;int next=0;std::string releases;
  void* Allocate(size_t size) override {
    void* pointer=nullptr;if(posix_memalign(&pointer,64,size)!=0)throw std::bad_alloc();
    blocks[pointer]=next++;return pointer;
  }
  void* AllocateOptimal(size_t size) override {return Allocate(size);}
  void Free(void* pointer) override {
    if(!pointer)return;releases+="F"+std::to_string(blocks.at(pointer))+",";
    blocks.erase(pointer);std::free(pointer);
  }
  ~TrackingArena() override {for(const auto& entry:blocks)std::free(entry.first);}
};
TrackingArena* PairTrace::arena=nullptr;bool PairTrace::active=false;
char PairTrace::phase='X',PairTrace::kind='X';int PairTrace::remaining=0;std::string PairTrace::events;
void PairTrace::event(char p,char k,int value) {
  if(!active)return;
  events+=std::string{p,k}+std::to_string(value)+":"+std::to_string(arena->blocks.size())+",";
  if(p==phase && k==kind && --remaining==0) {phase='X';throw std::runtime_error("rejected payload callback");}
}
template<char Kind> struct Payload {
  int value;
  Payload():value(0) {PairTrace::event('N',Kind,0);}
  explicit Payload(int item):value(item) {}
  Payload(const Payload& source):value(source.value) {PairTrace::event('C',Kind,value);}
  Payload& operator=(const Payload& source) {value=source.value;PairTrace::event('A',Kind,value);return *this;}
  ~Payload() {PairTrace::event('D',Kind,value);}
  void Add(const Payload&) {} bool IsOut(const Payload&)const {return false;}
  double SquareExtent()const {return 0;}
};
using Object=Payload<'O'>;using Bounds=Payload<'B'>;using Tree=NCollection_UBTree<Object,Bounds>;using Node=Tree::TreeNode;
using GemmateSignature=void(Node::*)(const Bounds&,const Object&,const Bounds&,
  const occ::handle<NCollection_BaseAllocator>&);
static_assert(std::is_same_v<decltype(&Node::Gemmate),GemmateSignature>);
static void populate(Tree& tree,bool branch) {
  tree.Add(Object(1),Bounds(10));if(branch)const_cast<Node&>(tree.Root()).Gemmate(Bounds(30),Object(2),Bounds(20),tree.Allocator());
}
static void construction(bool branch,char phase,char kind,int count) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  Tree tree(arena);populate(tree,branch);auto& root=const_cast<Node&>(tree.Root());
  const Node* previous=branch?&root.Child(0):nullptr;const Bounds unionBound(99),bound(30);const Object object(3);
  PairTrace::reset(phase,kind,count);bool failed=false;
  try {root.Gemmate(unionBound,object,bound,arena);}catch(const std::runtime_error&) {failed=true;}
  PairTrace::active=false;const bool published=!root.IsLeaf() && &root.Child(0)!=previous;
  const bool paired=root.IsLeaf() || &root.Child(1)==&root.Child(0)+1;
  std::cout<<"construct "<<branch<<' '<<phase<<kind<<count<<' '<<PairTrace::events<<' '<<failed<<' '
    <<published<<' '<<root.Bnd().value<<' '<<root.Object().value<<' '<<arena->blocks.size()<<' '<<paired<<'\n';
}
static void destruction(int killed,bool implicit) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  {Tree tree(arena);populate(tree,true);auto& root=const_cast<Node&>(tree.Root());root.Gemmate(Bounds(99),Object(3),Bounds(30),arena);
   PairTrace::reset();arena->releases.clear();if(killed>=0)root.Kill(killed,arena);else if(!implicit)tree.Clear();
   if(!implicit) {PairTrace::active=false;std::cout<<"destroy "<<killed<<' '<<PairTrace::events<<' '<<arena->releases<<' '<<arena->blocks.size()<<'\n';}
  }
  if(implicit) {PairTrace::active=false;std::cout<<"implicit "<<PairTrace::events<<' '<<arena->releases<<' '<<arena->blocks.size()<<'\n';}
}
int main() {
  for(bool branch:{false,true}) {
    construction(branch,'X','X',0);
    for(char kind:{'B','O'})for(int count:{1,2})construction(branch,'N',kind,count);
    for(char kind:{'B','O'})for(int count:{1,2,3})construction(branch,'A',kind,count);
    construction(branch,'N','O',3);
  }
  destruction(-1,false);destruction(0,false);destruction(1,false);destruction(-1,true);
  std::cout<<"layout "<<sizeof(Node)<<' '<<alignof(Node)<<' '<<2*sizeof(Node)<<'\n';
}
