//! Browser launching for interactive auth.

/// Open a URL in the user's default browser.
pub(crate) fn open_browser(url: &str) -> Result<(), String> {
    let candidates: Vec<(&str, Vec<&str>)> = {
        #[cfg(target_os = "macos")]
        {
            vec![("open", vec![url])]
        }

        #[cfg(target_os = "linux")]
        {
            vec![("xdg-open", vec![url])]
        }

        #[cfg(target_os = "windows")]
        {
            vec![("cmd", vec!["/C", "start", "", url])]
        }
    };

    for (command, args) in candidates {
        if let Ok(status) = std::process::Command::new(command).args(args).status()
            && status.success()
        {
            return Ok(());
        }
    }

    Err(format!(
        "Failed to open a browser automatically. Open this URL manually: {url}"
    ))
}
