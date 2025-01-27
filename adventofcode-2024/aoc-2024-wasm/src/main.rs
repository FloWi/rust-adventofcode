mod app;
pub mod components;
mod run_tasks_component;

use app::*;
use leptos::{logging, mount};

pub fn main() {
    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    mount::mount_to_body(App);
}
