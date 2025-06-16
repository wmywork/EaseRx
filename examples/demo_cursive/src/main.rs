use crate::counter::counter_model::CounterModel;
use crate::counter::counter_view::{counter_view, update_counter};
use crate::executor::executor_model::ExecutorModel;
use crate::executor::executor_view::{executor_view, update_executor};
use crate::input::input_handler::InputHandler;
use crate::progress::progress_model::ProgressModel;
use crate::progress::progress_view::{progress_view, update_progress};
use cursive::theme::{BorderStyle, Palette, Theme};
use cursive::views::{LinearLayout, Panel, TextView};
use cursive::Cursive;
use easerx::combine_state_flow;
use futures::StreamExt;
use futures_signals::map_ref;
use futures_signals::signal::SignalExt;
use input::input_handler;
use std::sync::Arc;

mod counter;
mod executor;
mod input;
mod progress;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut siv = cursive::default();

    let progress_model = Arc::new(ProgressModel::new());
    let counter_model = Arc::new(CounterModel::new());
    let executor_model = Arc::new(ExecutorModel::new());
    let input_handler = Arc::new(InputHandler::new());

    input_handler::setup_event_handler(
        &mut *siv,
        progress_model.clone(),
        counter_model.clone(),
        executor_model.clone(),
        input_handler.clone(),
    );

    ui_view(&mut *siv);

    let cb = siv.cb_sink().clone();

    let mut state_flow = combine_state_flow! {
        progress_model.store().to_signal(),
        counter_model.store().to_signal(),
        executor_model.store().to_signal(),
        input_handler.store().to_signal(),
    }
    .to_stream();
    tokio::spawn(async move {
        loop {
            if let Some((progress, counter, executor, input)) = state_flow.next().await {
                if input.exit {
                    break;
                }
                let _ = cb.send(Box::new(move |s: &mut Cursive| {
                    update_progress(s, &progress);
                    update_counter(s, &counter);
                    update_executor(s, &executor);
                }));
            } else {
                break;
            }
        }
    });
    siv.run();
    Ok(())
}

pub fn ui_view(siv: &mut Cursive) {
    // Set custom theme
    siv.set_theme(Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::retro(),
    });

    let controls_text = Panel::new(TextView::new(
        "App: quit <ESC> or <Ctrl+C>, reset <r>\n\
         Progress: <Left>/<Right> color: <Up>/<Down>\n\
         Counter: decrement <->, increment <+>, start <o>, stop <p>\n\
         Executor: calc <c>",
    ))
    .title("Controls");

    let layout = LinearLayout::vertical()
        .child(progress_view())
        .child(counter_view())
        .child(executor_view(siv.cb_sink().clone()))
        .child(controls_text);

    siv.add_layer(layout)
}
