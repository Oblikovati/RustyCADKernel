# FreeType stays an external library; build the pinned source with the Rust host's MSVC ABI.
$ErrorActionPreference = 'Stop'
$source = Join-Path $env:GITHUB_WORKSPACE 'target\freetype-source'
$build = Join-Path $env:RUNNER_TEMP 'martensite-freetype-build'
$prefix = Join-Path $env:RUNNER_TEMP 'martensite-freetype'
& cmake -S $source -B $build -A x64 -DBUILD_SHARED_LIBS=ON -DFT_DISABLE_ZLIB=ON -DFT_DISABLE_BZIP2=ON -DFT_DISABLE_PNG=ON -DFT_DISABLE_HARFBUZZ=ON -DFT_DISABLE_BROTLI=ON "-DCMAKE_INSTALL_PREFIX=$prefix"
if ($LASTEXITCODE -ne 0) { throw "FreeType configuration returned $LASTEXITCODE; expected zero" }
& cmake --build $build --config Release --parallel 2
if ($LASTEXITCODE -ne 0) { throw "FreeType build returned $LASTEXITCODE; expected zero" }
& cmake --install $build --config Release
if ($LASTEXITCODE -ne 0) { throw "FreeType installation returned $LASTEXITCODE; expected zero" }
$bin = Join-Path $prefix 'bin'
$dll = Join-Path $bin 'freetype.dll'
if (-not (Test-Path $dll)) { throw "Missing $dll; expected installed MSVC FreeType DLL" }
# Disabling external zlib still uses FreeType's bundled inflate implementation for WOFF.
$bin | Out-File -Append -Encoding utf8 $env:GITHUB_PATH
