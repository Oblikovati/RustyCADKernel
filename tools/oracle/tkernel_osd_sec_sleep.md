# `OSD::SecSleep` and `OSD::MilliSecSleep` interrupted-call oracle

The pinned Unix implementations compute `seconds * 1000 * 1000` and
`milliseconds * 1000` as `int`, call `usleep`, and ignore its return. The C++ probe arms a
100 ms timer before each call; the defined positive `SecSleep(30)` and `MilliSecSleep(30000)`
return when the signal interrupts `usleep` (`errno=EINTR`). An exploratory `SecSleep(-1)`
sample also returns, but that value is outside the POSIX `useconds_t` range and is recorded
only as pinned glibc behavior; integer overflow cases are not claimed.

The Rust regression repeatedly targets SIGALRM to the sleeping thread until it returns,
covering both seconds and milliseconds calls. Each case runs in a child test process; the
parent kills that child after two seconds, so an implementation that retries interruptions
cannot hang the suite. Injected `Sleeper` implementations retain the duration callback.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The source functions are `FoundationClasses/TKernel/OSD/OSD.cxx` (`OSD::SecSleep` and
`OSD::MilliSecSleep`).
Expected output is in `tkernel_osd_sec_sleep.expected`.

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_osd_sec_sleep.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-osd-sec-sleep
timeout 2s /tmp/tkernel-osd-sec-sleep
```

The change only specializes the native `Sleeper` seconds and leaves custom duration-based
sleepers unchanged. Non-Unix platform runtime behavior is not part of this Linux probe.
