use std::path::Path;

use poe_crafting::{
    init,
    item_state::{ItemState, Rarity},
    ui::{
        Page,
        pages::{currency_sim, item_builder, strategy_sim, ui_debug},
    },
};

struct MyEguiApp {
    base_item: ItemState,
    page: Page,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            base_item: ItemState {
                base_type: "Amulet".to_string(),
                item_level: 100,
                rarity: Rarity::Normal,
                mods: vec![],
            },
            page: Page::ItemBuilder,
        }
    }
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
        // let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
        init(data_root);

        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for page in Page::all() {
                    if ui.button(page.name()).clicked() {
                        self.page = page;
                    }
                }
            })
        });

        match &mut self.page {
            Page::ItemBuilder => {
                item_builder::show_page(ctx, &mut self.base_item);
            }
            Page::CraftProbability { .. } => {
                currency_sim::show_page(&mut self.page, ctx, &self.base_item);
            }
            Page::StrategyBuilder { .. } => {
                strategy_sim::show_page(&mut self.page, ctx, &mut self.base_item);
            }
            Page::UIDebug(state) => {
                ui_debug::show_page(ctx, state);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(crate::MyEguiApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
