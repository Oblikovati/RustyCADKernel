#ifndef CPU_HELPER_PROBE_H
#define CPU_HELPER_PROBE_H
#include <stddef.h>
#include <sys/types.h>
struct cpu_helper_probe { pid_t pid; int control; int max_closed_descriptor; };
/* Measure actual FD table space without a limit assumption; cpu_probe_fd_capacity(). */
int cpu_probe_fd_capacity(void);
/* Count open descriptors for leak controls; cpu_probe_fd_count(). */
int cpu_probe_fd_count(void);
/* Inject the late-open regression immediately before fork; cpu_probe_set_fork_hook(open_late). */
void cpu_probe_set_fork_hook(void (*hook)(void));
/* Prepare owned PID and child-side closure before CPU caps; cpu_probe_prepare(&helper,4096). */
int cpu_probe_prepare(struct cpu_helper_probe *, size_t capacity);
/* Transfer the actual current output handle; cpu_probe_send(&helper,STDOUT_FILENO). */
int cpu_probe_send(struct cpu_helper_probe *, int stdout_descriptor);
/* Read the actual output result within a caller guard; cpu_probe_receive(&helper,100). */
int cpu_probe_receive(struct cpu_helper_probe *, int milliseconds);
/* Kill/reap the owned helper and close its private channel; cpu_probe_stop(&helper). */
void cpu_probe_stop(struct cpu_helper_probe *);
/* Read native monotonic nanoseconds; cpu_probe_clock(). */
long long cpu_probe_clock(void);
#define CPU_PROBE_MESSAGE "Process killed by CPU limit  (1 sec)\n"
#endif
