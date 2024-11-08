use std::error::Error;
#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(target_os = "windows")] // conditionally set icon of program on windows
    {
        WindowsResource::new()
            .set_icon("./assets/program_icon.ico")
            .compile()?;
    }
    let build = vergen::BuildBuilder::all_build()?;
    let git2 = vergen_git2::Git2Builder::all_git()?;

    vergen::Emitter::new()
        .add_instructions(&build)?
        .add_instructions(&git2)?
        .emit()?;
    
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
