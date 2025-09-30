// Windows process helper to prevent console window flashing

pub trait ProcessCommandExt {
    fn no_window(&mut self) -> &mut Self;
}

#[cfg(target_os = "windows")]
impl ProcessCommandExt for std::process::Command {
    fn no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        self.creation_flags(CREATE_NO_WINDOW)
    }
}

#[cfg(not(target_os = "windows"))]
impl ProcessCommandExt for std::process::Command {
    fn no_window(&mut self) -> &mut Self {
        self
    }
}

#[cfg(target_os = "windows")]
impl ProcessCommandExt for tokio::process::Command {
    fn no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        self.creation_flags(CREATE_NO_WINDOW)
    }
}

#[cfg(not(target_os = "windows"))]
impl ProcessCommandExt for tokio::process::Command {
    fn no_window(&mut self) -> &mut Self {
        self
    }
}