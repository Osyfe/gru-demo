set RUSTFLAGS=-C lto=yes -C embed-bitcode=yes
cargo apk run --release
copy /y ..\target\release\apk\gru-opengl-demo.apk export\opengl.apk
adb logcat RustStdoutStderr:D *:S