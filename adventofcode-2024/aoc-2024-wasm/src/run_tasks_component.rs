use crate::app::{Status, TaskStore};
use chrono::{DateTime, Utc};
use humantime::format_duration;
use leptos::prelude::*;
use leptos::{component, IntoView};

#[component]
pub fn RunTasksComponent(store: TaskStore) -> impl IntoView {
    let store_clone = store.clone();
    let num_tasks = store.result_signals.len();

    let run_tasks = Action::new_local(move |_: &()| {
        let sure_why_not_clone_again = store_clone.clone();
        async move { sure_why_not_clone_again.run().await }
    });

    view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold mb-4">"Advent of Code Tasks"</h1>
            <button
                    class="bg-blue-500 text-white px-4 py-2 rounded disabled:bg-gray-400"
                    disabled={move || {
                        let status = store.status.get();
                        match status {
                            Status::Running {..} => true,
                            _ => false
                        }
                    }}
                    on:click=move |_| {run_tasks.dispatch(());}
                >    "Run All Tasks"

            </button>

            // Combined Tasks and Results view
            <div class="mb-4">
                <h2 class="text-xl mb-2">"Tasks:"</h2>
                <div>{move || {
                        let status = store.status.get();

                        match status {
                            Status::NotStarted => "Idle".to_string(),
                            Status::Running { num_tasks_done, start_time } => format!("{} of {} - {}", num_tasks_done, num_tasks, pretty_print_time_delta(start_time, Utc::now())),
                            Status::Done{ start_time, end_time } => format!("{} of {} - {}", num_tasks, num_tasks, pretty_print_time_delta(start_time, end_time) )
                        }
                    } }
                </div>
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

fn pretty_print_time_delta(start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    let duration = end - start;
    let std_duration = duration.to_std().unwrap();
    format_duration(std_duration).to_string()
}
