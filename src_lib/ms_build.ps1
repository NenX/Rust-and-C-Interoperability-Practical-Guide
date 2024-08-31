# 在 windows 系统调用 MSBuild 编译动态库和静态库
# Call MSBuild on windows system to compile dynamic and static libraries
param(
    [switch]$Help
)

if ($Help) {
    Write-Host "Usage: .\ms_build.ps1 [parameters]"
    return
}


$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsInstallation = & $vsWhere -latest -property installationPath
    $msbuild = Join-Path $vsInstallation "MSBuild\Current\Bin\MSBuild.exe"
    if (Test-Path $msbuild) {
        & $msbuild $sln $Args
    }
}