use crate::app::*;
use tui::backend::Backend;
use tui::layout::*;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::*;
use tui::Frame;

fn draw_tasks<B: Backend>(f: &mut Frame<B>, area: &Rect, state: &AppState) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            vec![
                Constraint::Percentage(100 / state.project.columns.len() as u16);
                state.project.columns.len()
            ]
            .as_ref(),
        )
        .split(*area);

    for (i, column) in state.project.columns.iter().enumerate() {
        let items: Vec<ListItem> = column.tasks
            .iter()
            .enumerate()
            .map(|(j, task)| {
                let mut style = Style::default();
                let col_idx = state.project.selected_column_idx;
                let task_idx = state.project.get_selected_column().selected_task_idx;
                if i == col_idx && j == task_idx {
                    style = style.fg(Color::White).add_modifier(Modifier::BOLD);
                } else {
                    style = style.fg(Color::White);
                }
                let mut s = Span::raw(task.title.as_str());
                s.style = style;
                ListItem::new(vec![Spans::from(s)])
            })
            .collect();
        let mut style = Style::default();
        if i == state.project.selected_column_idx {
            style = style.fg(Color::Green);
        };
        let mut s = Span::raw(format!("{:?}", column.name));
        s.style = Style::default()
            .add_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED)
            .fg(Color::White);
        let block = Block::default().style(style).title(s).borders(Borders::ALL);
        let list = List::new(items).block(block);
        f.render_widget(list, columns[i])
    }
}

fn draw_task_info<B: Backend>(f: &mut Frame<B>, area: &Rect, state: &AppState) {
    let block = Block::default().title("TASK INFO").borders(Borders::ALL);
    if let Some(task) = state.project.get_selected_column().get_selected_task() {
        let p = Paragraph::new(task.description.as_str())
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, *area);
    } else {
        let p = Paragraph::new("No tasks for this column").block(block);
        f.render_widget(p, *area);
    }
}

fn centered_rect_for_popup(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn draw_new_task_popup<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let area = centered_rect_for_popup(45, 60, f.size());
    f.render_widget(Clear, area);
    match &mut state.new_task_state {
        None => {}
        Some(task) => {
            let block = Block::default()
                .title("Add Task")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL);
            let block_inner = block.inner(area);
            // let main = Paragraph::new(task.description.clone()).block(block);
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Max(100),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(block_inner);
            let b1 = Block::default().title("Title").borders(Borders::ALL);
            // let title = Paragraph::new("Hello I am text")
                // .style(Style::default().fg(Color::Yellow))
                // .block(b1);
            let b2 = Block::default().title("Description").borders(Borders::ALL);
            // let description = Paragraph::new("Fill this out")
                // .style(Style::default().fg(Color::Yellow))
                // .block(b2);
            let b3 = Block::default().title("Keys").borders(Borders::TOP);
            let footer = Paragraph::new("p : Cancel").block(b3);
            // f.render_widget(main, area);
            // f.render_widget(title, layout[0]);
            task.title.set_block(b1);
            if let NewTaskFocus::Title = task.focus {
                task.title.set_style(Style::default().fg(Color::Yellow));
                task.title.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            } else {
                task.title.set_style(Style::default());
                task.title.set_cursor_style(Style::default());
            }
            f.render_widget(task.title.widget(), layout[0]);
            // f.render_widget(description, layout[1]);
            task.description.set_block(b2);
            if let NewTaskFocus::Description = task.focus {
                task.description.set_style(Style::default().fg(Color::Yellow));
                task.description.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            } else {
                task.description.set_style(Style::default());
                task.description.set_cursor_style(Style::default());
            }
            f.render_widget(task.description.widget(), layout[1]);
            f.render_widget(footer, layout[2]);
        }
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(65),
                Constraint::Percentage(20),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let block = Block::default().title("KANBAN BOARD").borders(Borders::ALL);
    f.render_widget(block, main_layout[0]);

    draw_tasks(f, &main_layout[1], &state);

    draw_task_info(f, &main_layout[2], &state);

    let block = Block::default().title("KEYBINDINGS").borders(Borders::TOP);

    let foot_txt =
        Paragraph::new("q : Quit | ⏪🔽🔼⏩ or hjkl : Navigation | < > or H L : Shift task left/right | = - or J K : Shift task up/down")
            .block(block);
    f.render_widget(foot_txt, main_layout[3]);

    if state.new_task_state.is_some() {
        draw_new_task_popup(f, state);
    }
}
