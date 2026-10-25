#define _DARWIN_C_SOURCE 1
#include "probe_macos_cpu_helper.h"
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <unistd.h>

static struct cpu_helper_probe active_helper = {-1, -1, -1};
static volatile sig_atomic_t suppress_request;
static int report_descriptor = -1;
struct cleanup_report { long long diagnostic_ns; long long cleanup_ns; int acknowledged; int reaped; int diagnostic_errno; };

static void demand(int condition, const char *label) {
  if (condition) return;
  cpu_probe_stop(&active_helper);
  fprintf(stderr, "native helper probe failed: %s (errno=%d)\n", label, errno);
  exit(80);
}

static int helper_is_gone(pid_t child) {
  errno = 0;
  return kill(child, 0) == -1 && errno == ESRCH;
}

static long long elapsed_since(long long start) {
  long long now = cpu_probe_clock();
  return start >= 0 && now >= 0 ? now - start : -1;
}

static int await_diagnostic_grace(long long start, int *diagnostic_errno) {
  for (;;) {
    long long elapsed = elapsed_since(start);
    if (elapsed < 0 || elapsed >= 100000000LL) { *diagnostic_errno = ETIMEDOUT; return 0; }
    int failure = cpu_probe_receive(&active_helper, 0);
    if (failure != ETIMEDOUT) { *diagnostic_errno = failure; return 1; }
  }
}

static void expire_with_owned_helper(int signal_number) {
  (void)signal_number;
  struct cleanup_report report;
  memset(&report, 0, sizeof(report));
  long long start = cpu_probe_clock();
  int failure = suppress_request ? 0 : cpu_probe_send(&active_helper, STDOUT_FILENO);
  report.acknowledged = failure ? 0 : await_diagnostic_grace(start, &report.diagnostic_errno);
  report.diagnostic_ns = elapsed_since(start);
  long long cleanup_started = cpu_probe_clock();
  pid_t helper_pid = active_helper.pid;
  cpu_probe_stop(&active_helper);
  report.cleanup_ns = elapsed_since(cleanup_started);
  int status;
  report.reaped = waitpid(helper_pid, &status, WNOHANG) == -1 && errno == ECHILD;
  (void)write(report_descriptor, &report, sizeof(report));
  _exit(2);
}

static void install_prototype_cpu_action(void) {
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  sigemptyset(&action.sa_mask);
  sigaddset(&action.sa_mask, SIGPIPE);
  action.sa_handler = expire_with_owned_helper;
  demand(sigaction(SIGXCPU, &action, NULL) == 0, "install CPU action");
  struct rlimit limit;
  demand(getrlimit(RLIMIT_CPU, &limit) == 0, "read CPU resource");
  limit.rlim_cur = 1;
  demand(setrlimit(RLIMIT_CPU, &limit) == 0, "apply original one-second CPU budget");
}

static void exhaust_prepared_pipe(int writer, int reader, int full) {
  int flags = fcntl(writer, F_GETFL);
  demand(flags >= 0 && fcntl(writer, F_SETFL, flags | O_NONBLOCK) == 0, "nonblocking fixture fill");
  char bytes[4096];
  memset(bytes, 'x', sizeof(bytes));
  if (full) {
    while (write(writer, bytes, sizeof(bytes)) > 0) {}
    while (write(writer, "x", 1) > 0) {}
    demand(errno == EAGAIN, "full fixture");
  } else {
    demand(write(writer, "x", 1) == 1 && read(reader, bytes, 1) == 1, "written fixture marker");
  }
  demand(fcntl(writer, F_SETFL, flags) == 0, "restore fixture configuration");
}

static void *idle_application_thread(void *unused) {
  (void)unused;
  for (;;) pause();
  return NULL;
}

static void controlled_cpu_child(int output, int inherited_reader, int monitor, int stalled) {
  suppress_request = stalled;
  demand(signal(SIGPIPE, SIG_DFL) != SIG_ERR, "default child SIGPIPE");
  if (inherited_reader >= 0) close(inherited_reader);
  demand(dup2(output, STDOUT_FILENO) == STDOUT_FILENO, "bind current stdout");
  close(output);
  int before = fcntl(STDOUT_FILENO, F_GETFL);
  demand(cpu_probe_prepare(&active_helper, 4096) == 0, "prepare before CPU cap");
  demand(fcntl(STDOUT_FILENO, F_GETFL) == before, "preparation kept stdout flags");
  demand(fcntl(STDOUT_FILENO, F_GETNOSIGPIPE) == 0, "preparation kept stdout SIGPIPE policy");
  demand(write(monitor, &active_helper.pid, sizeof(active_helper.pid)) == sizeof(active_helper.pid), "publish owned PID");
  report_descriptor = monitor;
  pthread_t application_thread;
  demand(pthread_create(&application_thread, NULL, idle_application_thread, NULL) == 0, "unmasked concurrent application thread");
  install_prototype_cpu_action();
  volatile unsigned long accumulator = 1;
  for (;;) accumulator = accumulator * 3 + 1;
}

static void await_cpu_child(pid_t child, int *status) {
  long long start = cpu_probe_clock();
  for (;;) {
    pid_t result = waitpid(child, status, WNOHANG);
    if (result == child) return;
    demand(result == 0 || (result == -1 && errno == EINTR), "controlled child wait");
    demand(cpu_probe_clock() - start < 12000000000LL, "original twelve-second safety guard");
    usleep(10000);
  }
}

struct expiry_fixture { int output[2]; int monitor[2]; int flags; pid_t child; };

static struct expiry_fixture prepared_expiry_fixture(const char *kind, int full, int closed) {
  struct expiry_fixture fixture;
  demand(pipe(fixture.output) == 0 && pipe(fixture.monitor) == 0, "expiry fixture pipes");
  exhaust_prepared_pipe(fixture.output[1], fixture.output[0], full);
  if (closed) { close(fixture.output[0]); fixture.output[0] = -1; }
  fixture.flags = fcntl(fixture.output[1], F_GETFL);
  fixture.child = fork();
  demand(fixture.child >= 0, "controlled CPU child");
  if (fixture.child == 0) {
    close(fixture.monitor[0]);
    controlled_cpu_child(fixture.output[1], fixture.output[0], fixture.monitor[1], strcmp(kind, "stalled") == 0);
  }
  close(fixture.monitor[1]);
  return fixture;
}

static void close_expiry_fixture(struct expiry_fixture *fixture) {
  close(fixture->monitor[0]);
  close(fixture->output[1]);
  if (fixture->output[0] >= 0) close(fixture->output[0]);
}

static void require_expiry_diagnostic(const char *kind, struct expiry_fixture *fixture, struct cleanup_report *report) {
  if (strcmp(kind, "closed") == 0) {
    demand(report->acknowledged && report->diagnostic_errno == EPIPE, "closed output acknowledged EPIPE without caller SIGPIPE");
    return;
  }
  if (strcmp(kind, "writable") != 0) {
    demand(!report->acknowledged && report->diagnostic_ns >= 100000000LL, "unchanged complete100ms diagnostic grace");
    return;
  }
  char message[sizeof(CPU_PROBE_MESSAGE)];
  demand(report->acknowledged && report->diagnostic_errno == 0, "writable output acknowledged success");
  demand(fcntl(fixture->output[0], F_SETFL, O_NONBLOCK) == 0, "diagnostic read guard");
  demand(read(fixture->output[0], message, sizeof(message)) == sizeof(CPU_PROBE_MESSAGE) - 1, "actual CPU diagnostic length");
  demand(memcmp(message, CPU_PROBE_MESSAGE, sizeof(CPU_PROBE_MESSAGE) - 1) == 0, "actual CPU diagnostic bytes");
}

static void probe_real_cpu_expiry(const char *kind, int full, int closed) {
  struct expiry_fixture fixture = prepared_expiry_fixture(kind, full, closed);
  pid_t helper_pid;
  demand(read(fixture.monitor[0], &helper_pid, sizeof(helper_pid)) == sizeof(helper_pid), "actual owned helper PID");
  int status;
  await_cpu_child(fixture.child, &status);
  struct cleanup_report report;
  demand(read(fixture.monitor[0], &report, sizeof(report)) == sizeof(report), "actual cleanup report");
  demand(WIFEXITED(status) && WEXITSTATUS(status) == 2 && report.reaped, "exit2 and helper reaped");
  demand(helper_is_gone(helper_pid), "no surviving helper");
  require_expiry_diagnostic(kind, &fixture, &report);
  demand(fcntl(fixture.output[1], F_GETFL) == fixture.flags && fcntl(fixture.output[1], F_GETNOSIGPIPE) == 0, "exact stdout policies retained");
  printf("{\"probe\":\"cpu_expiry\",\"kind\":\"%s\",\"exit_code\":2,\"reaped\":true,\"diagnostic_ns\":%lld,\"cleanup_ns\":%lld}\n", kind, report.diagnostic_ns, report.cleanup_ns);
  close_expiry_fixture(&fixture);
}

static int late_pipe[2];
static void *concurrent_open(void *argument) {
  (void)argument;
  return pipe(late_pipe) == 0 ? NULL : (void *)1;
}

static void inject_concurrent_descriptor_at_fork(void) {
  pthread_t opener;
  demand(pthread_create(&opener, NULL, concurrent_open, NULL) == 0, "concurrent late open");
  void *result;
  demand(pthread_join(opener, &result) == 0 && result == NULL, "late open completed after sizing");
}

static void probe_late_descriptor(void) {
  size_t capacity = (size_t)cpu_probe_fd_capacity();
  cpu_probe_set_fork_hook(inject_concurrent_descriptor_at_fork);
  demand(cpu_probe_prepare(&active_helper, capacity) == 0, "child actual FD snapshot");
  cpu_probe_set_fork_hook(NULL);
  close(late_pipe[1]);
  demand(fcntl(late_pipe[0], F_SETFL, O_NONBLOCK) == 0, "late reader guard");
  char byte;
  demand(read(late_pipe[0], &byte, 1) == 0, "late inherited writer closed; actual EOF");
  cpu_probe_stop(&active_helper);
  close(late_pipe[0]);
  printf("{\"probe\":\"concurrent_late_descriptor\",\"eof\":true,\"reaped\":true}\n");
}

static int retained_high_writer(int writer, struct rlimit *original) {
  demand(getrlimit(RLIMIT_NOFILE, original) == 0, "original NOFILE");
  struct rlimit raised = *original;
  if (raised.rlim_cur < 1024) raised.rlim_cur = 1024;
  demand(setrlimit(RLIMIT_NOFILE, &raised) == 0, "prepare high descriptor fixture");
  int high = fcntl(writer, F_DUPFD, 512);
  demand(high >= 512, "actual retained high descriptor");
  struct rlimit lowered = raised; lowered.rlim_cur = 64;
  demand(setrlimit(RLIMIT_NOFILE, &lowered) == 0, "lower soft limit below retained FD");
  return high;
}

static void probe_high_descriptor(void) {
  int output[2]; demand(pipe(output) == 0, "high descriptor pipe");
  struct rlimit original;
  int high = retained_high_writer(output[1], &original);
  demand(cpu_probe_prepare(&active_helper, 4096) == 0, "enumerate beyond lowered soft limit");
  demand(active_helper.max_closed_descriptor == high, "actual child inherited and closed the high descriptor");
  close(output[1]); close(high);
  demand(fcntl(output[0], F_SETFL, O_NONBLOCK) == 0, "high reader guard");
  char byte; demand(read(output[0], &byte, 1) == 0, "high inherited writer closed; actual EOF");
  cpu_probe_stop(&active_helper); close(output[0]);
  demand(setrlimit(RLIMIT_NOFILE, &original) == 0, "restore original limit");
  printf("{\"probe\":\"high_fd_after_lowered_limit\",\"eof\":true,\"high_fd\":%d,\"soft_limit\":64}\n", high);
}

static void probe_initialization_failure(void) {
  int before = cpu_probe_fd_count();
  demand(cpu_probe_prepare(&active_helper, 0) == EOVERFLOW, "actual undersized child buffer refuses initialization");
  demand(active_helper.pid == -1 && cpu_probe_fd_count() == before, "initialization failure reaped and closed descriptors");
  printf("{\"probe\":\"initialization_refusal\",\"errno\":%d,\"reaped\":true,\"fds_before\":%d,\"fds_after\":%d}\n", EOVERFLOW, before, cpu_probe_fd_count());
}

static void probe_redirected_stdout(void) {
  int original[2], current[2];
  demand(pipe(original) == 0 && pipe(current) == 0, "redirect pipes");
  int saved = dup(STDOUT_FILENO);
  fflush(stdout);
  demand(dup2(original[1], STDOUT_FILENO) == STDOUT_FILENO, "initial stdout");
  demand(cpu_probe_prepare(&active_helper, 4096) == 0, "prepare before stdout redirect");
  demand(dup2(current[1], STDOUT_FILENO) == STDOUT_FILENO, "redirect current stdout after preparation");
  demand(cpu_probe_send(&active_helper, STDOUT_FILENO) == 0, "send actual current stdout");
  demand(cpu_probe_receive(&active_helper, 100) == 0, "redirected diagnostic completed");
  cpu_probe_stop(&active_helper);
  demand(dup2(saved, STDOUT_FILENO) == STDOUT_FILENO, "restore probe console");
  close(saved); close(original[1]); close(current[1]);
  char observed[sizeof(CPU_PROBE_MESSAGE)];
  demand(read(current[0], observed, sizeof(observed)) == sizeof(CPU_PROBE_MESSAGE) - 1, "new output received exact diagnostic");
  demand(memcmp(observed, CPU_PROBE_MESSAGE, sizeof(CPU_PROBE_MESSAGE) - 1) == 0, "diagnostic bytes");
  demand(read(original[0], observed, 1) == 0, "old output unused and unretained");
  close(original[0]); close(current[0]);
  printf("{\"probe\":\"redirect_after_preparation\",\"current_stdout_received\":true,\"old_stdout_eof\":true}\n");
}

static void probe_failed_private_channel(void) {
  demand(cpu_probe_prepare(&active_helper, 4096) == 0, "prepare failed-channel control");
  demand(kill(active_helper.pid, SIGKILL) == 0, "close helper peer");
  int status;
  demand(waitpid(active_helper.pid, &status, 0) == active_helper.pid, "reap closed peer");
  int failure = cpu_probe_send(&active_helper, STDOUT_FILENO);
  demand(failure == EPIPE, "private socket failure cannot generate caller SIGPIPE");
  active_helper.pid = -1; close(active_helper.control); active_helper.control = -1;
  printf("{\"probe\":\"failed_private_channel\",\"errno\":%d,\"caller_survived\":true}\n", failure);
}

static void probe_failed_relay_channel(void) {
  int channel[2];
  demand(socketpair(AF_UNIX, SOCK_STREAM, 0, channel) == 0, "relay private pair");
  int enabled = 1;
  demand(setsockopt(channel[0], SOL_SOCKET, SO_NOSIGPIPE, &enabled, sizeof(enabled)) == 0, "private relay policy");
  close(channel[1]);
  errno = 0;
  long long request = 1;
  demand(write(channel[0], &request, sizeof(request)) == -1 && errno == EPIPE, "broken relay returns EPIPE");
  close(channel[0]);
  printf("{\"probe\":\"failed_relay_channel\",\"caller_survived\":true}\n");
}

static void abandoned_parent(int output, int inherited_reader, int monitor) {
  close(inherited_reader);
  demand(cpu_probe_prepare(&active_helper, 4096) == 0, "prepare forced-death helper");
  demand(cpu_probe_send(&active_helper, output) == 0, "block helper on full output");
  demand(write(monitor, &active_helper.pid, sizeof(active_helper.pid)) == sizeof(active_helper.pid), "publish actual helper PID");
  for (;;) pause();
}

static void probe_forced_parent_death(void) {
  int output[2], monitor[2];
  demand(pipe(output) == 0 && pipe(monitor) == 0, "forced-death fixtures");
  exhaust_prepared_pipe(output[1], output[0], 1);
  pid_t parent = fork(); demand(parent >= 0, "forced parent");
  if (parent == 0) { close(monitor[0]); abandoned_parent(output[1], output[0], monitor[1]); }
  close(monitor[1]); pid_t helper;
  demand(read(monitor[0], &helper, sizeof(helper)) == sizeof(helper), "forced-death helper identity");
  demand(kill(parent, SIGKILL) == 0, "force caller death");
  int status; demand(waitpid(parent, &status, 0) == parent, "reap forced caller");
  long long start = cpu_probe_clock();
  while (!helper_is_gone(helper)) {
    demand(cpu_probe_clock() - start < 12000000000LL, "parent-death guard removes helper");
    usleep(10000);
  }
  printf("{\"probe\":\"forced_parent_death\",\"helper_gone\":true,\"observed_cleanup_ns\":%lld}\n", cpu_probe_clock() - start);
  close(monitor[0]); close(output[0]); close(output[1]);
}

int main(void) {
  signal(SIGPIPE, SIG_DFL);
  struct sigaction pipe_before, pipe_after;
  struct itimerval timer_before, timer_after;
  demand(sigaction(SIGPIPE, NULL, &pipe_before) == 0, "caller signal snapshot");
  demand(getitimer(ITIMER_REAL, &timer_before) == 0, "caller timer snapshot");
  demand(timer_before.it_value.tv_sec == 0 && timer_before.it_value.tv_usec == 0, "inactive caller timer fixture");
  int descriptors = cpu_probe_fd_count();
  probe_initialization_failure();
  probe_late_descriptor();
  probe_high_descriptor();
  probe_redirected_stdout();
  probe_failed_private_channel();
  probe_failed_relay_channel();
  probe_real_cpu_expiry("writable", 0, 0);
  probe_real_cpu_expiry("closed", 0, 1);
  probe_real_cpu_expiry("full", 1, 0);
  probe_real_cpu_expiry("stalled", 0, 0);
  probe_forced_parent_death();
  demand(cpu_probe_fd_count() == descriptors, "all prototype descriptors closed");
  demand(sigaction(SIGPIPE, NULL, &pipe_after) == 0 && pipe_after.sa_handler == pipe_before.sa_handler
      && pipe_after.sa_flags == pipe_before.sa_flags
      && memcmp(&pipe_after.sa_mask, &pipe_before.sa_mask, sizeof(pipe_before.sa_mask)) == 0, "caller signal disposition retained");
  demand(getitimer(ITIMER_REAL, &timer_after) == 0 && timer_after.it_value.tv_sec == 0
      && timer_after.it_value.tv_usec == 0 && timer_after.it_interval.tv_sec == timer_before.it_interval.tv_sec
      && timer_after.it_interval.tv_usec == timer_before.it_interval.tv_usec, "caller timer retained");
  printf("{\"probe\":\"complete\",\"descriptor_count_preserved\":true}\n");
  return 0;
}
