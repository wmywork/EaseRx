use crate::executor::executor_state::ExecutorState;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Line, Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph};
use throbber_widgets_tui::Set;
const MOON_PHASE: Set = Set {
    full: "🌕",
    empty: "　",
    symbols: &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
};

pub fn executor_view<'a>(executor_state: &ExecutorState) -> Paragraph<'a> {
    let mut spans = Vec::new();
    let async_num_state = format!("calc num state: {:?}", executor_state.async_num);

    if executor_state.async_num.is_loading() {
        let throbber_span = throbber_widgets_tui::Throbber::default()
            .throbber_set(MOON_PHASE)
            .use_type(throbber_widgets_tui::WhichUse::Spin)
            .to_symbol_span(&executor_state.throbber_state);
        spans.push(throbber_span);
    }

    spans.push(async_num_state.into());

    if executor_state.repeated_clicks {
        spans.push(" Throttle: repeated clicks".red());
    };

    let num_calc_text = Line::from(spans);

    let status = if executor_state.async_num.is_loading() {
        "[Executing]"
    } else {
        ""
    };

    Paragraph::new(num_calc_text)
        .block(Block::default().title(format!("Executor {}", status)).borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
}
