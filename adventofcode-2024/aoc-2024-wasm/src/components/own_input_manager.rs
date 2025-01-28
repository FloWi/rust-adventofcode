use crate::components::lib::AocInput;
use crate::components::{parse_day_from_str, read_file_content, styled_button, AocDayInput};
use codee::string::JsonSerdeCodec;
use futures::FutureExt;
use itertools::Itertools;
use leptos::html::{Div, ElementChild};
use leptos::leptos_dom::log;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::{component, ev, view, IntoView};
use leptos_use::docs::BooleanDisplay;
use leptos_use::storage::use_local_storage;
use leptos_use::{use_drop_zone_with_options, UseDropZoneOptions, UseDropZoneReturn};
use send_wrapper::SendWrapper;
use std::ops::Not;
use web_sys::File;

fn is_valid_file(file: &File) -> bool {
    parse_day_from_str(&file.name()).is_some()
}

#[component]
pub fn OwnInputManager(local_storage_key: String) -> impl IntoView {
    let (read, write, delete_fn) = use_local_storage::<AocInput, JsonSerdeCodec>(local_storage_key.clone());

    let (dropped, set_dropped) = signal(false);

    let drop_zone_el = NodeRef::<Div>::new();

    let UseDropZoneReturn {
        is_over_drop_zone,
        files: dropped_files,
    } = use_drop_zone_with_options(
        drop_zone_el,
        UseDropZoneOptions::default().on_drop(move |_| set_dropped.set(true)).on_enter(move |_| set_dropped.set(false)),
    );

    let new_file_divs = move || {
        dropped_files
            .get()
            .iter()
            .map(|file| {
                view! {
                    <div class="w-200px bg-black-200/10 ma-2 pa-6 border">
                        <p>
                            <span>"Name:"</span>
                            <span>{file.name()}</span>
                        </p>
                        <p>
                            <span>"Size:"</span>
                            <span>{file.size()}</span>
                        </p>
                        <p>
                            <span>"Type:"</span>
                            <span>{file.type_()}</span>
                        </p>
                    </div>
                }
            })
            .collect_view()
    };

    let store_files_button = move || {
        dropped_files.get().is_empty().not().then_some(
            styled_button().child("Store files in localstorage").on(ev::click, move |_| spawn_local(store_files_in_localstorage(dropped_files.get(), write))),
        )
    };

    let delete_files_button = move || {
        let delete_fn_cloned = delete_fn.clone();
        read.get().days.is_empty().not().then_some(styled_button().child("Delete files from localstorage").on(ev::click, move |_| delete_fn_cloned()))
    };

    view! {
        <div class="flex">
            <div class="w-full h-auto relative">
                <p>
                    "Drop new files into dropZone. The files must be txt files and have the day as a filename (any \\d+ in there will do)."
                </p>
                <p>
                    {format!(
                        "The files will be stored in localstorage under the key '{local_storage_key}' and won't be uploaded. ",
                    )}
                </p>
                <p>"e.g. day-01.txt, day-02.txt"</p>
                <div
                    node_ref=drop_zone_el
                    class="flex flex-col w-full min-h-[200px] h-auto bg-gray-400/10 justify-center items-center pt-6"
                >
                    <div class="flex flex-wrap justify-center items-center">
                        {move || store_files_button()}
                    </div>
                    <div class="flex flex-wrap justify-center items-center">
                        {move || delete_files_button()}
                    </div>
                    <div class="flex flex-col justify-center items-center">
                        <div>{move || is_over_drop_zone.get().then_some("Just let go...")}</div>
                        <p>
                            <span>"Dropped "</span>
                            {move || dropped_files.get().len()}
                            <span>" file(s)"</span>
                        </p>
                    </div>
                    <div class="flex flex-wrap justify-center items-center gap-4">
                        {new_file_divs}
                    </div>
                </div>
            </div>
        </div>
    }
}

async fn store_files_in_localstorage(files: Vec<SendWrapper<File>>, set_all_real_input_files: WriteSignal<AocInput>) {
    log!("Storing {} files in localstorage", files.len());
    let files_with_contents = futures::future::join_all(files.iter().map(|file| read_file_content(file).map(|c| (file.name(), c)))).await;

    let content = files_with_contents
        .iter()
        .cloned()
        .map(|(name, content)| {
            let day: u32 = parse_day_from_str(&name).unwrap();
            AocDayInput { day, input: content }
        })
        .collect_vec();

    console_log(format!("content for all {} days", files.len()).as_str());
    let serialized = serde_json::to_string_pretty(&content).unwrap();
    console_log(serialized.as_str());

    set_all_real_input_files.set(AocInput { days: content });
}
