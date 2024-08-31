
# 在 Windows 使用这个脚本调用 ms_build.ps1 来编译动态库和静态库
# On Windows, use this script to call ms_build.ps1 to compile dynamic and static libraries
if (-Not (Test-Path -Path "build")) {
    New-Item -Path "build" -ItemType Directory
}

Set-Location .\build

cmake ..

Set-Location ..

.\ms_build.ps1 .\build\dylib_for_rust.sln /p:Configuration=Release

$files = Get-ChildItem -Path lib_build\Release -File

foreach ($file in $files) {
    Move-Item -Path $file.FullName -Destination lib_build -Force
}



