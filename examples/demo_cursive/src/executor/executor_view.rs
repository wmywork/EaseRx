use crate::executor::executor_state::ExecutorState;
use cursive::style::Color;
use cursive::traits::Nameable;
use cursive::views::{LinearLayout, NamedView, Panel, TextView};
use cursive::{CbSink, Cursive};
use cursive_spinner_view::{Frames, SpinnerView};

const EXECUTOR_NAME: &str = "executor";
const EXECUTOR_TITLE: &str = "executor_title";
const SPINNER_NAME: &str = "spinner";

pub const MOON_PHASE: Frames = &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];

pub fn executor_view(cb_sink: CbSink) -> NamedView<Panel<LinearLayout>> {
    let text_view = TextView::new("Calc num State: ").with_name(EXECUTOR_NAME);

    let mut spinner = SpinnerView::new(cb_sink);

    spinner
        .frames(MOON_PHASE)
        .style(Color::parse("black").unwrap());

    let line = LinearLayout::horizontal()
        .child(spinner.with_name(SPINNER_NAME))
        .child(text_view);

    Panel::new(line).title("Executor").with_name(EXECUTOR_TITLE)
}

pub fn update_executor(siv: &mut Cursive, executor_state: &ExecutorState) {
    let status = if executor_state.async_num.is_loading() {
        "[Executing]"
    } else {
        ""
    };
    siv.call_on_name(EXECUTOR_TITLE, |view: &mut Panel<LinearLayout>| {
        view.set_title(format!("Executor {}", status))
    });

    // Base status text
    let mut status = format!("calc num state: {:?}", executor_state.async_num);

    // Add throttle warning if needed
    if executor_state.repeated_clicks {
        status.push_str(" Throttle: repeated clicks");
    }

    siv.call_on_name(EXECUTOR_NAME, |view: &mut TextView| {
        view.set_content(status)
    });

    if executor_state.async_num.is_loading() {
        siv.call_on_name(SPINNER_NAME, |view: &mut SpinnerView| {
            view.spin_up();
        });
    } else {
        siv.call_on_name(SPINNER_NAME, |view: &mut SpinnerView| {
            view.spin_down();
            view.stop();
        });
    }
}
