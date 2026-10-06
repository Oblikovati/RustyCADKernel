#include "tkernel_ubtree_payload_trace.hxx"
static void node_construction() {
  Object object(7);Bounds bounds(8);Trace::clear();
  Node* node=new Node(object,bounds);watch(*node);delete node;row("node_ctor");
  Node source(object,bounds);Trace::clear();
  Node* copy=new Node(source);watch(*copy);delete copy;row("node_copy");
}
static void pair_construction() {
  Object object(7);Bounds bounds(8);Trace::clear();
  Pair* pair=new Pair(object,bounds);watch(*pair);delete pair;row("pair_ctor");
  Pair source(object,bounds);Trace::clear();
  Pair* copy=new Pair(source);watch(*copy);delete copy;row("pair_copy");
  Pair* empty=new Pair;watch(*empty);delete empty;row("pair_default");
}
static void assignment(bool pair) {
  Object object(7);Bounds bounds(8);Object oldObject(1);Bounds oldBounds(2);
  if (pair) {
    Pair source(object,bounds);Pair target(oldObject,oldBounds);Trace::clear();watch(target);
    target=source;row("pair_assign");
  } else {
    Node source(object,bounds);Node target(oldObject,oldBounds);Trace::clear();watch(target);
    target=source;row("node_assign");
  }
}
static void construction_failure(bool pair,char kind,bool defaults) {
  Object object(7);Bounds bounds(8);Trace::clear();
  Trace::rejectPhase=defaults?'N':'C';Trace::rejectKind=kind;bool failed=false;
  try {
    if (defaults) { Pair value; }
    else if (pair) { Pair value(object,bounds); }
    else { Node value(object,bounds); }
  } catch(const std::runtime_error&) { failed=true; }
  std::cout<<(defaults?"default_fail":pair?"pair_ctor_fail":"node_ctor_fail")<<' '<<kind
           <<' '<<Trace::events<<' '<<failed<<'\n';Trace::clear();
}
template<class Payload> static void assign_failure(Payload& target,const Payload& source,char kind) {
  Trace::clear();watch(target);Trace::rejectPhase='A';Trace::rejectKind=kind;
  bool failed=false;try { target=source; } catch(const std::runtime_error&) { failed=true; }
  std::cout<<kind<<' '<<Trace::events<<' '<<failed<<' '<<Trace::stable<<' ';
  Trace::clear();
}
static void assignment_failure(bool pair,char kind) {
  Object object(7);Bounds bounds(8);Object oldObject(1);Bounds oldBounds(2);
  if(pair) {
    Pair source(object,bounds);Pair target(oldObject,oldBounds);std::cout<<"pair_assign_fail ";
    assign_failure(target,source,kind);std::cout<<target.myObj.value<<' '<<target.myBnd.value<<'\n';
  } else {
    Node source(object,bounds);Node target(oldObject,oldBounds);std::cout<<"node_assign_fail ";
    assign_failure(target,source,kind);std::cout<<target.Object().value<<' '<<target.Bnd().value<<'\n';
  }
}
static void drop_failure(bool pair,char kind) {
  Object object(7);Bounds bounds(8);
  bool failed=false;
  if(pair) {
    Pair* value=new Pair(object,bounds);Trace::clear();watch(*value);
    Trace::rejectPhase='D';Trace::rejectKind=kind;
    try { delete value; } catch(const std::runtime_error&) { failed=true; }
  } else {
    Node* value=new Node(object,bounds);Trace::clear();watch(*value);
    Trace::rejectPhase='D';Trace::rejectKind=kind;
    try { delete value; } catch(const std::runtime_error&) { failed=true; }
  }
  std::cout<<(pair?"pair_drop_fail":"node_drop_fail")<<' '<<kind<<' '<<Trace::events
           <<' '<<failed<<' '<<Trace::stable<<'\n';Trace::clear();
}
static void filler_reset() {
  Tree tree;NCollection_UBTreeFiller<Object,Bounds> filler(tree);
  for(int i=0;i!=3;++i) { Object object(i);Bounds bounds(i);filler.Add(object,bounds); }
  Trace::clear();filler.Reset();row("filler_reset");
}
int main() {
  node_construction();pair_construction();assignment(false);assignment(true);
  for(char kind:{'O','B'}) {
    construction_failure(false,kind,false);construction_failure(true,kind,false);
    construction_failure(true,kind,true);assignment_failure(false,kind);assignment_failure(true,kind);
    drop_failure(false,kind);drop_failure(true,kind);
  }
  filler_reset();
}
