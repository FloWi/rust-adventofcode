use aoc_2024_wasm::testcases::Testcase;
use aoc_2024_wasm::Part::{Part1, Part2};
use aoc_2024_wasm::{solve_day, Solution};
use itertools::Itertools;
use leptos::ev::click;
use leptos::html::{button, div, h3, main, p, pre, span, textarea, title, ul, Button, HtmlElement};
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos::tachys::html::class::Class;
use leptos_meta::*;
use leptos_router::components::{Outlet, ParentRoute};
use leptos_router::hooks::use_params_map;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};

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

    view! {
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
       <div id="root">
      // we wrap the whole app in a <Router/> to allow client-side navigation
      // from our nav links below
      <Router>
        <main>
          // <Routes/> both defines our routes and shows them on the page
          <Routes fallback=|| "Not found.">
            <ParentRoute
              path=path!("adventofcode-2024")
              view=AocDays // this component has an <Outlet/> for rendering the inner <AocDay> component
            >
              <Route
                path=path!(":day")
                view=AocDay
              />
              // a fallback if the /:id segment is missing from the URL
              <Route
                path=path!("")
                view=move || view! { <p class="day">"Select a day."</p> }
              />
            </ParentRoute>
          </Routes>
        </main>
      </Router>
    </div>
    }
}

fn write_to_clipboard(text: &str) {
    let maybe_clipboard = web_sys::window().map(|w| w.navigator().clipboard());
    match maybe_clipboard {
        Some(cp) => {
            cp.write_text(text); //TODO: This is fire-and-forget - figure out how to deal with Promises in rust-world
        }
        None => console_log("Can't write to clipboard"),
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
    use humantime::format_duration;

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
        styled_button().on(click, move |_| write_to_clipboard(testcase_input.as_str())).child("Copy"),
        textarea()
            .readonly(true)
            .class("w-full overflow-y-auto overflow-x-auto whitespace-pre text-inherit bg-inherit")
            .child(testcase.input)
            .rows(20)
            .cols(40),
    ))
}

fn styled_button() -> HtmlElement<Button, (Class<&'static str>,), ()> {
    button().class("inline-flex items-center justify-center whitespace-nowrap text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 rounded-md px-3")
}

#[component]
fn AocDay() -> impl IntoView {
    let testcases_by_day = use_context::<ReadSignal<Vec<(u32, Vec<Testcase>)>>>().expect("to have found the testcases");

    let params = use_params_map();
    let maybe_day = move || params.read().get("day");

    move || {
        maybe_day().map(|day| {
            let maybe_testcases_for_day = testcases_by_day.read().iter().cloned().find(|(d, _)| d.to_string() == day);

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

            div().child(format!("AocDay - Day {:02}", day)).child(div().class("flex flex-row gap-8 divide-x").child(part_divs))
        })
    }
}

#[component]
fn AocDays() -> impl IntoView {
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
                      <A href=format!("{d}")>{label}</A>
                    </li>
                }
            })
            .collect_view(),
    );

    // needed to increase both max_width and chain_width to 160 in rustfmt for this to work

    // thanks to https://tailwindcomponents.com/component/blue-buttons-example for the showcase layout
    // can't put Title and main into an array or vec, because this break the type-checker - a tuple works fine.
    let nav_bar = div().class("flex flex-col min-w-fit").child(days_html);

    div().class("bg-gradient-to-tl from-blue-800 to-blue-500 text-white font-mono flex flex-col min-h-screen").child((
        title().child("Leptos + Tailwindcss"),
        main().child(
            div().class("bg-gradient-to-tl from-blue-800 to-blue-500 text-white font-mono flex flex-row min-w-screen gap-8 p-8").child((nav_bar, Outlet())),
        ),
    ))
}
/*
{
    #[allow(unused_braces)] {
        ::leptos::prelude::View::new(::leptos::tachys::html::element::button
            ().child(#[allow(unused_braces)] {
            "Copy to Clipboard"
        }).class("btn btn-accent w-full"
        ).on(::leptos::tachys::html::event::click, move |_| {
            write_to_clipboard(testcase_input.as_str())
        },
        ))
    }
}
 */
