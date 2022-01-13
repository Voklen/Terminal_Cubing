use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

#[derive(PartialEq)]

enum TimerStatus {
    COUNTDOWN,
    COUNTUP,
    PAUSED,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App<'a> {
    items: StatefulList<(&'a str, usize)>,
    keybinds: [(&'a str, &'a str); 26],
    time: i32,
    timing_status: TimerStatus,
    ticks_with_no_key: u32,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(vec![
                ("Item0", 1),
                ("Item1", 2),
                ("Item2", 1),
                ("Item3", 3),
                ("Item4", 1),
                ("Item5", 4),
                ("Item6", 1),
            ]),
            keybinds: [
                ("Quit", "q"),
                ("Event2", "INFO"),
                ("Event3", "CRITICAL"),
                ("Event4", "ERROR"),
                ("Event5", "INFO"),
                ("Event6", "INFO"),
                ("Event7", "WARNING"),
                ("Event8", "INFO"),
                ("Event9", "INFO"),
                ("Event10", "INFO"),
                ("Event11", "CRITICAL"),
                ("Event12", "INFO"),
                ("Event13", "INFO"),
                ("Event14", "INFO"),
                ("Event15", "INFO"),
                ("Event16", "INFO"),
                ("Event17", "ERROR"),
                ("Event18", "ERROR"),
                ("Event19", "INFO"),
                ("Event20", "INFO"),
                ("Event21", "WARNING"),
                ("Event22", "INFO"),
                ("Event23", "INFO"),
                ("Event24", "WARNING"),
                ("Event25", "INFO"),
                ("Event26", "INFO"),
            ],
            time: 0,
            timing_status: TimerStatus::PAUSED,
            ticks_with_no_key: 0,
        }
    }

    pub fn space(&mut self) {
        if self.timing_status != TimerStatus::COUNTDOWN {
            self.time = 1500;
            self.timing_status = TimerStatus::COUNTDOWN;
        }
    }

    pub fn update_timer(&mut self, key_pressed_in_tick: bool) {
        match self.timing_status {
            TimerStatus::COUNTDOWN => self.time -= 1,
            TimerStatus::COUNTUP => self.time += 1,
            TimerStatus::PAUSED => {}
        }

        if key_pressed_in_tick == true { // Stops if a key was pressed
            self.ticks_with_no_key = 0;
            return
        }
        if self.timing_status != TimerStatus::COUNTDOWN {return} // Stops if the timer is not counting down
        
        // We have to wait 600 ms because the termnal receives repeating keys, so if it's pressed again within 600 ms we can assume it is still being held
        self.ticks_with_no_key += 1;
        if self.ticks_with_no_key > 60 { // If no key was pressed and the timer is counting down. i.e. The spacebar was released.
            self.ticks_with_no_key = 0;
            self.time = 0;
            self.timing_status = TimerStatus::COUNTUP;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(10);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut key_pressed_in_tick = false;
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                key_pressed_in_tick = true;
                match key.code {
                    KeyCode::Char(' ') => app.space(),
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();

            app.update_timer(key_pressed_in_tick);
            key_pressed_in_tick = false;
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

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
