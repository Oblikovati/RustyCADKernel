# Build the pinned Tcl/Tk 8.6 sources with the same MSVC ABI as the Rust host.
$ErrorActionPreference = 'Stop'
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $installation) { throw 'Visual Studio installation missing; expected x64 C++ build tools' }
$developer = Join-Path $installation 'Common7\Tools\VsDevCmd.bat'
$prefix = Join-Path $env:RUNNER_TEMP 'martensite-tcl'
$tclSource = Join-Path $env:GITHUB_WORKSPACE 'target\tcl-source'
$tkSource = Join-Path $env:GITHUB_WORKSPACE 'target\tk-source'
foreach ($source in @($tclSource, $tkSource)) {
    $command = "call `"$developer`" -arch=x64 -host_arch=x64 && cd /d `"$source\win`" && nmake -nologo -f makefile.vc release install MACHINE=AMD64 `"INSTALLDIR=$prefix`" `"TCLDIR=$tclSource`""
    & cmd.exe /d /c $command
    if ($LASTEXITCODE -ne 0) { throw "Tcl/Tk build failed for $source with exit $LASTEXITCODE; expected zero" }
}
& choco install pkgconfiglite --version=0.28 --yes --no-progress
if ($LASTEXITCODE -ne 0) { throw "pkgconfiglite install returned $LASTEXITCODE; expected zero" }
$pcDirectory = Join-Path $prefix 'lib\pkgconfig'
New-Item -ItemType Directory -Force $pcDirectory | Out-Null
foreach ($module in @('tcl', 'tk')) {
    $importLibrary = Join-Path $prefix "lib\${module}86t.lib"
    if (-not (Test-Path $importLibrary)) { throw "Missing $importLibrary; expected threaded MSVC import library" }
    # Windows makefiles install no pkg-config metadata; describe the actual built ABI.
    @"
prefix=$($prefix.Replace('\', '/'))
Name: $module
Description: Martensite CI Tcl/Tk 8.6 MSVC prerequisites
Version: 8.6.18
Libs: -L`${prefix}/lib -l${module}86t
Cflags: -I`${prefix}/include
"@ | Set-Content -Encoding utf8 (Join-Path $pcDirectory "${module}8.6.pc")
}
"PKG_CONFIG_PATH=$pcDirectory" | Out-File -Append -Encoding utf8 $env:GITHUB_ENV
"TCL_LIBRARY=$prefix\lib\tcl8.6" | Out-File -Append -Encoding utf8 $env:GITHUB_ENV
"TK_LIBRARY=$prefix\lib\tk8.6" | Out-File -Append -Encoding utf8 $env:GITHUB_ENV
"$prefix\bin" | Out-File -Append -Encoding utf8 $env:GITHUB_PATH
$env:PKG_CONFIG_PATH = $pcDirectory
& pkg-config --modversion tcl8.6 tk8.6
if ($LASTEXITCODE -ne 0) { throw "Tcl/Tk pkg-config returned $LASTEXITCODE; expected zero" }
