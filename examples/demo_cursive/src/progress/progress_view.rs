use crate::progress::progress_state::ProgressState;
use cursive::view::Nameable;
use cursive::views::{NamedView, Panel, ProgressBar};

const PROGRESS_BAR_NAME: &str = "progress_bar";

pub fn progress_view() -> Panel<NamedView<ProgressBar>> {
    Panel::new(
        ProgressBar::new()
            .range(0, 100)
            .with_name(PROGRESS_BAR_NAME),
    )
    .title("Progress")
}

pub fn update_progress(siv: &mut cursive::Cursive, progress: &ProgressState) {
    siv.call_on_name(PROGRESS_BAR_NAME, |view: &mut ProgressBar| {
        view.set_value(progress.progress as usize);
        view.set_color(progress.get_color())
    });
}
