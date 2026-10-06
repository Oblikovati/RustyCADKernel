// Compile with -fno-access-control to invoke the exact private TreeNode() body.
// This changes access checking only; it does not make the method part of OCCT's public API.
#include "tkernel_ubtree_payload_trace.hxx"
static void default_construction() {
  Trace::clear();Node* node=new Node;
  const bool leaf=node->IsLeaf(),root=node->IsRoot();
  const int object=node->Object().value,bounds=node->Bnd().value;
  watch(*node);delete node;
  std::cout<<"node_default "<<Trace::events<<' '<<Trace::stable<<' '<<leaf<<' '<<root<<' '
           <<object<<' '<<bounds<<'\n';Trace::clear();
}
static void default_failure(char kind) {
  Trace::clear();Trace::rejectPhase='N';Trace::rejectKind=kind;bool failed=false;
  try { Node node; } catch(const std::runtime_error&) { failed=true; }
  std::cout<<"node_default_fail "<<kind<<' '<<Trace::events<<' '<<failed<<'\n';Trace::clear();
}
static void default_drop(char kind) {
  Node* node=new Node;Trace::clear();watch(*node);
  Trace::rejectPhase='D';Trace::rejectKind=kind;bool failed=false;
  try {delete node;}catch(const std::runtime_error&) {failed=true;}
  std::cout<<"node_default_drop "<<kind<<' '<<Trace::events<<' '<<failed<<' '<<Trace::stable<<'\n';Trace::clear();
}
int main() { default_construction();for(char kind:{'B','O'}) {default_failure(kind);default_drop(kind);} }
