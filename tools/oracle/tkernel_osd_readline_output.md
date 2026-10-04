# OSD_File output-reference `ReadLine` probe

`OSD_File::ReadLine(buffer, count, countOut)` on the POSIX branch updates buffer/count on a line
and EOF, and leaves both output arguments unchanged when `fgets` fails. The fixture checks a
line, EOF, then opens a second `FILE*`, closes its descriptor, and invokes the error branch. It
resets the recorded error and closes the second `OSD_File` before exiting.

Pinned source is OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`. The tested native library is
`libTKernel.so.8.1.0`, SHA-256
`6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`.
The native source fixture SHA-256 is
`07853ce84daf225ce0c4a765158e34a7e13be22dd62784b3e3c4481e23ed027d`; the built probe SHA-256
is `f8ae6016e4f839f505f8410e3a6bdf2fa80a7149faa2705a472bf36e53dd8dbd`.

Build and run against the current OCCT model build:

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_osd_readline_output_probe.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/tkernel_osd_readline_output_probe
printf 'read-error-probe\n' > /tmp/rustycad-readline-error-input.txt
/tmp/tkernel_osd_readline_output_probe /tmp/rustycad-readline-error-input.txt
```

Expected output is in `tkernel_osd_readline_output.expected`. The first line reads 17 bytes;
the next call clears the buffer, writes count 0, and marks EOF. The final line means the buffer
remains `keep`, the count remains `73`, `Failed()` is true, and `Error()` is 20 (`EBADF`
translated by `OSD_Error::Perror`). Rust retains raw errno 9 in `osd_error()`.
Windows has a different defined `ReadLine` error path that clears the buffer and writes a zero
count; this probe does not establish Windows parity.
