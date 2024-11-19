use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>>
{
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/wgsl");
    
    let mut panic = false;
    let mut compiler = naga::front::wgsl::Frontend::new();
    for file in fs::read_dir("src/wgsl")?
    {
        let path = file?.path();
        let src = std::fs::read_to_string(&path)?;
        if let Err(err) = compiler.parse(&src)
        {
            err.emit_to_stderr_with_path(&src, &path);
            panic = true;
        }
    }
    
    if panic { Err(Box::<dyn Error>::from("^")) }
    else { Ok(()) }
}
