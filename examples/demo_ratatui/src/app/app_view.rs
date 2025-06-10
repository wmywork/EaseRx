use crate::counter::counter_state::CounterState;
use crate::counter::counter_view::counter_view;
use crate::progress::progress_state::ProgressState;
use crate::progress::progress_view::progress_view;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::executor::executor_state::ExecutorState;
use crate::executor::executor_view::executor_view;

pub fn app_view(
    frame: &mut ratatui::Frame,
    progress_state: &ProgressState,
    counter_state: &CounterState,
    executor_state: &ExecutorState,
) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(frame.area());

    let widget_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20),Constraint::Percentage(40), Constraint::Percentage(40)])
        .split(main_layout[0]);

    let progress_widget = progress_view(progress_state);
    frame.render_widget(progress_widget, widget_layout[0]);
    let counter_widget = counter_view(counter_state);
    frame.render_widget(counter_widget, widget_layout[1]);
    let executor_widget = executor_view(executor_state);
    frame.render_widget(executor_widget, widget_layout[2]);


    let mut lines = vec![];
    let app_instructions = Line::from(vec![
        "App: quit ".into(),
        "<ESC>".red().bold(),
        " or ".into(),
        "<Ctrl+C>, ".red().bold(),
        "reset".into(),
        " <r> ".magenta().bold(),
    ])
        .centered();

    let progress_instructions = Line::from(vec![
        "Progress: decrement".into(),
        " <Left>,".cyan().bold(),
        " increment".into(),
        " <Right>, ".cyan().bold(),
        "color".into(),
        " <Up> or <Down>".cyan().bold(),
    ])
        .centered();

    let counter_instructions = Line::from(vec![
        "Counter: decrement".into(),
        " <->, ".cyan().bold(),
        "increment".into(),
        " <+> ".cyan().bold(),
        "start".into(),
        " <o> ".cyan().bold(),
        "stop".into(),
        " <p> ".cyan().bold(),
    ])
        .centered();

    let executor_instructions = Line::from(vec![
        "Executor: calc".into(),
        " <c>, ".cyan().bold(),
    ]);

    lines.push(app_instructions);
    lines.push(progress_instructions);
    lines.push(counter_instructions);
    lines.push(executor_instructions);
    let text = Text::from(lines);

    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().title("Controls").borders(Borders::ALL))
            .style(Style::default())
            .alignment(Alignment::Center),
        main_layout[1],
    );
}
