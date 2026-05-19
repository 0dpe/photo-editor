mod app;
mod render;

use app::PhotoEditorApp;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_title("Photo Editor"),
            ..Default::default()
        };
        if let Err(err) = eframe::run_native(
            "Photo Editor",
            native_options,
            Box::new(|cc| Ok(Box::new(PhotoEditorApp::new(cc)))),
        ) {
            log::error!("eframe exited with error: {err}");
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        use eframe::wasm_bindgen::JsCast as _;

        eframe::WebLogger::init(log::LevelFilter::Info).ok();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");

            let canvas = document
                .get_element_by_id("rust-insert-wasm")
                .expect("Failed to find canvas #rust-insert-wasm")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("Canvas element was not an HtmlCanvasElement");

            let start_result = eframe::WebRunner::new()
                .start(
                    canvas,
                    web_options,
                    Box::new(|cc| Ok(Box::new(PhotoEditorApp::new(cc)))),
                )
                .await;

            if let Some(loading_text) = document.get_element_by_id("loading_text") {
                match start_result {
                    Ok(()) => {
                        loading_text.remove();
                    }
                    Err(err) => {
                        loading_text.set_inner_html(
                            "<p><strong>The app failed to start.</strong> See the browser console for details.</p>",
                        );
                        panic!("Failed to start eframe: {err:?}");
                    }
                }
            }
        });
    }
}
