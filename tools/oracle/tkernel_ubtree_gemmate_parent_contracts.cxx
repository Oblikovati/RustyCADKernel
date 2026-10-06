// Actual Gemmate parent publication/assignment phases; child allocation remains separate.
#include <NCollection_UBTree.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <cassert>
#include <stdexcept>
#include <string>
struct ParentTrace {
  static const void *object,*bounds;
  static std::string events;
  static char rejected;
  static bool active;
  static void event(char phase,char kind,int value,const void* address) {
    if(phase=='A' && address==bounds)active=true;
    if(!active || (kind=='B' && address!=bounds))return;
    if(phase=='A' && kind=='O')assert(address==object);
    events+=std::string{phase,kind}+std::to_string(value)+",";
    if((rejected=='B' && phase=='A' && kind=='B') || (rejected=='N' && phase=='N') || (rejected=='O' && phase=='A' && kind=='O')) {
      rejected=0;throw std::runtime_error("rejected parent phase");
    }
  }
  static void reset(const void* parentObject,const void* parentBounds,char failure) {
    object=parentObject;bounds=parentBounds;events.clear();rejected=failure;active=false;
  }
};
const void *ParentTrace::object=nullptr,*ParentTrace::bounds=nullptr;
std::string ParentTrace::events;char ParentTrace::rejected=0;bool ParentTrace::active=false;
template<char Kind> struct Payload {
  int value;
  Payload():value(0) {ParentTrace::event('N',Kind,value,this);}
  explicit Payload(int item):value(item) {}
  Payload(const Payload& source):value(source.value) {ParentTrace::event('C',Kind,value,this);}
  Payload& operator=(const Payload& source) {value=source.value;ParentTrace::event('A',Kind,value,this);return *this;}
  ~Payload() {ParentTrace::event('D',Kind,value,this);}
  void Add(const Payload&) {} bool IsOut(const Payload&)const {return false;}
  double SquareExtent()const {return 0;}
};
using Object=Payload<'O'>;using Bounds=Payload<'B'>;using Tree=NCollection_UBTree<Object,Bounds>;
static void gemmate_case(bool branch,char failure) {
  Tree tree(new NCollection_IncAllocator(1024));const Object first(1),second(2),third(3);const Bounds one(10),two(20),three(30),unionBounds(99);
  tree.Add(first,one);auto& root=const_cast<Tree::TreeNode&>(tree.Root());
  if(branch)root.Gemmate(three,second,two,tree.Allocator());
  ParentTrace::reset(&root.Object(),&root.Bnd(),failure);bool failed=false;
  try {root.Gemmate(unionBounds,third,three,tree.Allocator());}catch(const std::runtime_error&) {failed=true;}
  ParentTrace::active=false;
  const bool parents=&root.Child(0).Parent()==&root && &root.Child(1).Parent()==&root;
  const bool grandchildren=!branch || (&root.Child(0).Child(0).Parent()==&root.Child(0) && &root.Child(0).Child(1).Parent()==&root.Child(0));
  std::cout<<"parent "<<branch<<' '<<failure<<' '<<ParentTrace::events<<' '<<failed<<' '
    <<root.Bnd().value<<' '<<root.Object().value<<' '<<root.Child(0).Object().value<<' '
    <<root.Child(1).Object().value<<' '<<parents<<' '<<grandchildren<<'\n';
}
int main() {for(bool branch:{false,true})for(char failure:{'X','B','N','O'})gemmate_case(branch,failure);}
