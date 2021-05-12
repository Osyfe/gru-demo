cargo apk run --release
copy /y ..\target\release\apk\opengl.apk export\opengl.apk
adb logcat RustStdoutStderr:D *:S