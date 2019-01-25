use std::io::{stdin, stdout, Stdout, Write};
use std::sync::{Arc, Mutex};

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::screen::AlternateScreen;

use crate::term;
use crate::term::ScreenExt;

use crate::widget::Widget;

pub struct Window<T>
where T: Widget
{
    pub selection: usize,
    pub widget: T,
    pub status: Arc<Mutex<Option<String>>>,
    pub screen: AlternateScreen<Box<Stdout>>,
    pub dimensions: (u16, u16),
}


impl<T> Window<T>
where
    T: Widget
{
    pub fn new(widget: T) -> Window<T> {
        let mut screen = AlternateScreen::from(Box::new(stdout()));
        screen.cursor_hide();
        let mut win = Window::<T> {
            selection: 0,
            widget: widget,
            status: STATUS_BAR_CONTENT.clone(),
            screen: screen,
            dimensions: termion::terminal_size().unwrap(),
        };
        win.widget.refresh();
        win
    }

    pub fn draw(&mut self) {
        let output = self.widget.get_drawlist() + &self.widget.get_header_drawlist();
        self.screen.write(output.as_ref()).unwrap();

        self.screen.flush().unwrap();
        Self::draw_status();
    }

    pub fn show_status(status: &str) {
        show_status(status);
    }

    pub fn draw_status() {
        draw_status();
    }

    pub fn clear_status() {
        Self::show_status("");
    }

    

    pub fn handle_input(&mut self) {
        self.draw();
        for event in stdin().events() {
            Self::clear_status();
            self.draw();
            let event = event.unwrap();
            self.widget.on_event(event);
            self.draw();
        }
    }
}

impl<T> Drop for Window<T>
where
     T: Widget
{
    fn drop(&mut self) {
        // When done, restore the defaults to avoid messing with the terminal.
        self.screen.write(format!("{}{}{}{}{}",
                                  termion::screen::ToMainScreen,
                                  termion::clear::All,
                                  termion::style::Reset,
                                  termion::cursor::Show,
                                  termion::cursor::Goto(1, 1)).as_ref()).unwrap();
    }
}

lazy_static! {
    static ref STATUS_BAR_CONTENT: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

pub fn draw_status() {
    let xsize = term::xsize() as u16;
    let status = STATUS_BAR_CONTENT.try_lock().unwrap().clone();

    status.or(Some("".to_string())).and_then(|status| {
        write!(
            stdout(),
            "{}{}{:xsize$}{}{}",
            term::move_bottom(),
            term::status_bg(),
            " ",
            term::move_bottom(),
            status,
            xsize = xsize as usize
        ).ok()
    });
    stdout().flush().unwrap();
}

pub fn show_status(status: &str) {
        {
            let mut status_content = STATUS_BAR_CONTENT.try_lock().unwrap();
            *status_content = Some(status.to_string());
        }
        draw_status();
}

pub fn minibuffer(query: &str) -> Option<String> {
    show_status(&(query.to_string() + ": "));
    let mut buffer = "".to_string();

    for key in stdin().events() {
        match key {
            Ok(Event::Key(key)) => {
                match key {
                    Key::Esc => { return None },
                    Key::Char('\n') => { return Some(buffer) },
                    Key::Backspace => { buffer.pop(); },
                    Key::Char(key) => { buffer = buffer + &format!("{}", key); },
                    _ => {}
                }
            },
            _ => {}
        }
        show_status(&(query.to_string() + ": " + &buffer));
                
    };
    None
}
