cargo apk run --release
copy ..\target\release\apk\gru-opengl-demo.apk export\android\opengl.apk
adb logcat RustStdoutStderr:D '*:S'