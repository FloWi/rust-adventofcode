use crate::app::{AocDayInput, RunTaskData, Status, TaskStore};
use aoc_2024_wasm::{Part, Solution};
use chrono::{DateTime, Utc};
use humantime::format_duration;
use itertools::Itertools;
use itertools::*;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::{component, IntoView};
use leptos_router::components::A;

#[component]
pub fn RunTasksComponent(store: TaskStore) -> impl IntoView {
    let store_clone = store.clone();
    let num_tasks = store.result_signals.len();

    let run_tasks = Action::new_local(move |_: &()| {
        let sure_why_not_clone_again = store_clone.clone();
        async move { sure_why_not_clone_again.run().await }
    });

    if store.result_signals.is_empty() {
        (view! {
        <div class="p-4">
            <h1 class="text-2xl font-bold mb-4">"Performance Test Of All Days"</h1>
            <p>
              <span>"The AoC authors asked us to not share the input files publicly. Please provide your own files for storing them in localstorage. "</span>
              <A href="/adventofcode-2024/manage-inputs"><span class="font-medium text-blue-800 underline dark:text-blue-300 hover:no-underline">"Manage inputs here"</span></A>
            </p>
        </div>
         })
        .into_any()
    } else {
        (view! {
            <div class="p-4">
                <h1 class="text-2xl font-bold mb-4">"Performance Test Of All Days"</h1>
                <button
                    class="bg-blue-500 text-white px-4 py-2 rounded disabled:bg-gray-400"
                    disabled=move || {
                        let status = store.status.get();
                        match status {
                            Status::Running { .. } => true,
                            _ => false,
                        }
                    }
                    on:click=move |_| {
                        run_tasks.dispatch(());
                    }
                >
                    "Run All Tasks"

                </button>

                // Combined Tasks and Results view
                <div class="mb-4">
                    <h2 class="text-xl mb-2">"Tasks:"</h2>
                    <div>
                        {move || {
                            let status = store.status.get();
                            match status {
                                Status::NotStarted => "Idle".to_string(),
                                Status::Running { num_tasks_done, start_time } => {
                                    format!(
                                        "{} of {} - {}",
                                        num_tasks_done,
                                        num_tasks,
                                        pretty_print_time_delta(start_time, Utc::now()),
                                    )
                                }
                                Status::Done { start_time, end_time } => {
                                    format!(
                                        "{} of {} - {}",
                                        num_tasks,
                                        num_tasks,
                                        pretty_print_time_delta(start_time, end_time),
                                    )
                                }
                            }
                        }}
                    </div>
                    <p>"For reference: Took ~9.5s on macbook pro m1 max 64GB."</p>
                    <div class="space-y-2">
                        <table class="table-auto border border-collapse border-gray-400 dark:border-gray-500">
                            <thead>
                                <tr>
                                    <th class="border border-gray-300 dark:border-gray-600 p-2">
                                        "Day"
                                    </th>
                                    <th class="border border-gray-300 dark:border-gray-600 p-2 text-right">
                                        "Part 1 Result"
                                    </th>
                                    <th class="border border-gray-300 dark:border-gray-600 p-2 text-right">
                                        "Part 1 Time"
                                    </th>
                                    <th class="border border-gray-300 dark:border-gray-600 p-2 text-right">
                                        "Part 2 Result"
                                    </th>
                                    <th class="border border-gray-300 dark:border-gray-600 p-2 text-right">
                                        "Part 2 Time"
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    store
                                        .result_signals
                                        .iter()
                                        .filter_map(|(t, signal)| match t {
                                            RunTaskData::RunReal { task } => Some((task, signal)),
                                            RunTaskData::RunTestcase { .. } => None,
                                        })
                                        .inspect(|t| {
                                            log!(
                                                "before group: day {} part {:?}", t.0.input.day, t.0.part
                                            )
                                        })
                                        .into_group_map_by(|(t, _)| t.input.clone())
                                        .into_iter()
                                        .inspect(|t| {
                                            log!(
                                                "after group: day {} num signals: {}", t.0.day, t.1.len()
                                            )
                                        })
                                        .sorted_by_key(|(day_input, _)| day_input.day)
                                        .map(|(day_input, data_for_day)| {
                                            let day_signals = data_for_day
                                                .into_iter()
                                                .map(|(task, signal)| (task.part.clone(), signal.clone()))
                                                .collect_vec();
                                            log!(
                                                "got {} signals for day {}", day_signals.len(), day_input.day
                                            );
                                            (day_input, day_signals)
                                        })
                                        .map(|(day_input, part_signals)| {
                                            // don't ask - borrow-checker made me do this
                                            view! {
                                                <ResultRowRealTask
                                                    input=day_input
                                                    part_result_signals=part_signals
                                                />
                                            }
                                        })
                                        .collect_view()
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        })
        .into_any()
    }
}

#[component]
fn ResultRowRealTask(input: AocDayInput, #[prop(into)] part_result_signals: Vec<(Part, ReadSignal<Option<Solution>>)>) -> impl IntoView {
    let day = input.day;

    log!("rendering row for day {} with {} parts", input.day, part_result_signals.len());

    view! {
        <tr>
            <td class="border border-gray-300 dark:border-gray-700 p-2">{format!("{day:02}")}</td>
            <For
                each=move || part_result_signals.clone()
                key=|(part, _)| part.clone()
                children=move |(_, signal)| {
                    {
                        move || match signal.get() {
                            None => {
                                (view! {
                                    <td class="border border-gray-300 dark:border-gray-700 text-right p-2 text-orange-400">
                                        "Pending"
                                    </td>
                                    <td class="border border-gray-300 dark:border-gray-700 text-right p-2 text-orange-400">
                                        "Pending"
                                    </td>
                                })
                                    .into_any()
                            }
                            Some(result) => {
                                (view! {
                                    <td class="border border-gray-300 dark:border-gray-700 text-right p-2">
                                        {result.result}
                                    </td>
                                    <td class="border border-gray-300 dark:border-gray-700 text-right p-2">
                                        {format_duration(result.duration.to_std().unwrap())
                                            .to_string()}
                                    </td>
                                })
                                    .into_any()
                            }
                        }
                    }
                }
            />
        </tr>
    }
}

fn pretty_print_time_delta(start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    let duration = end - start;
    let std_duration = duration.to_std().unwrap();
    format_duration(std_duration).to_string()
}
