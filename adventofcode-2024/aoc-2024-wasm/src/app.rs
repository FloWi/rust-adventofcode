use aoc_2024_wasm::testcases::Testcase;
use itertools::Itertools;
use leptos::html::{div, main, pre, title, ul};
use leptos::prelude::*;
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
        <Stylesheet id="leptos" href="/style/output.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
       <div id="root">
      // we wrap the whole app in a <Router/> to allow client-side navigation
      // from our nav links below
      <Router>
        <main>
          // <Routes/> both defines our routes and shows them on the page
          <Routes fallback=|| "Not found.">
            // our root route: the contact list is always shown
            <ParentRoute
              path=path!("days")
              view=AocDays
            >
              // users like /gbj or /bob
              <Route
                path=path!(":day")
                view=AocDay
              />
              // a fallback if the /:id segment is missing from the URL
              <Route
                path=path!("")
                view=move || view! { <p class="contact">"Select a contact."</p> }
              />
            </ParentRoute>
          </Routes>
        </main>
      </Router>
    </div>
    }
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
            let json_str = match maybe_testcases_for_day {
                None => "".to_string(),
                Some((_, testcases)) => serde_json::to_string_pretty(&testcases).unwrap_or("".to_string()),
            };

            div().child(format!("AocDay - Day {:02}", day)).child(pre().child(json_str))
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
