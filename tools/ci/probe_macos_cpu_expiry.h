#ifndef CPU_EXPIRY_PROBE_H
#define CPU_EXPIRY_PROBE_H
#include <sys/types.h>
enum expiry_probe_claim { EXPIRY_DETERMINISTIC_RECIPIENT, EXPIRY_UNMASKED_EXIT_ONLY };
struct expiry_observation { int kind; pid_t helper; int result; int flags_preserved; int policies_preserved; int relay_stage; int factory_phase; int acknowledged; long long elapsed; };
/* Fail this isolated probe with the observed cause; cpu_probe_expiry_require(fd>=0,"owned output"). */
void cpu_probe_expiry_require(int condition, const char *label);
/* Run the real CPU expiry fixture with explicit callback I/O; cpu_probe_run_expiry_parent(out,in,monitor,0,0,EXPIRY_DETERMINISTIC_RECIPIENT,block). */
_Noreturn void cpu_probe_run_expiry_parent(int output, int reader, int monitor, int policy, int mode, enum expiry_probe_claim claim, void (*blocking_callback)(void));
#endif
