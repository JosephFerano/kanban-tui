use crate::app::{State, TaskEditFocus};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use tui::Frame;

fn draw_tasks<B: Backend>(f: &mut Frame<'_, B>, area: Rect, state: &State<'_>) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            vec![
                Constraint::Percentage(100 / u16::try_from(state.columns.len()).unwrap_or(4));
                state.columns.len()
            ]
            .as_ref(),
        )
        .split(area);

    for (i, column) in state.columns.iter().enumerate() {
        let items: Vec<ListItem<'_>> = column
            .tasks
            .iter()
            .enumerate()
            .map(|(j, task)| {
                let mut style = Style::default();
                let col_idx = state.selected_column_idx;
                let task_idx = state.get_selected_column().selected_task_idx;
                let mut span;
                if i == col_idx && j == task_idx {
                    style = style.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
                    span = Span::raw(format!("{} 👈", task.title));
                } else {
                    span = Span::raw(&task.title);
                }
                span.style = style;
                ListItem::new(vec![Spans::from(span)])
            })
            .collect();

        let mut style = Style::default();
        if i == state.selected_column_idx {
            style = style.add_modifier(Modifier::REVERSED);
        };
        let mut s = Span::raw(column.name.as_str());
        let mods = Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED;
        s.style = Style::default().add_modifier(mods);
        let block = Block::default().title(s).borders(Borders::ALL);
        let inner_area = block.inner(columns[i]);
        let inner_block = Block::default().style(style);
        let list = List::new(items).block(inner_block);

        let mut list_state = ListState::default();
        list_state.select(Some(column.selected_task_idx + 1));

        f.render_widget(block, columns[i]);
        f.render_stateful_widget(list, inner_area, &mut list_state);
    }
}

fn draw_task_info<B: Backend>(f: &mut Frame<'_, B>, area: Rect, state: &State<'_>) {
    let block = Block::default().title("TASK INFO").borders(Borders::ALL);
    if let Some(task) = state.get_selected_task() {
        let p = Paragraph::new(task.description.as_str())
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
    } else {
        let p = Paragraph::new("No tasks for this column").block(block);
        f.render_widget(p, area);
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

fn draw_task_popup<B: Backend>(f: &mut Frame<'_, B>, state: &mut State<'_>, popup_title: &str) {
    let area = centered_rect_for_popup(45, 60, f.size());
    let block = Block::default()
        .title(popup_title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
    let block_inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new("").block(block), area);
    if let Some(task) = &mut state.task_edit_state {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Max(100),
                    Constraint::Length(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(block_inner);

        let buttons = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(80),
                    Constraint::Min(10),
                    Constraint::Min(10),
                ]
                .as_ref(),
            )
            .split(layout[2]);

        let (create_style, cancel_style, create_txt, cancel_txt) = match task.focus {
            TaskEditFocus::ConfirmBtn => (
                Style::default().add_modifier(Modifier::BOLD),
                Style::default(),
                "[Confirm]",
                " Cancel ",
            ),
            TaskEditFocus::CancelBtn => (
                Style::default(),
                Style::default().add_modifier(Modifier::BOLD),
                " Confirm ",
                "[Cancel]",
            ),
            _ => (Style::default(), Style::default(), " Confirm ", " Cancel "),
        };

        let create_btn = Paragraph::new(create_txt).style(create_style);
        let cancel_btn = Paragraph::new(cancel_txt).style(cancel_style);
        f.render_widget(create_btn, buttons[1]);
        f.render_widget(cancel_btn, buttons[2]);

        let b1 = Block::default().title("Title").borders(Borders::ALL);
        let b2 = Block::default().title("Description").borders(Borders::ALL);
        let b3 = Block::default().title("Keys").borders(Borders::TOP);
        task.title.set_cursor_line_style(Style::default());
        task.description.set_cursor_line_style(Style::default());

        task.title.set_block(b1);
        if let TaskEditFocus::Title = task.focus {
            task.title
                .set_style(Style::default().add_modifier(Modifier::BOLD));
            task.title
                .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        } else {
            task.title.set_style(Style::default());
            task.title.set_cursor_style(Style::default());
        }
        f.render_widget(task.title.widget(), layout[0]);

        task.description.set_block(b2);
        if let TaskEditFocus::Description = task.focus {
            task.description
                .set_style(Style::default().add_modifier(Modifier::BOLD));
            task.description
                .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        } else {
            task.description.set_style(Style::default());
            task.description.set_cursor_style(Style::default());
        }
        f.render_widget(task.description.widget(), layout[1]);

        let footer = Paragraph::new("Tab/Backtab : Cycle").block(b3);
        f.render_widget(footer, layout[3]);
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn draw_project_stats<B: Backend>(f: &mut Frame<'_, B>, area: Rect, state: &mut State<'_>) {
    let block = Block::default()
        .title("PROJECT STATS")
        .borders(Borders::ALL);

    let c1_len = state.columns[0].tasks.len();
    let c2_len = state.columns[1].tasks.len();
    let c3_len = state.columns[2].tasks.len();
    let c4_len = state.columns[3].tasks.len();
    let tocomplete_total = c1_len + c2_len + c3_len;
    let percentage = (c3_len as f32 / tocomplete_total as f32 * 100.0) as i8;
    let list = List::new(vec![ListItem::new(vec![
        Spans::from("Tasks per Column:"),
        Spans::from(format!("  Todo ({c1_len})")),
        Spans::from(format!("  In Progress ({c2_len})")),
        Spans::from(format!("  Done ({c3_len})")),
        Spans::from(format!("  Ideas ({c4_len})")),
        Spans::from(format!("Progress: {c3_len} / {tocomplete_total} - {percentage}%")),
    ])])
    .block(block);

    f.render_widget(list, area);
}

/// Macro to generate the app's keybindings string at compile time
macro_rules! unroll {
    (($first_a:literal, $first_b:literal), $(($a:literal, $b:literal)),*) => {
        concat!(concat!($first_a, ": ", $first_b) $(," | ", concat!($a, ": ", $b))*)
    };
}

/// Takes the app's [`State`] so [ratatui][`tui`] can render it to the
/// terminal screen
pub fn draw_ui_from_state<B: Backend>(f: &mut Frame<'_, B>, state: &mut State<'_>) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Min(10),
                Constraint::Max(10),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(f.size());

    let block = Block::default()
        .title(format!("⎸ {} ⎹", state.project_name))
        .title_alignment(Alignment::Center)
        .borders(Borders::TOP);
    f.render_widget(block, main_layout[0]);

    draw_tasks(f, main_layout[1], state);

    let info_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Min(60), Constraint::Max(60)].as_ref())
        .split(main_layout[2]);

    draw_task_info(f, info_area[0], state);
    draw_project_stats(f, info_area[1], state);

    let block = Block::default().title("KEYBINDINGS").borders(Borders::TOP);

    let foot_txt = unroll![
        ("quit", "q"),
        ("navigation", "hjkl"),
        ("move task", "HJKL"),
        ("new task", "n"),
        ("edit task", "e"),
        ("cycle edit fields", "Tab"),
        ("column top", "g"),
        ("column bottom", "G")
    ];

    let footer = Paragraph::new(foot_txt).block(block);
    f.render_widget(footer, main_layout[3]);

    if state.task_edit_state.is_some() {
        draw_task_popup(f, state, "Create Task");
    }
}
