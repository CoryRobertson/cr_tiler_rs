use std::error::Error;
use vergen::EmitBuilder;
#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(target_os = "windows")] // conditionally set icon of program on windows
    {
        WindowsResource::new()
            .set_icon("./assets/program_icon.ico")
            .compile()?;
    }

    EmitBuilder::builder().all_git().emit()?;

    Ok(())
}
