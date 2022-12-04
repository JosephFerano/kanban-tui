use tui::backend::{Backend};
use tui::layout::*;
use tui::{Frame};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::*;
use crate::types::*;
use int_enum::IntEnum;

fn draw_tasks<B: Backend>(f: &mut Frame<B>, area: &Rect, state: &AppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            vec![Constraint::Percentage(20);
                 state.current_project.tasks_per_column.len()].as_ref()
        )
        .split(*area);

    for (i, (status, tasks)) in state.current_project.tasks_per_column.iter().enumerate() {
        let items: Vec<ListItem> = tasks.iter().enumerate().map(|(j, task)| {
            let mut style = Style::default();
            if i == state.selected_column && j == state.selected_task[state.selected_column] {
                style = style.fg(Color::White).add_modifier(Modifier::BOLD);
            } else {
                style = style.fg(Color::White);
            }
            let mut s = Span::raw(task.title.as_str());
            s.style = style;
            ListItem::new(vec![Spans::from(s)])
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
    let column: TaskStatus = TaskStatus::from_int(state.selected_column).unwrap();
    let tasks = state.current_project.tasks_per_column.get(&column).unwrap();
    if tasks.len() > 0 {
        let task: &Task = &tasks[state.selected_task[state.selected_column]];
        let p = Paragraph::new(task.description.as_str()).block(block).wrap(Wrap { trim: true });
        f.render_widget(p, *area);
    } else {
        let p = Paragraph::new("No tasks for this column").block(block);
        f.render_widget(p, *area);
    }
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