#include <execinfo.h>
#include <signal.h>
#include <unistd.h>
void crashTrace(int signal){void* frames[40];int n=backtrace(frames,40);backtrace_symbols_fd(frames,n,2);_exit(128+signal);}
__attribute__((constructor)) void installTrace(){signal(SIGSEGV,crashTrace);}
