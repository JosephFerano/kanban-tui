use tui::backend::{Backend};
use tui::layout::*;
use tui::{Frame};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::*;
use crate::types::*;

fn draw_tasks<B: Backend>(f: &mut Frame<B>, area: &Rect, state: &AppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            vec![Constraint::Percentage(20);
                 state.current_project.tasks.len()].as_ref()
        )
        .split(*area);

    for (i, (status, tasks)) in state.current_project.tasks.iter().enumerate() {
        let items: Vec<ListItem> = tasks.iter().map(|t| {
            ListItem::new(vec![Spans::from(Span::raw(&t.title))])
        }).collect();
        let mut style = Style::default();
        if i == state.selected_column { style = style.fg(Color::Green); };
        let mut s = Span::raw(format!("{:?}", status));
        s.style = Style::default()
            .add_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED)
            .fg(Color::White);
        let block = Block::default()
            .style(style)
            .title(s)
            .borders(Borders::ALL);
        let list = List::new(items).block(block);
        f.render_widget(list, columns[i])
    }
}

fn draw_task_info<B: Backend>(f: &mut Frame<B>, area: &Rect, state: &AppState) {
    let block = Block::default()
        .title("TASK INFO")
        .borders(Borders::ALL);
    f.render_widget(block, *area);
}

pub fn draw<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ].as_ref()
        ).split(f.size());

    let block = Block::default()
        .title("KANBAN BOARD")
        .borders(Borders::ALL);
    f.render_widget(block, main_layout[0]);

    draw_tasks(f, &main_layout[1], &state);

    draw_task_info(f, &main_layout[2], &state);

    let block = Block::default()
        .title("KEYBINDINGS")
        .borders(Borders::ALL);

    let foot_txt =
        Paragraph::new("q : Quit | ⏪🔽🔼⏩ or hjkl : Navigation")
            .block(block);
    f.render_widget(foot_txt, main_layout[3]);
}