// OCCT 3d097a0: copy constructor partial-node retention and real allocator handle callbacks.
#include "ordered_copy_lifetimes/fixture.hxx"
#include "ordered_copy_lifetimes/defaults.hxx"
int main(){
 runCopies<CopySet>("set",2);runCopies<CopyMap>("data",4);
 reclaimCopy<CopySet,CopySet::OrderedMapNode>("set");
 reclaimCopy<CopyMap,CopyMap::OrderedDataMapNode>("data");
 resetCopy<CopySet>("set");resetCopy<CopyMap>("data");
 defaultSuite<NCollection_OrderedMap<int,DefaultPolicy>>("set");
 defaultSuite<NCollection_OrderedDataMap<int,int,DefaultPolicy>>("data");
}
