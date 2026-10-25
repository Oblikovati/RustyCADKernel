#define _DARWIN_C_SOURCE 1
#include "probe_macos_cpu_helper.h"
#include <errno.h>
#include <libproc.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/proc_info.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static volatile sig_atomic_t helper_parent_pid;
static void (*prototype_before_fork)(void);

void cpu_probe_set_fork_hook(void (*hook)(void)) {
  prototype_before_fork = hook;
}

long long cpu_probe_clock(void) {
  struct timespec value;
  if (clock_gettime(CLOCK_MONOTONIC, &value) != 0) return -1;
  return (long long)value.tv_sec * 1000000000LL + value.tv_nsec;
}

static int checked_fd_info_count(int bytes) {
  if (bytes <= 0 || bytes % (int)sizeof(struct proc_fdinfo) != 0) {
    errno = EIO;
    return -1;
  }
  return bytes / (int)sizeof(struct proc_fdinfo);
}

int cpu_probe_fd_capacity(void) {
  int bytes = proc_pidinfo(getpid(), PROC_PIDLISTFDS, 0, NULL, 0);
  return checked_fd_info_count(bytes);
}

int cpu_probe_fd_count(void) {
  struct proc_fdinfo entries[4096];
  int bytes = proc_pidinfo(getpid(), PROC_PIDLISTFDS, 0, entries, sizeof(entries));
  return checked_fd_info_count(bytes);
}

static void helper_parent_gone(int signal_number) {
  (void)signal_number;
  if (getppid() != helper_parent_pid) _exit(0);
}

static int helper_parent_guard(pid_t expected_parent) {
  helper_parent_pid = expected_parent;
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  action.sa_handler = helper_parent_gone;
  if (sigemptyset(&action.sa_mask) != 0 || sigaction(SIGALRM, &action, NULL) != 0) return errno;
  sigset_t blocked;
  if (sigfillset(&blocked) != 0 || sigdelset(&blocked, SIGALRM) != 0) return errno;
  if (sigprocmask(SIG_SETMASK, &blocked, NULL) != 0) return errno;
  struct itimerval guard = {{0, 100000}, {0, 100000}};
  if (setitimer(ITIMER_REAL, &guard, NULL) != 0) return errno;
  helper_parent_gone(0);
  return 0;
}

static int helper_close_inherited(int retained, int output, struct proc_fdinfo *entries, size_t capacity, int *maximum) {
  int bytes = proc_pidinfo(getpid(), PROC_PIDLISTFDS, 0, NULL, 0);
  if (checked_fd_info_count(bytes) < 0) return errno;
  if ((size_t)bytes > capacity * sizeof(*entries)) return EOVERFLOW;
  int used = proc_pidinfo(getpid(), PROC_PIDLISTFDS, 0, entries, (int)(capacity * sizeof(*entries)));
  if (used <= 0) return errno ? errno : EIO;
  if ((size_t)used > capacity * sizeof(*entries) || (size_t)used % sizeof(*entries)) return EIO;
  size_t count = (size_t)used / sizeof(*entries);
  for (size_t i = 0; i < count; ++i) {
    if (entries[i].proc_fd != retained && entries[i].proc_fd != output) {
      if (entries[i].proc_fd > *maximum) *maximum = entries[i].proc_fd;
      close(entries[i].proc_fd);
    }
  }
  return 0;
}

static int helper_write_all(int descriptor, const char *bytes, size_t length) {
  while (length > 0) {
    ssize_t count = write(descriptor, bytes, length);
    if (count < 0 && errno == EINTR) continue;
    if (count <= 0) return count == 0 ? EIO : errno;
    bytes += count;
    length -= (size_t)count;
  }
  return 0;
}

static int helper_receive_output(int control, char *bytes, size_t *length) {
  union { struct cmsghdr alignment; char bytes[CMSG_SPACE(sizeof(int))]; } ancillary;
  struct iovec region = {bytes, 64};
  struct msghdr packet;
  memset(&packet, 0, sizeof(packet));
  memset(&ancillary, 0, sizeof(ancillary));
  packet.msg_iov = &region;
  packet.msg_iovlen = 1;
  packet.msg_control = ancillary.bytes;
  packet.msg_controllen = sizeof(ancillary);
  ssize_t count;
  do { count = recvmsg(control, &packet, 0); } while (count < 0 && errno == EINTR);
  if (count <= 0) return -1;
  struct cmsghdr *rights = CMSG_FIRSTHDR(&packet);
  if (!rights || rights->cmsg_level != SOL_SOCKET || rights->cmsg_type != SCM_RIGHTS) return -1;
  if (rights->cmsg_len != CMSG_LEN(sizeof(int)) || (packet.msg_flags & MSG_CTRUNC)) return -1;
  int output;
  memcpy(&output, CMSG_DATA(rights), sizeof(output));
  *length = (size_t)count;
  return output;
}

static void helper_child(int control, pid_t expected_parent, struct proc_fdinfo *entries, size_t capacity) {
  int readiness[2] = {helper_parent_guard(expected_parent), -1};
  if (!readiness[0]) readiness[0] = helper_close_inherited(control, -1, entries, capacity, &readiness[1]);
  (void)helper_write_all(control, (const char *)readiness, sizeof(readiness));
  int failure = readiness[0];
  if (failure) _exit(1);
  char message[64];
  size_t length = 0;
  int output = helper_receive_output(control, message, &length);
  if (output < 0) _exit(0);
  failure = helper_write_all(output, message, length);
  close(output);
  (void)helper_write_all(control, (const char *)&failure, sizeof(failure));
  for (;;) pause();
}

static void helper_preflight(void) {
  (void)cpu_probe_fd_capacity(); /* Bind libproc/__proc_info before fork. */
  struct itimerval unused = {{0, 0}, {0, 0}};
  (void)setitimer(-1, &unused, NULL); /* Bind without changing any caller timer. */
  struct sigaction action;
  (void)sigaction(SIGALRM, NULL, &action);
  (void)getppid();
}

void cpu_probe_stop(struct cpu_helper_probe *helper) {
  if (helper->pid <= 0) return;
  int status;
  pid_t result;
  do { result = waitpid(helper->pid, &status, WNOHANG); } while (result < 0 && errno == EINTR);
  if (result == 0) {
    (void)kill(helper->pid, SIGKILL);
    do { result = waitpid(helper->pid, &status, 0); } while (result < 0 && errno == EINTR);
  }
  close(helper->control);
  helper->pid = -1;
  helper->control = -1;
}

int cpu_probe_prepare(struct cpu_helper_probe *helper, size_t capacity) {
  int pair[2];
  struct proc_fdinfo entries[4096];
  if (capacity > 4096) return EOVERFLOW;
  helper_preflight();
  if (socketpair(AF_UNIX, SOCK_STREAM, 0, pair) != 0) return errno;
  int enabled = 1;
  if (setsockopt(pair[0], SOL_SOCKET, SO_NOSIGPIPE, &enabled, sizeof(enabled)) != 0) {
    int failure = errno; close(pair[0]); close(pair[1]); return failure;
  }
  pid_t expected_parent = getpid();
  if (prototype_before_fork) prototype_before_fork();
  pid_t child = fork();
  if (child == 0) helper_child(pair[1], expected_parent, entries, capacity);
  if (child < 0) { int failure = errno; close(pair[0]); close(pair[1]); return failure; }
  close(pair[1]);
  helper->pid = child;
  helper->control = pair[0];
  struct pollfd pending = {helper->control, POLLIN, 0};
  int readiness[2] = {0, -1};
  int failure = poll(&pending, 1, 12000) > 0 ? 0 : ETIMEDOUT;
  if (!failure) {
    ssize_t used = read(helper->control, readiness, sizeof(readiness));
    failure = used == (ssize_t)sizeof(readiness) ? readiness[0] : EPIPE;
  }
  helper->max_closed_descriptor = readiness[1];
  if (failure) cpu_probe_stop(helper);
  return failure;
}

int cpu_probe_send(struct cpu_helper_probe *helper, int stdout_descriptor) {
  char message[] = CPU_PROBE_MESSAGE;
  union { struct cmsghdr alignment; char bytes[CMSG_SPACE(sizeof(int))]; } ancillary;
  struct iovec region = {message, sizeof(message) - 1};
  struct msghdr packet;
  memset(&packet, 0, sizeof(packet));
  memset(&ancillary, 0, sizeof(ancillary));
  packet.msg_iov = &region;
  packet.msg_iovlen = 1;
  packet.msg_control = ancillary.bytes;
  packet.msg_controllen = sizeof(ancillary.bytes);
  struct cmsghdr *rights = CMSG_FIRSTHDR(&packet);
  rights->cmsg_level = SOL_SOCKET;
  rights->cmsg_type = SCM_RIGHTS;
  rights->cmsg_len = CMSG_LEN(sizeof(int));
  memcpy(CMSG_DATA(rights), &stdout_descriptor, sizeof(stdout_descriptor));
  ssize_t result = sendmsg(helper->control, &packet, MSG_DONTWAIT);
  if (result == (ssize_t)sizeof(message) - 1) return 0;
  return result >= 0 ? EIO : errno;
}

int cpu_probe_receive(struct cpu_helper_probe *helper, int milliseconds) {
  struct pollfd readiness = {helper->control, POLLIN, 0};
  int result;
  do { result = poll(&readiness, 1, milliseconds); } while (result < 0 && errno == EINTR);
  if (result <= 0) return result == 0 ? ETIMEDOUT : errno;
  int status = 0;
  ssize_t count = read(helper->control, &status, sizeof(status));
  return count == (ssize_t)sizeof(status) ? status : EPIPE;
}

static long long bounded_deadline;
static pid_t bounded_parent;

static void bounded_output_alarm(int signal_number) {
  (void)signal_number;
  long long now = cpu_probe_clock();
  if (getppid() != bounded_parent || now < 0 || now >= bounded_deadline) _exit(0);
}

static int arm_bounded_timer(long long remaining) {
  struct itimerval timer = {{0, 1000}, {remaining / 1000000000LL, (remaining % 1000000000LL + 999) / 1000}};
  if (timer.it_value.tv_usec == 1000000) { ++timer.it_value.tv_sec; timer.it_value.tv_usec = 0; }
  return setitimer(ITIMER_REAL, &timer, NULL) == 0 ? 0 : errno;
}

static int bounded_child_timer(pid_t parent, long long deadline) {
  bounded_parent = parent;
  bounded_deadline = deadline;
  struct sigaction action;
  memset(&action, 0, sizeof(action));
  action.sa_handler = bounded_output_alarm;
  sigemptyset(&action.sa_mask);
  if (sigaction(SIGALRM, &action, NULL) != 0) return errno;
  sigset_t blocked;
  sigfillset(&blocked);
  sigdelset(&blocked, SIGALRM);
  if (sigprocmask(SIG_SETMASK, &blocked, NULL) != 0) return errno;
  long long now = cpu_probe_clock();
  long long remaining = deadline - now;
  if (getppid() != parent || now < 0 || remaining <= 0) return ETIMEDOUT;
  return arm_bounded_timer(remaining);
}

static void bounded_output_child(int output, int acknowledgement, pid_t parent,
    long long deadline, struct proc_fdinfo *entries, size_t capacity) {
  int maximum = -1;
  int failure = bounded_child_timer(parent, deadline);
  if (!failure) failure = helper_close_inherited(acknowledgement, output, entries, capacity, &maximum);
  if (!failure) failure = helper_write_all(output, CPU_PROBE_MESSAGE, sizeof(CPU_PROBE_MESSAGE) - 1);
  (void)helper_write_all(acknowledgement, (const char *)&failure, sizeof(failure));
  _exit(0);
}

static void wait_before_guard_fault(int *gate) {
  if (!gate) return;
  close(gate[1]);
  char release;
  while (read(gate[0], &release, 1) < 0 && errno == EINTR) {}
  close(gate[0]);
}

pid_t cpu_probe_spawn_bounded(int output, int acknowledgement, long long deadline, size_t capacity, int *gate) {
  struct proc_fdinfo entries[4096];
  if (capacity > 4096) { errno = EOVERFLOW; return -1; }
  helper_preflight();
  pid_t parent = getpid();
  /* Public Darwin vfork is fork without user atfork handlers since macOS12.
     This deprecated diagnostic experiment is never a generic POSIX vfork use. */
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
  pid_t child = vfork();
#pragma clang diagnostic pop
  if (child == 0) {
    wait_before_guard_fault(gate);
    bounded_output_child(output, acknowledgement, parent, deadline, entries, capacity);
  }
  return child;
}
