// Isolated noexcept destructor failures: observe the defined termination prefix, never resume it.
#include <exception>
#include <cstdlib>
#include <sys/wait.h>
#include <unistd.h>
struct TerminationState {std::string label;const NCollection_BaseMap* map;ClearTrace* trace;const void* buckets;};
TerminationState* termination=nullptr;
[[noreturn]]void recordTermination() {
 const auto& state=*termination;
 snapshot(state.label,*state.map,*state.trace,state.map->Buckets()==state.buckets);
 std::cout.flush();std::_Exit(73);
}
template<class Map>[[noreturn]]void terminateChild(const std::string& label,int mode,char rejected) {
 occ::handle<NCollection_BaseAllocator> old=new ClearArena,next=new ClearArena;
 ClearTrace trace{old.get(),next.get(),0,false};Map map(ClearPolicy(),size_t(13),old);populate(map,trace);
 TerminationState state{label,&map,&trace,map.Buckets()};termination=&state;std::set_terminate(recordTermination);
 trace.active=true;trace.reject=rejected;invokeClear(map,mode,old,next);std::_Exit(74);
}
template<class Map>void terminationCase(const char* family,int mode,char rejected) {
 std::string label=std::string(family)+"_terminate_"+clearModes[mode]+"_"+rejected;
 std::cout.flush();std::cerr.flush();pid_t child=fork();if(child<0)std::exit(75);
 if(child==0)terminateChild<Map>(label,mode,rejected);
 int status=0;if(waitpid(child,&status,0)!=child||!WIFEXITED(status)||WEXITSTATUS(status)!=73)std::exit(76);
 std::cout<<label<<"_exit "<<WEXITSTATUS(status)<<'\n';
}
