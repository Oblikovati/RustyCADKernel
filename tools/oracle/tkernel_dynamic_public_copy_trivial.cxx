// OCCT 3d097a0: public trivial-copy construction/Assign with empty retained blocks.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
struct Arena : NCollection_BaseAllocator {};
struct IntEngine : NCollection_DynamicArray<int> {
  using NCollection_DynamicArray<int>::NCollection_DynamicArray;
  using NCollection_DynamicArray<int>::blockCount;
  using NCollection_DynamicArray<int>::ensureBlockCount;
};
static void emitRow(int used,int blocks,int mode,IntEngine& target,
                    const occ::handle<NCollection_BaseAllocator>& allocator) {
  std::cout<<used<<','<<blocks<<','<<mode<<" size="<<target.Size()<<" blocks="<<target.blockCount()
           <<" source="<<(target.Allocator()==allocator)<<" values=";
  for(size_t i=0;i<target.Size();++i) { if(i) std::cout<<','; std::cout<<target.Value(i); }
  std::cout<<'\n';
}
static void trivialCase(int used,int blocks) {
  occ::handle<NCollection_BaseAllocator> allocator=new Arena;
  IntEngine source(2,allocator);
  for(int value=1;value<=used;++value) source.Append(value);
  source.ensureBlockCount(size_t(blocks),sizeof(int));
  IntEngine copied(source);emitRow(used,blocks,0,copied,allocator);
  for(int mode=1;mode<=2;++mode) {
    IntEngine target(4);target.Append(99);target.Assign(source,mode==1);
    emitRow(used,blocks,mode,target,allocator);
  }
}
int main() {
  trivialCase(0,0);
  trivialCase(0,3);
  trivialCase(3,3);
}
