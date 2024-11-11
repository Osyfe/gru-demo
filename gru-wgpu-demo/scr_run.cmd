cargo build
copy /y ..\target\debug\desktop.exe export\gru_wgpu_demo.exe
cd export
gru_wgpu_demo
cd ..
