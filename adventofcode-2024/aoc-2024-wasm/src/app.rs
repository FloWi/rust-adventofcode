use crate::components::{parse_day_from_str, styled_button, AocDayInput, AocInput, OwnInputManager};
use crate::run_tasks_component::RunTasks;
use aoc_2024_wasm::testcases::Testcase;
use aoc_2024_wasm::Part::{Part1, Part2};
use aoc_2024_wasm::{solve_day, Part, Solution};
use chrono::{DateTime, Utc};
use codee::string::JsonSerdeCodec;
use humantime::format_duration;
use itertools::Itertools;
use leptos::html::{div, h2, h3, p, span, textarea, ul};
use leptos::logging::{error, log};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::components::{Outlet, ParentRoute};
use leptos_router::hooks::use_params_map;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};

use leptos_use::storage::use_local_storage;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::Duration;
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let testcases = aoc_2024_wasm::testcases::read_all_testcases();

    let testcases_by_day: Vec<(u32, Vec<Testcase>)> = testcases
        .into_iter()
        .chunk_by(|tc| tc.day)
        // .map(|tc| (format!("{:02}",tc.day), tc))
        .into_iter()
        .map(|(day, day_testcases)| (day, day_testcases.collect_vec()))
        .sorted_by_key(|(day, _)| *day)
        .collect_vec();

    let (testcases_by_day, _set_testcases_by_day) = signal(testcases_by_day);

    provide_context(testcases_by_day);
    let local_storage_key = "adventofcode-2024".to_string();
    let (all_real_input_files, _, _) = use_local_storage::<AocInput, JsonSerdeCodec>(local_storage_key.clone());

    view! {
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
        <div id="root" class="dark">
            // we wrap the whole app in a <Router/> to allow client-side navigation
            // from our nav links below
            <Router>
                // <Routes/> both defines our routes and shows them on the page
                <Routes fallback=|| "Not found.">
                    <ParentRoute
                        path=path!("adventofcode-2024")
                        // this component has an <Outlet/> for rendering the inner <AocDay> component
                        view=Main
                    >
                        <Route
                            path=path!("manage-inputs")
                            view=move || {
                                view! {
                                    <OwnInputManager local_storage_key=local_storage_key.clone() />
                                }
                            }
                        />
                        <Route
                            path=path!("all-days-performance")
                            view=move || {
                                view! { <RunAllComponent aoc_input_files=all_real_input_files /> }
                            }
                        />
                        <Route
                            path=path!("day/:day")
                            view=move || {
                                view! { <AocDay aoc_input_files=all_real_input_files /> }
                            }
                        />
                        // a fallback if the /:id segment is missing from the URL
                        <Route
                            path=path!("")
                            view=move || view! { <p class="day">"Select a day."</p> }
                        />
                    </ParentRoute>
                </Routes>
            </Router>
        </div>
    }
}

async fn write_to_clipboard(text: String) {
    let maybe_clipboard = web_sys::window().map(|w| w.navigator().clipboard());
    match maybe_clipboard {
        Some(cp) => {
            match JsFuture::from(cp.write_text(text.as_str())).await.map_err(|err| format!("Error writing to clipboard: {:?}", err)) {
                Ok(_) => {}
                Err(_) => {
                    error!("Can't write to clipboard")
                }
            }
            
        }
        None => error!("Can't write to clipboard"),
    }
}

#[component]
fn AocTestcase(testcase: Testcase) -> impl IntoView {
    let part = match testcase.part {
        1 => Ok(Part1),
        2 => Ok(Part2),
        _ => Err("Let's not get too ambitions - two parts are enough ;-)"),
    };

    let result = match part {
        Ok(part) => Ok(solve_day(testcase.day, part, &testcase.input, testcase.args.clone())),
        Err(err) => Err(err),
    };

    let result_html = match result.clone() {
        Ok(res) => span().class("font-bold").child(res.result),
        Err(err) => span().class("font-bold red").child(format!("Error: {}", err)),
    };

    let testcase_input = testcase.input.clone();

    let duration = match result {
        Ok(res) => {
            let std_duration = res.duration.to_std().unwrap();
            format_duration(std_duration).to_string()
        }
        Err(_) => "-".to_string(),
    };

    div().child((
        //view! { <textarea readonly class="">{testcase.input}</textarea>  },
        p().child(span().class("font-bold").child("Expected Solution: ")).child(span().child(testcase.solution)),
        p().child(span().class("font-bold").child("Actual Solution: ")).child(result_html),
        p().child(span().class("font-bold").child("Duration: ")).child(duration),
        //testcase.args.map(|args| p().child(span().class("font-bold").child("Custom Args: ")).child(args)),
        testcase.args.map(|arg| p().child(span().class("font-bold").child("Custom Args: ")).child(span().child(arg))),
        p().class("font-bold mt-4").child(span().child("Testdata:")),
        styled_button().on(leptos::ev::click, move |_| spawn_local(write_to_clipboard(testcase_input.clone()))).child("Copy"),
        textarea()
            .readonly(true)
            .class("w-full overflow-y-auto overflow-x-auto whitespace-pre text-inherit bg-inherit")
            .child(testcase.input)
            .rows(20)
            .cols(40),
    ))
}

fn parts_for_day(day: u32) -> Vec<Part> {
    if day < 25 {
        vec![Part1, Part2]
    } else {
        vec![Part1]
    }
}

#[component]
fn AocDay(aoc_input_files: Signal<AocInput>) -> impl IntoView {
    //let (real_inputs, _, _) = use_local_storage::<AocInput, codee::string::JsonSerdeCodec>("adventofcode-2024");

    let testcases_by_day = use_context::<ReadSignal<Vec<(u32, Vec<Testcase>)>>>().expect("to have found the testcases");
    let params = use_params_map();
    let maybe_day = move || params.read().get("day");
    let maybe_day_and_inputs = move || {
        maybe_day().map(|day_str| {
            let maybe_real_input = aoc_input_files.get().days.iter().find(|d| d.day.to_string() == day_str).cloned();
            (day_str, maybe_real_input)
        })
    };

    move || {
        maybe_day_and_inputs().map(|(day_str, maybe_real_input)| {
            let maybe_testcases_for_day = testcases_by_day.read().iter().find(|&(d, _)| d.to_string() == day_str).cloned();
            let day = parse_day_from_str(&day_str).unwrap();

            //FIXME: this doesn't refresh when I navigate around by clicking links
            // reloading the page with the route '/days/:day' _does_ work
            let part_divs = maybe_testcases_for_day.map(|(_, testcases)| {
                testcases
                    .iter()
                    .into_group_map_by(|tc| tc.part)
                    .into_iter()
                    .sorted_by_key(|tup| tup.0)
                    .map(|(part, testcases)| {
                        div()
                            .class("flex flex-col gap-4 divide-y")
                            .child(h3().child(format!("Part {}: {} Testcases", part, testcases.len())))
                            .child(testcases.into_iter().map(|tc| AocTestcase(AocTestcaseProps { testcase: tc.clone() })).collect_view())
                    })
                    .collect_view()
            });

            let real_input_divs = maybe_real_input.clone().map(|inp| {
                parts_for_day(day)
                    .into_iter()
                    .map(|part| {
                        log!("calculating result for real input for day {day} part {part:?}. Input: {}", inp.input);
                        let result = solve_day(day, part.clone(), inp.input.as_str(), None);
                        let std_duration = result.duration.to_std().unwrap();
                        let duration_pretty = format_duration(std_duration).to_string();

                        log!("calculated result for real input for day {day} part {part:?}. Result: {result:?}");
                        div()
                            .child(h3().child(format!("Real input {part:?}")))
                            .child([
                                p().child(span().class("font-bold").child("Actual Solution: ")).child(result.result),
                                p().child(span().class("font-bold").child("Duration: ")).child(duration_pretty),
                            ])
                            .into_any()
                    })
                    .collect_view()
            });

            div()
                .child(h2().class("text-xl font-bold").child(format!("AocDay - Day {:02}", day_str)))
                .child(div().class("flex flex-row gap-8 divide-x").child(part_divs).child(real_input_divs))
                .child(
                    div().class("flex flex-col gap-4").child(h3().child("real input")).child(div().class("flex flex-row gap-4")).child(
                        textarea()
                            .readonly(true)
                            .class("w-full overflow-y-auto overflow-x-auto whitespace-pre text-inherit bg-inherit")
                            .child(maybe_real_input.map(|real| real.input))
                            .rows(40)
                            .cols(40),
                    ),
                )
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealTask {
    pub(crate) input: AocDayInput,
    pub(crate) part: Part,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestcaseTask {
    testcase: Testcase,
    id: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunTaskData {
    RunReal { task: RealTask },
    RunTestcase { task: TestcaseTask },
}

impl RunTaskData {
    pub(crate) fn id(&self) -> String {
        match self {
            RunTaskData::RunReal {
                task: RealTask { input, part },
            } => {
                format!("Day {:02} - Part {part:?} - real", input.day)
            }
            RunTaskData::RunTestcase {
                task: TestcaseTask { testcase, id },
            } => {
                format!("Day {:02} - Part {:?} - testcase #{}", testcase.day, testcase.part, id)
            }
        }
    }
}

impl Hash for RunTaskData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.id().as_bytes())
    }
}

#[derive(Clone)]
pub enum Status {
    NotStarted,
    Done { start_time: DateTime<Utc>, end_time: DateTime<Utc> },
    Running { start_time: DateTime<Utc>, num_tasks_done: u32 },
}

// Store Component
#[derive(Clone)]
pub struct TaskStore {
    results: HashMap<RunTaskData, (ReadSignal<Option<Solution>>, WriteSignal<Option<Solution>>)>,
    pub result_signals: Vec<(RunTaskData, ReadSignal<Option<Solution>>)>,
    pub status: ReadSignal<Status>,
    set_status: WriteSignal<Status>,
}

impl TaskStore {
    fn new(tasks: Vec<RunTaskData>) -> Self {
        let results: HashMap<RunTaskData, (ReadSignal<Option<Solution>>, WriteSignal<Option<Solution>>)> =
            tasks.iter().map(|t| (t.clone(), signal(None))).collect();
        let result_signals = results.iter().map(|(t, signals)| (t.clone(), signals.0)).collect_vec();
        let (status, set_status) = signal(Status::NotStarted);
        Self {
            results,
            result_signals,
            status,
            set_status,
        }
    }

    pub async fn run(&self) {
        let start_time = Utc::now();
        self.set_status.set(Status::Running { num_tasks_done: 0, start_time });

        let mut tasks_queue: VecDeque<&RunTaskData> = VecDeque::from(
            self.results
                .keys()
                .filter(|t| {
                    // skip slow-running task in dev-build (take way too long)
                    if cfg!(not(debug_assertions)) {
                        true
                    } else {
                        let too_slow = HashSet::from([
                            (6, Part2),
                            (7, Part2),
                            (9, Part1),
                            (9, Part2),
                            (12, Part1),
                            (12, Part2),
                            (22, Part1),
                        ]);
                        match t {
                            RunTaskData::RunReal {
                                task: RealTask { input, part },
                            } => !too_slow.contains(&(input.day, part.clone())),
                            RunTaskData::RunTestcase { .. } => true,
                        }
                    }
                })
                .sorted_by_key(|t| t.id())
                .collect_vec(),
        );

        let mut num_tasks_done = 0;

        while let Some(task) = tasks_queue.pop_front() {
            let result = run_task(task).await;
            num_tasks_done += 1;
            self.results.get(task).unwrap().1.set(Some(result));
            self.set_status.set(Status::Running { num_tasks_done, start_time });

            //TODO: figure out how to properly yield to UI thread (or rather give it time to catch on)
            gloo_timers::future::sleep(Duration::from_micros(1)).await;
        }
        self.set_status.set(Status::Done {
            start_time,
            end_time: Utc::now(),
        });
    }
}

#[component]
fn RunAllComponent(aoc_input_files: Signal<AocInput>) -> impl IntoView {
    // let testcases_by_day = use_context::<ReadSignal<Vec<(u32, Vec<Testcase>)>>>().expect("to have found the testcases");
    let all_tasks: Vec<RunTaskData> = aoc_input_files
        .get_untracked()
        .days
        .iter()
        .flat_map(|d| {
            parts_for_day(d.day).into_iter().map(|part| RunTaskData::RunReal {
                task: RealTask { input: d.clone(), part },
            })
        })
        // .chain(testcases_by_day.get().iter().flat_map(|(_day, testcases)| {
        //     testcases.iter().enumerate().map(|(idx, tc)| RunTaskData::RunTestcase {
        //         task: TestcaseTask { testcase: tc.clone(), id: idx },
        //     })
        // }))
        .sorted_by_key(|t| t.id())
        .collect_vec();

    let store: TaskStore = TaskStore::new(all_tasks);

    view! { <RunTasks store=store /> }
}

async fn run_task(task: &RunTaskData) -> Solution {
    log!("running {}", task.id());
    let result = match task {
        RunTaskData::RunReal {
            task: RealTask { input, part },
        } => solve_day(input.day, part.clone(), &input.input, None),
        RunTaskData::RunTestcase {
            task: TestcaseTask { testcase, .. },
        } => {
            let part: Part = testcase.part.try_into().unwrap();
            solve_day(testcase.day, part, &testcase.input, testcase.args.clone())
        }
    };

    log!("ran {}", task.id());
    result
}

#[component]
fn Main() -> impl IntoView {
    let testcases_by_day = use_context::<ReadSignal<Vec<(u32, Vec<Testcase>)>>>().expect("to have found the testcases");

    let days = testcases_by_day.read().iter().cloned().map(|(day, _)| day).collect_vec();

    let days_html = ul().child(
        days.iter()
            .cloned()
            .map(|d| {
                let label = format!("Day {:02}", d);
                // haven't figured out how to generate an <A> tag using the builder-pattern
                // let foo = A(AProps {href: format!("{d}"), target: None, exact: false, strict_trailing_slash: false, scroll: false);

                // haven't found a way to attach a class attribute to the <A> tag
                // e.g. bg-gray-600 aria-current:bg-sky-700
                // using css file to style a-tag conditionally
                view! {
                    <li>
                        <A href=format!("day/{d}")>{label}</A>
                    </li>
                }
            })
            .collect_view(),
    );

    let other_links_html = view! {
        <ul>
            <li>
                <A href="manage-inputs">"manage inputs"</A>
            </li>
            <li>
                <A href="all-days-performance">"performance all days"</A>
            </li>
        </ul>
    };

    // let nav_bar = div().class("flex flex-col min-w-fit gap-4 divide-y").child(days_html).child(other_links_html);

    view! {
        <div class="desktop-page bg-background shadow-md p-4">
            <title>"Advent Of Code 2024"</title>
            <div class="header">
                <h1>"Advent Of Code 2024"</h1>
            </div>
            <div class="sub-header"></div>
            <div class="left">{days_html} {other_links_html}</div>
            <div class="content">
                <Outlet />
            </div>
        </div>
    }

    // div().class("bg-gradient-to-tl from-blue-900 to-blue-600 text-white font-mono flex flex-col min-h-screen").child((
    //     title().child("Advent Of Code 2024"),
    //     main().child(div().class("text-white font-mono flex flex-row min-w-screen gap-8 p-8").child((nav_bar, Outlet()))),
    // ))
}
