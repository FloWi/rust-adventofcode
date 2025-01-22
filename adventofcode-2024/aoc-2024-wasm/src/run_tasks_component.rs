use leptos::prelude::*;
use leptos::{component, IntoView};

#[component]
pub fn RunTasksComponent(store: ReadSignal<crate::app::TaskStore>, run_tasks: Action<(), (), LocalStorage>) -> impl IntoView {
    move || {
        view! {
            <div class="p-4">
                <h1 class="text-2xl font-bold mb-4">"Advent of Code Tasks"</h1>
                <button
                        class="bg-blue-500 text-white px-4 py-2 rounded disabled:bg-gray-400"
                        disabled={move || store.get().is_running || store.get().tasks.is_empty()}
                        on:click=move |_| {run_tasks.dispatch(());}
                    >
                        {move || if store.get().is_running {
                            "Running..."
                        } else {
                            "Run All Tasks"
                        }}
                </button>

                // Combined Tasks and Results view
                <div class="mb-4">
                    <h2 class="text-xl mb-2">"Tasks:"</h2>
                    <div class="space-y-2">
                        {move || store.get().all_tasks_with_results().into_iter().map(|(task, maybe_result)| {
                            match maybe_result {
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
}
