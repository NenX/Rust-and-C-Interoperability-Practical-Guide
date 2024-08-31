Set-Location src_lib
.\build.ps1

Set-Location ..

cargo clean
cargo r
