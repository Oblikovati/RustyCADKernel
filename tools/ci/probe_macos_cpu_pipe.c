/* Independent Darwin syscall observations for product Bug289; no kernel code is linked. */
#define _DARWIN_C_SOURCE 1
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static void require_success(int result, const char *operation) {
  if (result >= 0) return;
  perror(operation);
  exit(70);
}

static void probe_write_metadata(void) {
  int descriptors[2];
  require_success(pipe(descriptors), "metadata pipe");
  int before = fcntl(descriptors[1], F_GETFL);
  require_success(before, "metadata flags before");
  require_success((int)write(descriptors[1], "x", 1), "metadata write");
  int after = fcntl(descriptors[1], F_GETFL);
  require_success(after, "metadata flags after");
  printf("{\"probe\":\"write_metadata\",\"before\":%d,\"after\":%d,\"xor\":%d}\n", before, after, before ^ after);
  close(descriptors[0]);
  close(descriptors[1]);
}

static void probe_alias(int original, int alias, const char *kind) {
  require_success(alias, kind);
  int before = fcntl(original, F_GETNOSIGPIPE);
  int flags_before = fcntl(original, F_GETFL);
  require_success(fcntl(alias, F_SETNOSIGPIPE, 1), "set alias nosigpipe");
  int after = fcntl(original, F_GETNOSIGPIPE);
  int flags_after = fcntl(original, F_GETFL);
  printf("{\"probe\":\"private_handle\",\"kind\":\"%s\",\"original_nosigpipe_before\":%d,\"original_nosigpipe_after\":%d,\"original_flags_before\":%d,\"original_flags_after\":%d}\n", kind, before, after, flags_before, flags_after);
  require_success(fcntl(alias, F_SETNOSIGPIPE, before), "restore alias nosigpipe");
  close(alias);
}

static void probe_private_handles(void) {
  int descriptors[2];
  require_success(pipe(descriptors), "alias pipe");
  probe_alias(descriptors[1], dup(descriptors[1]), "dup");
  char pathname[64];
  snprintf(pathname, sizeof(pathname), "/dev/fd/%d", descriptors[1]);
  int reopened = open(pathname, O_WRONLY);
  if (reopened >= 0) probe_alias(descriptors[1], reopened, "open-dev-fd");
  else printf("{\"probe\":\"open-dev-fd-refused\",\"errno\":%d}\n", errno);
  close(descriptors[0]);
  close(descriptors[1]);
}

static void *masked_pipe_write(void *argument) {
  sigset_t blocked;
  sigemptyset(&blocked);
  sigaddset(&blocked, SIGPIPE);
  int status = pthread_sigmask(SIG_BLOCK, &blocked, NULL);
  if (status != 0) _exit(71);
  int descriptor = *(int *)argument;
  (void)write(descriptor, "x", 1);
  return NULL;
}

static void run_masked_child(void) {
  signal(SIGPIPE, SIG_DFL);
  int descriptors[2];
  require_success(pipe(descriptors), "masked pipe");
  close(descriptors[0]);
  pthread_t worker;
  if (pthread_create(&worker, NULL, masked_pipe_write, &descriptors[1]) != 0) _exit(72);
  pthread_join(worker, NULL);
  struct timespec delivery = {0, 100000000};
  nanosleep(&delivery, NULL);
  _exit(0);
}

static void probe_process_signal(void) {
  pid_t child = fork();
  require_success((int)child, "probe fork");
  if (child == 0) run_masked_child();
  int status;
  require_success((int)waitpid(child, &status, 0), "probe reap");
  printf("{\"probe\":\"masked_thread_closed_pipe\",\"signal\":%d,\"exit_code\":%d}\n", WIFSIGNALED(status) ? WTERMSIG(status) : 0, WIFEXITED(status) ? WEXITSTATUS(status) : -1);
}

int main(void) {
  probe_write_metadata();
  probe_private_handles();
  probe_process_signal();
  return 0;
}
