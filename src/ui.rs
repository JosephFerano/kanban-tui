use tui::backend::{Backend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::{Frame};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use crate::types::{Project, Task};

fn draw_tasks<B: Backend>(f: &mut Frame<B>, columns: Vec<Rect>, tasks: &Vec<Task>) {
    let ts: Vec<ListItem> = tasks.iter().map(|t| {
        ListItem::new(vec![Spans::from(Span::raw(&t.title))])
    }).collect();

    let cols = &["DONE", "TODO", "IN-PROGRESS", "TESTING", "BACKLOG"];
    let blocks: Vec<Block> =
        columns.iter().enumerate()
            .map(|(i, col)| {
                Block::default()
                    .title(cols[i])
                    .borders(Borders::ALL)
            }).collect();
    let l1 = List::new(ts).block(blocks[0].clone());
    let l2 = List::new(vec![]).block(blocks[1].clone());
    let l3 = List::new(vec![]).block(blocks[2].clone());
    let l4 = List::new(vec![]).block(blocks[3].clone());
    let l5 = List::new(vec![]).block(blocks[4].clone());
    f.render_widget(l1, columns[0]);
    f.render_widget(l2, columns[1]);
    f.render_widget(l3, columns[2]);
    f.render_widget(l4, columns[3]);
    f.render_widget(l5, columns[4]);
}

pub fn draw<B: Backend>(f: &mut Frame<B>, project: &mut Project) {
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

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ].as_ref()
        )
        .split(main_layout[1]);

    let block = Block::default()
        .title("KANBAN BOARD")
        .borders(Borders::ALL);
    f.render_widget(block, main_layout[0]);

    draw_tasks(f, columns, &project.tasks);

    let block = Block::default()
        .title("TASK INFO")
        .borders(Borders::ALL);
    f.render_widget(block, main_layout[2]);

    let block = Block::default()
        .title("FOOTER")
        .borders(Borders::ALL);

    let foot_txt =
        Paragraph::new("Press 'q' to quit")
            .block(block);
    f.render_widget(foot_txt, main_layout[3]);
}