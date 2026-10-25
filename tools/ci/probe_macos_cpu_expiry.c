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

_Static_assert(ATOMIC_INT_LOCK_FREE == 2 && ATOMIC_LLONG_LOCK_FREE == 2, "signal telemetry atomics must be lock free");

struct expiry_observation { int kind; pid_t helper; int result; int flags_preserved; int relay_stage; long long elapsed; };
static int request_pipe[2], acknowledgement[2], monitor_fd, output_flags, expiry_mode;
static _Atomic long long request_deadline;
static _Atomic int completed, helper_result, relay_ready, relay_stage;
static sigset_t relay_original_mask;
static _Atomic pid_t helper_pid;
static volatile sig_atomic_t callback_fd = -1;
static struct sigaction original_pipe_policy, original_child_policy;

static void demand(int condition, const char *label) {
  if (condition) return;
  fprintf(stderr, "native expiry probe failed: %s (errno=%d)\n", label, errno);
  _exit(82);
}

static void blocking_user_atfork(void) {
  if (callback_fd >= 0) (void)write(callback_fd, "B", 1);
  for (;;) pause();
}

static void report_helper(pid_t helper, int failure) {
  struct expiry_observation observation = {1, helper, failure, 0, atomic_load(&relay_stage), 0};
  demand(write(monitor_fd, &observation, sizeof(observation)) == sizeof(observation), "helper observation");
}

static void prepare_relay_thread(void) {
  sigset_t blocked;
  sigemptyset(&blocked);
  sigaddset(&blocked, SIGXCPU);
  sigaddset(&blocked, SIGPIPE);
  demand(pthread_sigmask(SIG_BLOCK, &blocked, NULL) == 0, "private relay signal mask");
  demand(pthread_sigmask(SIG_SETMASK, NULL, &relay_original_mask) == 0, "relay mask snapshot");
  atomic_store(&relay_ready, 1);
}

static void assert_application_policy_unchanged(void) {
  sigset_t relay_current_mask;
  demand(pthread_sigmask(SIG_SETMASK, NULL, &relay_current_mask) == 0
      && memcmp(&relay_current_mask, &relay_original_mask, sizeof(sigset_t)) == 0, "relay mask preserved");
  struct sigaction pipe_policy, child_policy;
  memset(&pipe_policy, 0, sizeof(pipe_policy));
  memset(&child_policy, 0, sizeof(child_policy));
  demand(sigaction(SIGPIPE, NULL, &pipe_policy) == 0 && sigaction(SIGCHLD, NULL, &child_policy) == 0,
      "actual parent signal policies");
  demand(pipe_policy.sa_handler == original_pipe_policy.sa_handler && pipe_policy.sa_flags == original_pipe_policy.sa_flags
      && memcmp(&pipe_policy.sa_mask, &original_pipe_policy.sa_mask, sizeof(sigset_t)) == 0, "parent SIGPIPE policy preserved");
  demand(child_policy.sa_handler == original_child_policy.sa_handler && child_policy.sa_flags == original_child_policy.sa_flags
      && memcmp(&child_policy.sa_mask, &original_child_policy.sa_mask, sizeof(sigset_t)) == 0, "application reaper policy preserved");
}

static void receive_child_completion(pid_t helper, int failure) {
  ssize_t count = -1;
  if (helper >= 0) do { count = read(acknowledgement[0], &failure, sizeof(failure)); }
    while (count < 0 && errno == EINTR);
  atomic_store(&helper_result, helper < 0 ? failure : count == sizeof(failure) ? failure : ETIMEDOUT);
  atomic_store(&relay_stage, 4);
  atomic_store(&completed, 1);
}

static void spawn_expired_output(void) {
  int gate[2];
  if (expiry_mode == 7) demand(pipe(gate) == 0, "controlled pre-guard stall until parent death");
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
  demand(read(request_pipe[0], &requested, 1) == 1, "regular relay request");
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

static void expiry_without_pid_actions(int signal_number) {
  (void)signal_number;
  long long started = cpu_probe_clock();
  if (started < 0) _exit(2);
  atomic_store(&request_deadline, started + 100000000LL);
  (void)write(request_pipe[1], "x", 1);
  await_original_grace(started);
  struct expiry_observation observation = {2, atomic_load(&helper_pid), atomic_load(&helper_result),
      fcntl(STDOUT_FILENO, F_GETFL) == output_flags && fcntl(STDOUT_FILENO, F_GETNOSIGPIPE) == 0,
      atomic_load(&relay_stage), cpu_probe_clock() - started};
  (void)write(monitor_fd, &observation, sizeof(observation));
  _exit(2);
}

static void *allocation_stress(void *unused) {
  (void)unused;
  for (;;) {
    char *allocation = malloc(65536);
    if (allocation) { memset(allocation, 1, 65536); free(allocation); }
  }
  return NULL;
}

static void *loader_stress(void *unused) {
  (void)unused;
  for (;;) {
    void *library = dlopen("/usr/lib/libSystem.B.dylib", RTLD_LAZY);
    if (library) dlclose(library);
  }
  return NULL;
}

static void *application_reaper(void *unused) {
  (void)unused;
  for (;;) { int status; (void)waitpid(-1, &status, WNOHANG); usleep(100); }
  return NULL;
}

static void start_concurrent_application(int policy) {
  pthread_t thread;
  for (int i = 0; i < 4; ++i)
    demand(pthread_create(&thread, NULL, allocation_stress, NULL) == 0, "allocator stress thread");
  for (int i = 0; i < 2; ++i)
    demand(pthread_create(&thread, NULL, loader_stress, NULL) == 0, "loader stress thread");
  if (policy == 2)
    demand(pthread_create(&thread, NULL, application_reaper, NULL) == 0, "application reaper thread");
}

static void install_application_policy(int policy) {
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  sigemptyset(&action.sa_mask);
  action.sa_handler = policy == 0 ? SIG_IGN : SIG_DFL;
  action.sa_flags = policy == 1 ? SA_NOCLDWAIT : 0;
  demand(sigaction(SIGCHLD, &action, NULL) == 0, "application reaping policy");
  demand(signal(SIGPIPE, SIG_DFL) != SIG_ERR, "application SIGPIPE default");
  memset(&original_pipe_policy, 0, sizeof(original_pipe_policy));
  memset(&original_child_policy, 0, sizeof(original_child_policy));
  demand(sigaction(SIGPIPE, NULL, &original_pipe_policy) == 0 && sigaction(SIGCHLD, NULL, &original_child_policy) == 0,
      "application signal snapshots");
  demand(pthread_atfork(blocking_user_atfork, blocking_user_atfork, blocking_user_atfork) == 0,
      "actual blocking user atfork callbacks");
}

static void install_original_cpu_budget(void) {
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  sigemptyset(&action.sa_mask);
  sigaddset(&action.sa_mask, SIGPIPE);
  action.sa_handler = expiry_without_pid_actions;
  demand(sigaction(SIGXCPU, &action, NULL) == 0, "native CPU expiry action");
  struct rlimit limit;
  demand(getrlimit(RLIMIT_CPU, &limit) == 0, "read CPU budget");
  limit.rlim_cur = 1;
  demand(setrlimit(RLIMIT_CPU, &limit) == 0, "original one-second CPU budget");
}

static void start_request_relay(void) {
  demand(pipe(request_pipe) == 0 && socketpair(AF_UNIX, SOCK_STREAM, 0, acknowledgement) == 0,
      "private regular relay channels");
  pthread_t relay;
  demand(pthread_create(&relay, NULL, expire_regular_relay, NULL) == 0, "regular relay thread");
  while (!atomic_load(&relay_ready)) usleep(100);
}

static void refuse_actual_spawn(int mode) {
  if (mode != 6) return;
  struct rlimit process_limit;
  demand(getrlimit(RLIMIT_NPROC, &process_limit) == 0, "read process limit fault fixture");
  process_limit.rlim_cur = 0;
  demand(setrlimit(RLIMIT_NPROC, &process_limit) == 0, "actual spawn refusal fault fixture");
}

static int initialize_current_output(int output, int mode) {
  if (mode != 8) {
    demand(dup2(output, STDOUT_FILENO) == STDOUT_FILENO, "bind actual current stdout");
    close(output);
    return -1;
  }
  int original[2];
  demand(pipe(original) == 0, "stdout before relay initialization");
  demand(dup2(original[1], STDOUT_FILENO) == STDOUT_FILENO, "bind initial stdout fixture");
  close(original[1]);
  return original[0];
}

static void redirect_after_initialization(int output, int original_reader) {
  if (original_reader < 0) return;
  demand(dup2(output, STDOUT_FILENO) == STDOUT_FILENO, "redirect stdout after relay readiness");
  close(output);
  char byte;
  demand(read(original_reader, &byte, 1) == 0, "initial stdout has no retained helper writer");
  close(original_reader);
}

static void run_cpu_parent(int output, int reader, int monitor, int policy, int mode) {
  close(reader);
  int original_reader = initialize_current_output(output, mode);
  monitor_fd = monitor;
  expiry_mode = mode;
  helper_pid = -1;
  install_application_policy(policy);
  start_request_relay();
  redirect_after_initialization(output, original_reader);
  output_flags = fcntl(STDOUT_FILENO, F_GETFL);
  start_concurrent_application(policy);
  refuse_actual_spawn(mode);
  install_original_cpu_budget();
  volatile unsigned long accumulator = 1;
  for (;;) accumulator = accumulator * 3 + 1;
}

static void prepare_output(int descriptors[2], int full) {
  demand(pipe(descriptors) == 0, "stdout pipe fixture");
  int flags = fcntl(descriptors[1], F_GETFL);
  demand(fcntl(descriptors[1], F_SETFL, flags | O_NONBLOCK) == 0, "fixture fill mode");
  char bytes[4096];
  memset(bytes, 'x', sizeof(bytes));
  if (full) {
    while (write(descriptors[1], bytes, sizeof(bytes)) > 0) {}
    while (write(descriptors[1], "x", 1) > 0) {}
    demand(errno == EAGAIN, "full stdout fixture");
  } else demand(write(descriptors[1], "x", 1) == 1 && read(descriptors[0], bytes, 1) == 1,
      "natural write metadata before flag snapshot");
  demand(fcntl(descriptors[1], F_SETFL, flags) == 0, "restore fixture configuration");
}

static struct expiry_observation read_observation(int descriptor) {
  struct pollfd ready = {descriptor, POLLIN, 0};
  demand(poll(&ready, 1, 12000) > 0, "original twelve-second safety guard");
  struct expiry_observation observation;
  demand(read(descriptor, &observation, sizeof(observation)) == sizeof(observation), "whole observation");
  fprintf(stderr, "{\"probe\":\"raw_observation\",\"kind\":%d,\"helper\":%d,\"result\":%d,"
      "\"flags_preserved\":%d,\"relay_stage\":%d,\"elapsed_ns\":%lld}\n", observation.kind,
      observation.helper, observation.result, observation.flags_preserved, observation.relay_stage, observation.elapsed);
  return observation;
}

static void await_controlled_parent(pid_t parent, int expected_signal) {
  int status;
  pid_t observed;
  do { observed = waitpid(parent, &status, 0); } while (observed < 0 && errno == EINTR);
  demand(observed == parent, "controlled parent reaped");
  if (expected_signal) demand(WIFSIGNALED(status) && WTERMSIG(status) == expected_signal, "forced parent death");
  else demand(WIFEXITED(status) && WEXITSTATUS(status) == 2, "unchanged CPU expiry exit2");
}

static void await_output_eof(int descriptor) {
  int flags = fcntl(descriptor, F_GETFL);
  demand(fcntl(descriptor, F_SETFL, flags | O_NONBLOCK) == 0, "observer drain mode");
  long long started = cpu_probe_clock();
  char bytes[4096];
  for (;;) {
    ssize_t count = read(descriptor, bytes, sizeof(bytes));
    if (count == 0) return;
    demand(count > 0 || errno == EAGAIN || errno == EINTR, "observer pipe drain");
    demand(cpu_probe_clock() - started < 12000000000LL, "bounded helper output closure");
    if (count < 0) usleep(1000);
  }
}

static struct expiry_observation observe_parent_expiry(int descriptor, pid_t parent, int mode) {
  struct expiry_observation observation = read_observation(descriptor);
  if (mode == 5 || mode == 7) {
    demand(observation.kind == 1 && observation.helper > 0, "actual expiry-only helper before forced death");
    demand(kill(parent, SIGKILL) == 0, "forced controlled parent death");
    return observation;
  }
  if (observation.kind == 1) observation = read_observation(descriptor);
  demand(observation.kind == 2 && observation.flags_preserved, "actual CPU expiry and exact stdout flags");
  if (mode == 2 || mode == 3) demand(observation.elapsed >= 100000000LL, "unchanged full diagnostic grace");
  if (mode != 3 && mode != 6) demand(observation.helper > 0, "expiry helper bypassed all blocking user atfork callbacks");
  if (mode == 1) demand(observation.result == EPIPE, "actual closed stdout EPIPE");
  if (mode == 4) demand(observation.result == EOVERFLOW, "actual child initialization refusal");
  if (mode == 6) demand(observation.helper == -1 && observation.result == EAGAIN, "actual vfork syscall failure");
  return observation;
}

static void check_and_close_output(int descriptor, int mode) {
  if (descriptor < 0) return;
  if (mode == 0 || mode == 8) {
    char message[sizeof(CPU_PROBE_MESSAGE)] = {0};
    demand(read(descriptor, message, sizeof(message)) == sizeof(CPU_PROBE_MESSAGE) - 1
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
  demand(pipe(monitor) == 0, "observer telemetry pipe");
  pid_t parent = fork();
  demand(parent >= 0, "controlled parent fork");
  if (parent == 0) { close(monitor[0]); run_cpu_parent(output[1], output[0], monitor[1], policy, mode); }
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
  demand(pipe(events) == 0, "atfork positive-control channel");
  pid_t owner = fork();
  demand(owner >= 0, "atfork positive-control owner");
  if (owner == 0) {
    close(events[0]);
    callback_fd = events[1];
    demand(pthread_atfork(blocking_user_atfork, NULL, NULL) == 0, "positive-control registration");
    (void)fork();
    _exit(84);
  }
  close(events[1]);
  struct pollfd ready = {events[0], POLLIN, 0};
  char marker;
  demand(poll(&ready, 1, 12000) > 0 && read(events[0], &marker, 1) == 1 && marker == 'B', "actual blocking callback ran");
  demand(kill(owner, SIGKILL) == 0, "cleanup blocked positive-control owner with controlled default reaping");
  await_controlled_parent(owner, SIGKILL);
  close(events[0]);
}

static void terminate_blocked_callback_child(pid_t child, int descriptor) {
  struct pollfd ready = {descriptor, POLLIN, 0};
  char marker;
  demand(poll(&ready, 1, 12000) > 0 && read(descriptor, &marker, 1) == 1 && marker == 'B',
      "actual post-fork user callback blocked before any guard");
  int status;
  demand(waitpid(child, &status, WNOHANG) == 0, "blocked child has not reached user body");
  demand(kill(child, SIGKILL) == 0, "cleanup controlled default-reaping callback child");
  await_controlled_parent(child, SIGKILL);
}

static void run_child_callback_owner(void) {
  int events[2];
  demand(pipe(events) == 0, "child callback positive-control channel");
  callback_fd = events[1];
  demand(pthread_atfork(NULL, NULL, blocking_user_atfork) == 0, "blocking child callback registration");
  pid_t child = fork();
  demand(child >= 0, "ordinary fork child callback control");
  if (child == 0) _exit(85);
  close(events[1]);
  terminate_blocked_callback_child(child, events[0]);
  close(events[0]);
  _exit(0);
}

static void prove_child_callback_really_blocks(void) {
  pid_t owner = fork();
  demand(owner >= 0, "isolated child callback owner");
  if (owner == 0) run_child_callback_owner();
  int status;
  demand(waitpid(owner, &status, 0) == owner && WIFEXITED(status) && WEXITSTATUS(status) == 0,
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
  demand(signal(SIGCHLD, SIG_DFL) != SIG_ERR, "outer controlled observer reaping");
  int descriptors = cpu_probe_fd_count();
  demand(descriptors >= 0, "initial observer descriptor count");
  prove_fork_callback_really_blocks();
  prove_child_callback_really_blocks();
  probe_all_reaping_policies();
  demand(cpu_probe_fd_count() == descriptors, "observer descriptors preserved");
  printf("{\"probe\":\"expiry_prototype_complete\",\"descriptor_count_preserved\":true}\n");
  return 0;
}
