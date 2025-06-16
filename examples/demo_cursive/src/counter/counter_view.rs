use crate::counter::counter_state::CounterState;
use cursive::view::Nameable;
use cursive::views::{NamedView, Panel, TextView};
const COUNTER_NAME: &str = "counter";
const COUNTER_TITLE: &str = "counter_title";

pub fn counter_view() -> NamedView<Panel<NamedView<TextView>>> {
    Panel::new(TextView::new("Count: ").with_name(COUNTER_NAME))
        .title("Counter")
        .with_name(COUNTER_TITLE)
}

pub fn update_counter(siv: &mut cursive::Cursive, counter_state: &CounterState) {
    let status = if counter_state.started {
        "[Running]"
    } else {
        ""
    };
    siv.call_on_name(COUNTER_TITLE, |view: &mut Panel<NamedView<TextView>>| {
        view.set_title(format!("Counter {}", status))
    });
    siv.call_on_name(COUNTER_NAME, |view: &mut TextView| {
        view.set_content(format!("Count: {}", counter_state.count))
    });
}
