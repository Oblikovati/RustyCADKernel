#define _DARWIN_C_SOURCE 1
#include "probe_macos_cpu_helper.h"
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <pthread.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <unistd.h>

#include "probe_macos_cpu_expiry.h"

static volatile sig_atomic_t callback_fd = -1;

void cpu_probe_expiry_require(int condition, const char *label) {
  if (condition) return;
  fprintf(stderr, "native expiry probe failed: %s (errno=%d)\n", label, errno);
  _exit(82);
}

static void blocking_user_atfork(void) {
  if (callback_fd >= 0) (void)write(callback_fd, "B", 1);
  for (;;) pause();
}

static void prepare_output(int descriptors[2], int full) {
  cpu_probe_expiry_require(pipe(descriptors) == 0, "stdout pipe fixture");
  int flags = fcntl(descriptors[1], F_GETFL);
  cpu_probe_expiry_require(fcntl(descriptors[1], F_SETFL, flags | O_NONBLOCK) == 0, "fixture fill mode");
  char bytes[4096];
  memset(bytes, 'x', sizeof(bytes));
  if (full) {
    while (write(descriptors[1], bytes, sizeof(bytes)) > 0) {}
    while (write(descriptors[1], "x", 1) > 0) {}
    cpu_probe_expiry_require(errno == EAGAIN, "full stdout fixture");
  } else cpu_probe_expiry_require(write(descriptors[1], "x", 1) == 1 && read(descriptors[0], bytes, 1) == 1,
      "natural write metadata before flag snapshot");
  cpu_probe_expiry_require(fcntl(descriptors[1], F_SETFL, flags) == 0, "restore fixture configuration");
}

static struct expiry_observation read_observation(int descriptor) {
  struct pollfd ready = {descriptor, POLLIN, 0};
  cpu_probe_expiry_require(poll(&ready, 1, 12000) > 0, "original twelve-second safety guard");
  struct expiry_observation observation;
  cpu_probe_expiry_require(read(descriptor, &observation, sizeof(observation)) == sizeof(observation), "whole observation");
  fprintf(stderr, "{\"probe\":\"raw_observation\",\"kind\":%d,\"helper\":%d,\"result\":%d,"
      "\"flags_preserved\":%d,\"relay_stage\":%d,\"factory_phase\":%d,\"elapsed_ns\":%lld}\n", observation.kind,
      observation.helper, observation.result, observation.flags_preserved, observation.relay_stage, observation.factory_phase, observation.elapsed);
  return observation;
}

static void await_controlled_parent(pid_t parent, int expected_signal) {
  int status;
  pid_t observed;
  do { observed = waitpid(parent, &status, 0); } while (observed < 0 && errno == EINTR);
  cpu_probe_expiry_require(observed == parent, "controlled parent reaped");
  if (expected_signal) cpu_probe_expiry_require(WIFSIGNALED(status) && WTERMSIG(status) == expected_signal, "forced parent death");
  else cpu_probe_expiry_require(WIFEXITED(status) && WEXITSTATUS(status) == 2, "unchanged CPU expiry exit2");
}

static void await_output_eof(int descriptor) {
  int flags = fcntl(descriptor, F_GETFL);
  cpu_probe_expiry_require(fcntl(descriptor, F_SETFL, flags | O_NONBLOCK) == 0, "observer drain mode");
  long long started = cpu_probe_clock();
  char bytes[4096];
  for (;;) {
    ssize_t count = read(descriptor, bytes, sizeof(bytes));
    if (count == 0) return;
    cpu_probe_expiry_require(count > 0 || errno == EAGAIN || errno == EINTR, "observer pipe drain");
    cpu_probe_expiry_require(cpu_probe_clock() - started < 12000000000LL, "bounded helper output closure");
    if (count < 0) usleep(1000);
  }
}

static struct expiry_observation observe_parent_expiry(int descriptor, pid_t parent, int mode) {
  struct expiry_observation observation = read_observation(descriptor);
  if (mode == 5 || mode == 7) {
    cpu_probe_expiry_require(observation.kind == 1 && observation.helper > 0, "actual expiry-only helper before forced death");
    cpu_probe_expiry_require(kill(parent, SIGKILL) == 0, "forced controlled parent death");
    return observation;
  }
  if (observation.kind == 1) observation = read_observation(descriptor);
  cpu_probe_expiry_require(observation.kind == 2 && observation.flags_preserved, "actual CPU expiry and exact stdout flags");
  if (mode == 2 || mode == 3) cpu_probe_expiry_require(observation.elapsed >= 100000000LL, "unchanged full diagnostic grace");
  if (mode != 3 && mode != 6) cpu_probe_expiry_require(observation.helper > 0, "expiry helper bypassed all blocking user atfork callbacks");
  if (mode == 1) cpu_probe_expiry_require(observation.result == EPIPE, "actual closed stdout EPIPE");
  if (mode == 4) cpu_probe_expiry_require(observation.result == EOVERFLOW, "actual child initialization refusal");
  if (mode == 6) cpu_probe_expiry_require(observation.helper == -1 && observation.result == EAGAIN, "actual vfork syscall failure");
  return observation;
}

static void check_and_close_output(int descriptor, int mode) {
  if (descriptor < 0) return;
  if (mode == 0 || mode == 8) {
    char message[sizeof(CPU_PROBE_MESSAGE)] = {0};
    cpu_probe_expiry_require(read(descriptor, message, sizeof(message)) == sizeof(CPU_PROBE_MESSAGE) - 1
        && memcmp(message, CPU_PROBE_MESSAGE, sizeof(CPU_PROBE_MESSAGE) - 1) == 0, "original diagnostic exact bytes");
  }
  await_output_eof(descriptor);
  close(descriptor);
}

static void report_expiry(int policy, int mode, const char *kind, struct expiry_observation observation) {
  printf("{\"probe\":\"expiry_only_helper\",\"policy\":%d,\"kind\":\"%s\",\"exit_code\":%d,"
      "\"diagnostic_ns\":%lld,\"no_parent_pid_actions\":true,\"output_closed\":true}\n",
      policy, kind, (mode == 5 || mode == 7) ? -1 : 2, observation.elapsed);
  fflush(stdout);
}

static void probe_cpu_expiry(int policy, int mode, const char *kind) {
  int output[2], monitor[2];
  prepare_output(output, mode == 2 || mode == 5 || mode == 7);
  cpu_probe_expiry_require(pipe(monitor) == 0, "observer telemetry pipe");
  pid_t parent = fork();
  cpu_probe_expiry_require(parent >= 0, "controlled parent fork");
  if (parent == 0) { close(monitor[0]); cpu_probe_run_expiry_parent(output[1], output[0], monitor[1], policy, mode, blocking_user_atfork); }
  close(output[1]);
  close(monitor[1]);
  if (mode == 1) { close(output[0]); output[0] = -1; }
  struct expiry_observation observation = observe_parent_expiry(monitor[0], parent, mode);
  await_controlled_parent(parent, (mode == 5 || mode == 7) ? SIGKILL : 0);
  check_and_close_output(output[0], mode);
  close(monitor[0]);
  report_expiry(policy, mode, kind, observation);
}

static void prove_fork_callback_really_blocks(void) {
  int events[2];
  cpu_probe_expiry_require(pipe(events) == 0, "atfork positive-control channel");
  pid_t owner = fork();
  cpu_probe_expiry_require(owner >= 0, "atfork positive-control owner");
  if (owner == 0) {
    close(events[0]);
    callback_fd = events[1];
    cpu_probe_expiry_require(pthread_atfork(blocking_user_atfork, NULL, NULL) == 0, "positive-control registration");
    (void)fork();
    _exit(84);
  }
  close(events[1]);
  struct pollfd ready = {events[0], POLLIN, 0};
  char marker;
  cpu_probe_expiry_require(poll(&ready, 1, 12000) > 0 && read(events[0], &marker, 1) == 1 && marker == 'B', "actual blocking callback ran");
  cpu_probe_expiry_require(kill(owner, SIGKILL) == 0, "cleanup blocked positive-control owner with controlled default reaping");
  await_controlled_parent(owner, SIGKILL);
  close(events[0]);
}

static void terminate_blocked_callback_child(pid_t child, int descriptor) {
  struct pollfd ready = {descriptor, POLLIN, 0};
  char marker;
  cpu_probe_expiry_require(poll(&ready, 1, 12000) > 0 && read(descriptor, &marker, 1) == 1 && marker == 'B',
      "actual post-fork user callback blocked before any guard");
  int status;
  cpu_probe_expiry_require(waitpid(child, &status, WNOHANG) == 0, "blocked child has not reached user body");
  cpu_probe_expiry_require(kill(child, SIGKILL) == 0, "cleanup controlled default-reaping callback child");
  await_controlled_parent(child, SIGKILL);
}

static void run_child_callback_owner(void) {
  int events[2];
  cpu_probe_expiry_require(pipe(events) == 0, "child callback positive-control channel");
  callback_fd = events[1];
  cpu_probe_expiry_require(pthread_atfork(NULL, NULL, blocking_user_atfork) == 0, "blocking child callback registration");
  pid_t child = fork();
  cpu_probe_expiry_require(child >= 0, "ordinary fork child callback control");
  if (child == 0) _exit(85);
  close(events[1]);
  terminate_blocked_callback_child(child, events[0]);
  close(events[0]);
  _exit(0);
}

static void prove_child_callback_really_blocks(void) {
  pid_t owner = fork();
  cpu_probe_expiry_require(owner >= 0, "isolated child callback owner");
  if (owner == 0) run_child_callback_owner();
  int status;
  cpu_probe_expiry_require(waitpid(owner, &status, 0) == owner && WIFEXITED(status) && WEXITSTATUS(status) == 0,
      "child callback positive-control complete");
}

static void probe_all_reaping_policies(void) {
  for (int policy = 0; policy < 3; ++policy) {
    probe_cpu_expiry(policy, 0, "writable");
    probe_cpu_expiry(policy, 1, "closed");
    probe_cpu_expiry(policy, 2, "full");
    probe_cpu_expiry(policy, 3, "stalled_relay");
    probe_cpu_expiry(policy, 4, "child_initialization_refused");
    probe_cpu_expiry(policy, 5, "forced_parent_death");
    probe_cpu_expiry(policy, 6, "actual_spawn_refused");
    probe_cpu_expiry(policy, 7, "forced_parent_death_before_guard");
    probe_cpu_expiry(policy, 8, "stdout_redirected_after_initialization");
  }
}

int main(void) {
  cpu_probe_expiry_require(signal(SIGCHLD, SIG_DFL) != SIG_ERR, "outer controlled observer reaping");
  int descriptors = cpu_probe_fd_count();
  cpu_probe_expiry_require(descriptors >= 0, "initial observer descriptor count");
  prove_fork_callback_really_blocks();
  prove_child_callback_really_blocks();
  probe_all_reaping_policies();
  cpu_probe_expiry_require(cpu_probe_fd_count() == descriptors, "observer descriptors preserved");
  printf("{\"probe\":\"expiry_prototype_complete\",\"descriptor_count_preserved\":true}\n");
  return 0;
}
