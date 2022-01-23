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
    widgets::ListState,
    Terminal,
};

mod ui;

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

//This struct holds the current state of the app.
pub struct App<'a> {
    times: StatefulList<u32>,
    keybinds: [(&'a str, &'a str); 20],
    time: i32,
    timing_status: TimerStatus,
    ticks_with_no_key: u32,
    key_released_since_timer_start: bool,
}
/*
Set starting values and define functions
*/
impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            times: StatefulList::with_items(vec![
                1,
                2,
                4, // In the wrong order to check if it displays it this way
                3,
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
            ],
            time: 0,
            timing_status: TimerStatus::PAUSED,
            ticks_with_no_key: 0,
            key_released_since_timer_start: false,
        }
    }

    pub fn update_timer(&mut self, key_pressed_in_tick: bool) {
        match self.timing_status {
            TimerStatus::COUNTDOWN => self.time -= 1,
            TimerStatus::COUNTUP => self.time += 1,
            TimerStatus::PAUSED => {}
        }

        if key_pressed_in_tick == true {
            if self.timing_status == TimerStatus::PAUSED && self.ticks_with_no_key == 0{
                self.time = 1500;
                self.timing_status = TimerStatus::COUNTDOWN;
            }
            self.ticks_with_no_key = 0;
            return
        }
        // No key was pressed this tick

        if self.timing_status != TimerStatus::COUNTDOWN {return}
        // No key was pressed this tick and the timer is counting down

        self.ticks_with_no_key += 1;

        /*
        We have to wait 600 ms because the termnal receives repeating keys, so if it's pressed again within 600 ms we can assume it is still being held
        */
        if self.ticks_with_no_key > 60 {return}
        // The key was not pressed for 600ms (i.e. The key was released) and the timer is counting down.

        self.ticks_with_no_key = 0;
        self.time = 0;
        self.timing_status = TimerStatus::COUNTUP;
    }
}

/*
Setup, run the program and cleanup
*/
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

/*
Main loop
*/
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut timer_key_pressed_in_tick = false;
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(' ') => timer_key_pressed_in_tick = true,
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.times.unselect(),
                    KeyCode::Down => app.times.next(),
                    KeyCode::Up => app.times.previous(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();

            app.update_timer(timer_key_pressed_in_tick);
            timer_key_pressed_in_tick = false;
        }
    }
}