use tauri::WebviewWindow;

/// Opens WebView2's system print dialog instead of the browser preview.
///
/// The browser preview always adds the document URL, date and page number as a
/// header/footer, which must not appear on an invoice or receipt. The system
/// dialog prints the page as it is laid out, without those additions.
#[tauri::command]
pub fn show_print_dialog(window: WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::sync::{Arc, Mutex};
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            ICoreWebView2_16, COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM,
        };
        use windows_core::Interface;

        let failure: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let captured = Arc::clone(&failure);

        window
            .with_webview(move |webview| unsafe {
                let controller = webview.controller();
                let core = match controller.CoreWebView2() {
                    Ok(core) => core,
                    Err(error) => {
                        *captured.lock().unwrap() = Some(error.to_string());
                        return;
                    }
                };
                let core = match core.cast::<ICoreWebView2_16>() {
                    Ok(core) => core,
                    Err(error) => {
                        *captured.lock().unwrap() = Some(error.to_string());
                        return;
                    }
                };
                if let Err(error) = core.ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM) {
                    *captured.lock().unwrap() = Some(error.to_string());
                }
            })
            .map_err(|error| error.to_string())?;

        if let Some(message) = failure.lock().unwrap().take() {
            return Err(message);
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = window;
        Err("The system print dialog is only available on Windows".to_string())
    }
}
