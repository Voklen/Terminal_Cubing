/*
Basically the HTML/CSS of the program
*/

use crate::App;
use tui::{
    backend::Backend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};


pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(50),
                Constraint::Percentage(20),
            ].as_ref(),
        )
        .split(f.size());

    let ver_chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .direction(Direction::Vertical)
        .split(chunks[0]);
    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(i.0)];
            for _ in 0..i.1 {
                lines.push(Spans::from(Span::styled(
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                    Style::default().add_modifier(Modifier::ITALIC),
                )));
            }
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, ver_chunks[1], &mut app.items.state);

    // This is the central timer section
    let mut centeral_time = app.time.to_string();
    match centeral_time.len() {
        0 => {centeral_time = "0.00".to_owned()},
        1 => {centeral_time = "0.0".to_owned() + &centeral_time},
        2 => {centeral_time = "0.".to_owned() + &centeral_time},
        _ => {centeral_time.insert(app.time.to_string().len()-2, '.');}
    }
    let text = vec![
        Spans::from(Span::styled(
            centeral_time,
            Style::default().add_modifier(Modifier::ITALIC),
        )),
        Spans::from(Span::styled("Second line", Style::default().fg(Color::Red))),
    ];
    let time_text = tui::widgets::Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .alignment(tui::layout::Alignment::Center)
        .wrap(tui::widgets::Wrap { trim: true });
    let timer_margin = tui::layout::Margin {
        vertical: 10,
        horizontal: 30,
    };
    chunks[1].inner(&timer_margin);
    f.render_widget(time_text, chunks[1].inner(&timer_margin));

    let keybinds: Vec<ListItem> = app
        .keybinds
        .iter()
        .rev()
        .map(|&(event, level)| {
            // Colorcode the level depending on its type
            let s = match level {
                "CRITICAL" => Style::default().fg(Color::Red),
                "ERROR" => Style::default().fg(Color::Magenta),
                "WARNING" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Blue),
                _ => Style::default(),
            };
            // Add a example datetime and apply proper spacing between them
            let header = Spans::from(vec![
                Span::styled(format!("{:<9}", level), s),
                Span::raw(" "),
                Span::styled(event, Style::default().add_modifier(Modifier::ITALIC)),
            ]);
            // Add the line to list
            ListItem::new(header)
        })
        .collect();
    // Border formating
    let events_list = List::new(keybinds)
        .block(Block::default().borders(Borders::ALL).title("Keybinds"))
        .start_corner(Corner::BottomLeft);
    f.render_widget(events_list, chunks[2]);
}
