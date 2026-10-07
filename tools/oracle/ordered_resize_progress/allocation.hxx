// Forward to the actual pinned Standard backend; interception observes ownership only.
#include <Standard.hxx>
#include <dlfcn.h>
#include <cstdlib>
struct ResizeAllocation {bool active=false;void* replacement=nullptr;size_t bytes=0;int allocated=0,freed=0;};
ResizeAllocation allocation;
void* Standard::Allocate(const size_t bytes) {
 using Function=void*(*)(size_t);
 static Function original=reinterpret_cast<Function>(dlsym(RTLD_NEXT,"_ZN8Standard8AllocateEm"));
 if(!original)std::abort();void* result=original(bytes);
 if(allocation.active){allocation.replacement=result;allocation.bytes=bytes;++allocation.allocated;}
 return result;
}
void Standard::Free(void* const pointer) {
 using Function=void(*)(void*);
 static Function original=reinterpret_cast<Function>(dlsym(RTLD_NEXT,"_ZN8Standard4FreeEPv"));
 if(!original)std::abort();if(pointer&&pointer==allocation.replacement)++allocation.freed;
 original(pointer);
}
