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

_Static_assert(ATOMIC_INT_LOCK_FREE == 2 && ATOMIC_LLONG_LOCK_FREE == 2, "signal telemetry atomics must be lock free");

static int request_pipe[2], acknowledgement[2], monitor_fd, output_flags, expiry_mode;
static _Atomic long long request_deadline;
static _Atomic int completed, helper_result, relay_ready, relay_stage, acknowledged, fixture_workers_ready;
static enum expiry_probe_claim fixture_claim;
static sigset_t relay_original_mask;
static _Atomic pid_t helper_pid;
static struct sigaction original_pipe_policy, original_child_policy;

static void report_helper(pid_t helper, int failure) {
  struct expiry_observation observation = {1, helper, failure, 0, 0, atomic_load(&relay_stage), cpu_probe_bounded_spawn_phase(), atomic_load(&acknowledged), 0};
  cpu_probe_expiry_require(write(monitor_fd, &observation, sizeof(observation)) == sizeof(observation), "helper observation");
}

static void prepare_relay_thread(void) {
  sigset_t blocked;
  sigemptyset(&blocked);
  sigaddset(&blocked, SIGXCPU);
  sigaddset(&blocked, SIGPIPE);
  cpu_probe_expiry_require(pthread_sigmask(SIG_BLOCK, &blocked, NULL) == 0, "private relay signal mask");
  cpu_probe_expiry_require(pthread_sigmask(SIG_SETMASK, NULL, &relay_original_mask) == 0, "relay mask snapshot");
  cpu_probe_expiry_require(cpu_probe_prepare_bounded_runtime() == 0, "ordinary runtime and public primitive bound before readiness");
  atomic_store(&relay_ready, 1);
}

static int caller_policies_preserved(void) {
  struct sigaction pipe_policy, child_policy;
  memset(&pipe_policy, 0, sizeof(pipe_policy));
  memset(&child_policy, 0, sizeof(child_policy));
  if (sigaction(SIGPIPE, NULL, &pipe_policy) != 0 || sigaction(SIGCHLD, NULL, &child_policy) != 0) return 0;
  return pipe_policy.sa_handler == original_pipe_policy.sa_handler && pipe_policy.sa_flags == original_pipe_policy.sa_flags
      && memcmp(&pipe_policy.sa_mask, &original_pipe_policy.sa_mask, sizeof(sigset_t)) == 0
      && child_policy.sa_handler == original_child_policy.sa_handler && child_policy.sa_flags == original_child_policy.sa_flags
      && memcmp(&child_policy.sa_mask, &original_child_policy.sa_mask, sizeof(sigset_t)) == 0;
}

static void assert_application_policy_unchanged(void) {
  sigset_t relay_current_mask;
  cpu_probe_expiry_require(pthread_sigmask(SIG_SETMASK, NULL, &relay_current_mask) == 0
      && memcmp(&relay_current_mask, &relay_original_mask, sizeof(sigset_t)) == 0, "relay mask preserved");
  cpu_probe_expiry_require(caller_policies_preserved(), "actual caller SIGPIPE and reaper policies preserved");
}

static void receive_child_completion(pid_t helper, int failure) {
  ssize_t count = -1;
  if (helper >= 0) do { count = read(acknowledgement[0], &failure, sizeof(failure)); }
    while (count < 0 && errno == EINTR);
  atomic_store(&acknowledged, helper >= 0 && count == sizeof(failure));
  atomic_store(&helper_result, helper < 0 ? failure : count == sizeof(failure) ? failure : ETIMEDOUT);
  atomic_store(&relay_stage, 4);
  atomic_store(&completed, 1);
}

static void spawn_expired_output(void) {
  int gate[2];
  if (expiry_mode == 7) cpu_probe_expiry_require(pipe(gate) == 0, "controlled pre-guard stall until parent death");
  atomic_store(&relay_stage, 2);
  pid_t helper = cpu_probe_spawn_bounded(STDOUT_FILENO, acknowledgement[1],
      atomic_load(&request_deadline), expiry_mode == 4 ? 0 : 4096, expiry_mode == 7 ? gate : NULL);
  int spawn_failure = helper < 0 ? errno : 0;
  atomic_store(&relay_stage, 3);
  assert_application_policy_unchanged();
  atomic_store(&helper_pid, helper);
  report_helper(helper, spawn_failure);
  close(acknowledgement[1]);
  receive_child_completion(helper, spawn_failure);
}

static void *expire_regular_relay(void *unused) {
  (void)unused;
  prepare_relay_thread();
  char requested;
  cpu_probe_expiry_require(read(request_pipe[0], &requested, 1) == 1, "regular relay request");
  atomic_store(&relay_stage, 1);
  if (expiry_mode == 3) for (;;) pause();
  spawn_expired_output();
  return NULL;
}

static void await_original_grace(long long started) {
  while (!atomic_load(&completed)) {
    long long now = cpu_probe_clock();
    if (now < 0 || now - started >= 100000000LL) return;
  }
}

static struct expiry_observation published_expiry_snapshot(long long started) {
  int done = atomic_load(&completed);
  int stage = done ? 4 : atomic_load(&relay_stage);
  if (!done && stage == 4) stage = 3;
  struct expiry_observation observation = {2, atomic_load(&helper_pid), done ? atomic_load(&helper_result) : EINPROGRESS,
      fcntl(STDOUT_FILENO, F_GETFL) == output_flags && fcntl(STDOUT_FILENO, F_GETNOSIGPIPE) == 0,
      caller_policies_preserved(), stage, cpu_probe_bounded_spawn_phase(), done ? atomic_load(&acknowledged) : 0,
      cpu_probe_clock() - started};
  return observation;
}

static void expiry_without_pid_actions(int signal_number) {
  (void)signal_number;
  long long started = cpu_probe_clock();
  if (started < 0) _exit(2);
  atomic_store(&request_deadline, started + 100000000LL);
  (void)write(request_pipe[1], "x", 1);
  await_original_grace(started);
  struct expiry_observation observation = published_expiry_snapshot(started);
  (void)write(monitor_fd, &observation, sizeof(observation));
  _exit(2);
}

static void configure_fixture_recipient(void) {
  sigset_t blocked;
  sigemptyset(&blocked);
  /* Test-only recipient control; production must not change application masks. */
  if (fixture_claim == EXPIRY_DETERMINISTIC_RECIPIENT) {
    sigaddset(&blocked, SIGXCPU);
    cpu_probe_expiry_require(pthread_sigmask(SIG_BLOCK, &blocked, NULL) == 0, "fixture CPU recipient mask");
  }
  cpu_probe_expiry_require(pthread_sigmask(SIG_SETMASK, NULL, &blocked) == 0
      && sigismember(&blocked, SIGPIPE) == 0
      && sigismember(&blocked, SIGXCPU) == (fixture_claim == EXPIRY_DETERMINISTIC_RECIPIENT),
      "declared fixture CPU recipient and unmasked application SIGPIPE");
  atomic_fetch_add(&fixture_workers_ready, 1);
}

static void *allocation_stress(void *unused) {
  (void)unused;
  configure_fixture_recipient();
  for (;;) {
    char *allocation = malloc(65536);
    if (allocation) { memset(allocation, 1, 65536); free(allocation); }
  }
  return NULL;
}

static void *loader_stress(void *unused) {
  (void)unused;
  configure_fixture_recipient();
  for (;;) {
    void *library = dlopen("/usr/lib/libSystem.B.dylib", RTLD_LAZY);
    if (library) dlclose(library);
  }
  return NULL;
}

static void *application_reaper(void *unused) {
  (void)unused;
  configure_fixture_recipient();
  for (;;) { int status; (void)waitpid(-1, &status, WNOHANG); usleep(100); }
  return NULL;
}

static void start_concurrent_application(int policy) {
  pthread_t thread;
  for (int i = 0; i < 4; ++i)
    cpu_probe_expiry_require(pthread_create(&thread, NULL, allocation_stress, NULL) == 0, "allocator stress thread");
  for (int i = 0; i < 2; ++i)
    cpu_probe_expiry_require(pthread_create(&thread, NULL, loader_stress, NULL) == 0, "loader stress thread");
  if (policy == 2)
    cpu_probe_expiry_require(pthread_create(&thread, NULL, application_reaper, NULL) == 0, "application reaper thread");
  while (atomic_load(&fixture_workers_ready) < 6 + (policy == 2)) usleep(100);
}

static void install_application_policy(int policy, void (*blocking_callback)(void)) {
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  sigemptyset(&action.sa_mask);
  action.sa_handler = policy == 0 ? SIG_IGN : SIG_DFL;
  action.sa_flags = policy == 1 ? SA_NOCLDWAIT : 0;
  cpu_probe_expiry_require(sigaction(SIGCHLD, &action, NULL) == 0, "application reaping policy");
  cpu_probe_expiry_require(signal(SIGPIPE, SIG_DFL) != SIG_ERR, "application SIGPIPE default");
  memset(&original_pipe_policy, 0, sizeof(original_pipe_policy));
  memset(&original_child_policy, 0, sizeof(original_child_policy));
  cpu_probe_expiry_require(sigaction(SIGPIPE, NULL, &original_pipe_policy) == 0 && sigaction(SIGCHLD, NULL, &original_child_policy) == 0,
      "application signal snapshots");
  cpu_probe_expiry_require(pthread_atfork(blocking_callback, blocking_callback, blocking_callback) == 0,
      "actual blocking user atfork callbacks");
}

static void install_original_cpu_budget(void) {
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  sigemptyset(&action.sa_mask);
  sigaddset(&action.sa_mask, SIGPIPE);
  action.sa_handler = expiry_without_pid_actions;
  cpu_probe_expiry_require(sigaction(SIGXCPU, &action, NULL) == 0, "native CPU expiry action");
  struct rlimit limit;
  cpu_probe_expiry_require(getrlimit(RLIMIT_CPU, &limit) == 0, "read CPU budget");
  limit.rlim_cur = 1;
  cpu_probe_expiry_require(setrlimit(RLIMIT_CPU, &limit) == 0, "original one-second CPU budget");
}

static void start_request_relay(void) {
  cpu_probe_expiry_require(pipe(request_pipe) == 0 && socketpair(AF_UNIX, SOCK_STREAM, 0, acknowledgement) == 0,
      "private regular relay channels");
  pthread_t relay;
  cpu_probe_expiry_require(pthread_create(&relay, NULL, expire_regular_relay, NULL) == 0, "regular relay thread");
  while (!atomic_load(&relay_ready)) usleep(100);
}

static void refuse_actual_spawn(int mode) {
  if (mode != 6) return;
  struct rlimit process_limit;
  cpu_probe_expiry_require(getrlimit(RLIMIT_NPROC, &process_limit) == 0, "read process limit fault fixture");
  process_limit.rlim_cur = 0;
  cpu_probe_expiry_require(setrlimit(RLIMIT_NPROC, &process_limit) == 0, "actual spawn refusal fault fixture");
}

static int initialize_current_output(int output, int mode) {
  if (mode != 8) {
    cpu_probe_expiry_require(dup2(output, STDOUT_FILENO) == STDOUT_FILENO, "bind actual current stdout");
    close(output);
    return -1;
  }
  int original[2];
  cpu_probe_expiry_require(pipe(original) == 0, "stdout before relay initialization");
  cpu_probe_expiry_require(dup2(original[1], STDOUT_FILENO) == STDOUT_FILENO, "bind initial stdout fixture");
  close(original[1]);
  return original[0];
}

static void redirect_after_initialization(int output, int original_reader) {
  if (original_reader < 0) return;
  cpu_probe_expiry_require(dup2(output, STDOUT_FILENO) == STDOUT_FILENO, "redirect stdout after relay readiness");
  close(output);
  char byte;
  cpu_probe_expiry_require(read(original_reader, &byte, 1) == 0, "initial stdout has no retained helper writer");
  close(original_reader);
}

_Noreturn void cpu_probe_run_expiry_parent(int output, int reader, int monitor, int policy, int mode, enum expiry_probe_claim claim, void (*blocking_callback)(void)) {
  close(reader);
  int original_reader = initialize_current_output(output, mode);
  monitor_fd = monitor;
  expiry_mode = mode;
  fixture_claim = claim;
  helper_pid = -1;
  helper_result = EINPROGRESS;
  install_application_policy(policy, blocking_callback);
  start_request_relay();
  redirect_after_initialization(output, original_reader);
  output_flags = fcntl(STDOUT_FILENO, F_GETFL);
  start_concurrent_application(policy);
  refuse_actual_spawn(mode);
  install_original_cpu_budget();
  volatile unsigned long accumulator = 1;
  for (;;) accumulator = accumulator * 3 + 1;
}

