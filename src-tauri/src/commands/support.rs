/// Opens the developer's LinkedIn profile in the system browser.
///
/// The URL is fixed here rather than taken from the frontend so the app cannot
/// be used to launch arbitrary pages.
#[tauri::command]
pub fn open_developer_profile() -> Result<(), String> {
    let url = "https://www.linkedin.com/in/libemurg";

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    #[cfg(not(windows))]
    {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        std::process::Command::new(opener)
            .arg(url)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}
