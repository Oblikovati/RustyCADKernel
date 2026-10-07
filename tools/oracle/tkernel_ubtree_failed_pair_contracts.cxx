// Unpublished pairs remain raw: explicit fixture repair destroys only completed nodes.
#include "tkernel_ubtree_allocation_trace.hxx"
static void populate(Tree& tree,bool branch) {
  tree.Add(Object(1),Bounds(10));
  if(branch)const_cast<Node&>(tree.Root()).Gemmate(Bounds(30),Object(2),Bounds(20),tree.Allocator());
}
static Node* failed_pair(const TrackingArena& arena,int ordinal) {
  for(const auto& entry:arena.blocks)if(entry.second==ordinal)return static_cast<Node*>(entry.first);
  throw std::runtime_error("missing failed pair allocation; expected original raw block");
}
static int initialized(char phase,int ordinal) {
  return phase=='N'?ordinal-1:2;
}
static void retained(bool branch,char phase,char kind,int ordinal) {
  occ::handle<TrackingArena> arena=new TrackingArena;PairTrace::arena=arena.get();PairTrace::active=false;
  Tree tree(arena);populate(tree,branch);const int allocation=arena->next;
  const Bounds unionBound(99),bound(30);const Object object(3);
  PairTrace::reset(phase,kind,ordinal);bool rejected=false;
  try {const_cast<Node&>(tree.Root()).Gemmate(unionBound,object,bound,arena);}
  catch(const std::runtime_error&) {rejected=true;}
  PairTrace::active=false;const auto failed=PairTrace::events;const auto outstanding=arena->blocks.size();
  Node* pair=failed_pair(*arena,allocation);PairTrace::reset();tree.Clear();
  PairTrace::active=false;const auto cleared=PairTrace::events;const auto remaining=arena->blocks.size();
  PairTrace::reset();for(int slot=0;slot<initialized(phase,ordinal);++slot)pair[slot].~Node();arena->Free(pair);
  PairTrace::active=false;
  std::cout<<"retained "<<branch<<' '<<phase<<kind<<ordinal<<' '<<rejected<<' '<<failed<<' '<<outstanding
    <<' '<<cleared<<' '<<remaining<<' '<<(PairTrace::events.empty()?"-":PairTrace::events)
    <<' '<<arena->blocks.size()<<'\n';
}
int main() {
  for(bool branch:{false,true}) {
    for(char kind:{'B','O'})for(int ordinal:{1,2})retained(branch,'N',kind,ordinal);
    for(char kind:{'B','O'})for(int ordinal:{1,2})retained(branch,'A',kind,ordinal);
  }
}
