#define _DARWIN_C_SOURCE 1
#include "probe_macos_cpu_helper.h"
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

static void require_observation(int observed, const char *label) {
  if (observed) return;
  fprintf(stderr, "native reaper refutation failed: %s (errno=%d)\n", label, errno);
  _exit(81);
}

struct application_reaper { pid_t child; int start; pid_t observed; int status; };

static void *reap_application_child(void *argument) {
  struct application_reaper *reaper = argument;
  char release;
  require_observation(read(reaper->start, &release, 1) == 1, "reaper release");
  do { reaper->observed = waitpid(reaper->child, &reaper->status, 0); }
  while (reaper->observed < 0 && errno == EINTR);
  return NULL;
}

static void await_lost_ownership(pid_t child) {
  long long started = cpu_probe_clock();
  require_observation(started >= 0, "valid deadline start");
  for (;;) {
    int status;
    pid_t observed = waitpid(child, &status, WNOHANG);
    if (observed == -1 && errno == ECHILD) return;
    require_observation(observed == 0 || (observed == -1 && errno == EINTR), "no owner reap");
    long long now = cpu_probe_clock();
    require_observation(now >= 0 && now - started < 12000000000LL, "bounded auto-reap");
    usleep(1000);
  }
}

static void apply_policy(int policy) {
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  sigemptyset(&action.sa_mask);
  action.sa_handler = policy == 0 ? SIG_IGN : SIG_DFL;
  action.sa_flags = policy == 1 ? SA_NOCLDWAIT : 0;
  require_observation(sigaction(SIGCHLD, &action, NULL) == 0, "controlled owner SIGCHLD policy");
}

static void finish_application_reap(struct application_reaper *reaper, int start[2], pthread_t thread) {
  require_observation(write(start[1], "x", 1) == 1, "release actual application reaper");
  require_observation(pthread_join(thread, NULL) == 0, "join actual application reaper");
  require_observation(reaper->observed == reaper->child && WIFEXITED(reaper->status)
      && WEXITSTATUS(reaper->status) == 0, "application reaped original helper");
  close(start[0]);
  close(start[1]);
}

static void observe_race_gap(int policy) {
  struct cpu_helper_probe helper = {-1, -1, -1};
  require_observation(cpu_probe_prepare(&helper, 4096) == 0, "prepare actual helper");
  int start[2] = {-1, -1};
  pthread_t reaper_thread;
  struct application_reaper reaper = {helper.pid, -1, -1, 0};
  if (policy == 2) {
    require_observation(pipe(start) == 0, "application reaper synchronization");
    reaper.start = start[0];
    require_observation(pthread_create(&reaper_thread, NULL, reap_application_child, &reaper) == 0,
        "start actual application reaper");
  }
  int status;
  require_observation(waitpid(helper.pid, &status, WNOHANG) == 0, "helper alive at original pre-kill check");
  close(helper.control); /* EOF asks this exact helper to exit; no numeric PID is signalled. */
  if (policy == 2) finish_application_reap(&reaper, start, reaper_thread);
  await_lost_ownership(helper.pid);
}

static void report_refutation(const char *name) {
  printf("{\"probe\":\"reaper_ownership_refuted\",\"policy\":\"%s\",\"repetitions\":32,"
      "\"alive_check_returned_zero\":true,\"later_errno\":%d,\"post_reap_signal_sent\":false,"
      "\"pid_reuse_observed\":false,\"descriptor_count_preserved\":true}\n", name, ECHILD);
  fflush(stdout);
}

static void run_policy_owner(int policy, const char *name) {
  alarm(15);
  apply_policy(policy);
  struct sigaction before, after;
  require_observation(sigaction(SIGCHLD, NULL, &before) == 0, "policy snapshot");
  int descriptors = cpu_probe_fd_count();
  require_observation(descriptors >= 0, "initial descriptor count");
  for (int repetition = 0; repetition < 32; ++repetition) observe_race_gap(policy);
  require_observation(cpu_probe_fd_count() == descriptors, "descriptor count preserved");
  require_observation(sigaction(SIGCHLD, NULL, &after) == 0, "final policy snapshot");
  require_observation(before.sa_handler == after.sa_handler && before.sa_flags == after.sa_flags,
      "caller reaping policy preserved");
  report_refutation(name);
  _exit(0);
}

static void probe_policy(int policy, const char *name) {
  pid_t owner = fork();
  require_observation(owner >= 0, "fork isolated policy owner");
  if (owner == 0) run_policy_owner(policy, name);
  int status;
  pid_t observed;
  do { observed = waitpid(owner, &status, 0); } while (observed < 0 && errno == EINTR);
  require_observation(observed == owner && WIFEXITED(status) && WEXITSTATUS(status) == 0,
      "isolated policy owner completed");
}

int main(void) {
  require_observation(signal(SIGCHLD, SIG_DFL) != SIG_ERR, "isolated outer owner policy");
  probe_policy(0, "SIG_IGN");
  probe_policy(1, "SA_NOCLDWAIT");
  probe_policy(2, "application_waitpid_reaper");
  return 0;
}
