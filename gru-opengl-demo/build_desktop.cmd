cargo build --release --bin desktop
copy /y ..\target\release\desktop.exe export\opengl.exe
cd export
opengl
pause