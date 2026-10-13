// Actual Incremental Free is a no-op: typed/foreign override failure permits raw Free retry without destruction.
#include <NCollection_IncAllocator.hxx>
#include <Standard_ConstructionError.hxx>
#include <iostream>
#include <stdexcept>
class IncrementalCurrentFreeProbe : public NCollection_IncAllocator {
public:
  int outcome,calls=0;
  bool parentCompleted=false;
  void* expected=nullptr;
  void* received=nullptr;
  explicit IncrementalCurrentFreeProbe(int value):outcome(value) {}
  void Free(void* pointer) override {
    NCollection_IncAllocator::Free(pointer);parentCompleted=true;received=pointer;++calls;
    if(outcome==1)throw Standard_ConstructionError("incremental Free outcome=1; expected successful qualified no-op completion");
    if(outcome==2)throw std::logic_error("incremental Free outcome=2; expected original foreign exception");
  }
};
static int invoke_incremental_free(const occ::handle<NCollection_BaseAllocator>& current,void* pointer) {
  try {current->Free(pointer);return 0;}
  catch(const Standard_ConstructionError& failure) {
    if(std::string(failure.what())!="incremental Free outcome=1; expected successful qualified no-op completion")return 3;
    return 1;
  }
  catch(const std::logic_error&) {return 2;}
}
static void emit_incremental_free(int outcome,int phase,int failure,
    const IncrementalCurrentFreeProbe& current,unsigned char originalByte) {
  bool unchanged=*static_cast<unsigned char*>(current.expected)==originalByte;
  std::cout<<outcome<<' '<<phase<<' '<<failure<<' '<<current.calls<<' '<<current.parentCompleted
    <<' '<<(current.received==current.expected)<<' '<<unchanged<<' '<<current.GetRefCount()<<'\n';
}
static void run_incremental_free(int outcome) {
  occ::handle<NCollection_IncAllocator> original=new NCollection_IncAllocator(4096);
  void* pointer=original->Allocate(sizeof(int));*static_cast<int*>(pointer)=17;
  unsigned char originalByte=*static_cast<unsigned char*>(pointer);
  occ::handle<IncrementalCurrentFreeProbe> current=new IncrementalCurrentFreeProbe(outcome);
  occ::handle<NCollection_BaseAllocator> selected=current;current->expected=pointer;
  int failure=invoke_incremental_free(selected,pointer);emit_incremental_free(outcome,0,failure,*current,originalByte);
  if(outcome!=0) {
    current->outcome=0;failure=invoke_incremental_free(selected,pointer);
    emit_incremental_free(outcome,1,failure,*current,originalByte);
  }
  // No-op Free did not release original storage; all raw accesses end before Reset.
  original->Reset(true);
}
int main() {
  for(int outcome=0;outcome<3;++outcome)run_incremental_free(outcome);
  return 0;
}
