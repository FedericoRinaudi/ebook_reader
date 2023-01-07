mod algorithms;
mod app;
mod book;
mod bookcase;
mod controllers;
mod delegate;
mod formatters;
mod ocr;
mod utilities;
mod view;
mod widgets;

use druid::{AppLauncher, WindowDesc};
use view::view::WINDOW_TITLE;

use crate::app::ApplicationState;
use crate::book::page_element::ContentType;
use crate::book::Book;
use crate::view::render::build_main_view;
use delegate::Delegate;

fn main() {
    let app = ApplicationState::new();

    // Describe the main Window
    let main_window = WindowDesc::new(build_main_view())
        .title(WINDOW_TITLE)
        .window_size(app.view.get_window_size_home());

    // Start the Application
    AppLauncher::with_window(main_window)
        .delegate(Delegate {})
        .launch(app)
        .expect("Failed to launch application");
}
