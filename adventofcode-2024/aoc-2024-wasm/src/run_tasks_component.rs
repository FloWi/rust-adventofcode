use crate::app::TaskStore;
use leptos::prelude::*;
use leptos::{component, IntoView};

#[component]
pub fn RunTasksComponent(store: TaskStore) -> impl IntoView {
    let store_clone = store.clone();

    let run_tasks: Action<(), ()> = Action::new(move |_: &()| {
        let sure_why_not_clone_again = store_clone.clone();
        async move { sure_why_not_clone_again.run().await }
    });

    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold mb-4">"Advent of Code Tasks"</h1>
            <button
                    class="bg-blue-500 text-white px-4 py-2 rounded disabled:bg-gray-400"
                    disabled={move || store.is_running.get()}
                    on:click=move |_| {run_tasks.dispatch(());}
                >
                    {move || if store.is_running.get() {
                        "Running..."
                    } else {
                        "Run All Tasks"
                    }}
            </button>

            // Combined Tasks and Results view
            <div class="mb-4">
                <h2 class="text-xl mb-2">"Tasks:"</h2>
                <div class="space-y-2">
                    { move || store.result_signals.iter().map(|(task, maybe_result_signal)| {
                        match maybe_result_signal.get() {
                        None => {
                            (view! {
                                <div class="p-2 bg-gray-800 rounded">
                                    <div class="font-bold">{format!("{:?}", task)}</div>
                                    <div class="text-gray-500">"Pending..."</div>
                                </div>
                            }).into_any()
                            },
                            Some(result) => {
                                (view! {
                                    <div class="p-2 bg-green-800 rounded">
                                        <div class="font-bold">{format!("{:?}", task)}</div>
                                        <div>{format!("{:?}", result)}</div>
                                    </div>
                                }).into_any()
                            }
                        }
                    }).collect_view()}
                </div>
            </div>


        </div>
    }
}
