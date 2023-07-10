use std::io;
#[cfg(target_os = "windows")]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(target_os = "windows")] // conditionally set icon of program on windows
    {
        WindowsResource::new()
            .set_icon("./assets/program_icon.ico")
            .compile()?;
    }
    Ok(())
}
