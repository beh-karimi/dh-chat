use std::io::{Write, self};
use crossterm::{
    QueueableCommand, cursor, ExecutableCommand,
    style::{self, Color, Print, SetForegroundColor},
    terminal::{enable_raw_mode, disable_raw_mode, size, ClearType, self, EnterAlternateScreen, LeaveAlternateScreen},
    event::{Event, KeyCode, EventStream,},
};
use tokio::{sync::{mpsc, watch}, select};
use futures::{future::FutureExt, StreamExt};

const RECIPIENT_COLOR: Color = Color::Yellow;
const USER_COLOR: Color = Color::Green;

struct Msg {
    from: u8,
    content: String,
}
impl Msg {
    fn new(from: u8, content: String) -> Self {
        Msg { from, content }
    }
}
pub enum UiAction {Nothing, Exit, NewChar(char), DelChar, Send, Redraw}

pub struct Ui {
    messages: Vec<Msg>,
    inp_buf: String,
    pub action_tx: mpsc::Sender<UiAction>,
    action_rx: mpsc::Receiver<UiAction>,
}

impl Ui {
    pub fn new() -> Self {
        let messages:Vec<Msg> = vec![];
        let inp_buf = String::new();
        let (action_tx, action_rx) = mpsc::channel::<UiAction>(20);

        Ui {messages, inp_buf, action_tx, action_rx}
    }

    pub async fn run(&mut self, screen: &mut dyn std::io::Write) -> io::Result<()> {
        screen.execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        screen.flush()?;

        let (cancel_tx, cancel_rx) = watch::channel(false);
        let a_tx_c = self.action_tx.clone();
        tokio::spawn(async move {
            input_handler(a_tx_c, cancel_rx).await.unwrap();
        });

        loop {
            let action = self.action_rx.recv().await;
            match action {
                Some(a) => { match a {
                    UiAction::Nothing => {},
                    UiAction::Send => {

                    }
                    UiAction::NewChar(c) => {
                        self.inp_buf.push(c);
                        redraw(&self.messages, &self.inp_buf, screen)?;
                    },
                    UiAction::DelChar => {
                        self.inp_buf.pop();
                        redraw(&self.messages, &self.inp_buf, screen)?;
                    },
                    UiAction::Redraw => redraw(&self.messages, &self.inp_buf, screen)?,
                    UiAction::Exit => {
                        cancel_tx.send(true).unwrap();
                        break;
                    },
                }}
                None => todo!(),
            };
        }
        screen.execute(LeaveAlternateScreen)?;
        disable_raw_mode()
    }
}

pub async fn input_handler(action_tx: mpsc::Sender<UiAction>,
        mut cancel_rx: watch::Receiver<bool>) -> io::Result<()>
    {
        let mut reader = EventStream::new();
        loop {
            select! {
                _ = cancel_rx.changed() => { break; }
                event = reader.next().fuse() => {
                    if let Event::Key(k) = event.unwrap().unwrap() {
                        action_tx.send(key_handle(k.code)).await.unwrap();
                    }
                }
            }
        }
        Ok(())
    }

fn key_handle(k: KeyCode) -> UiAction {
    match k {
        KeyCode::Esc => UiAction::Exit,
        KeyCode::Enter => UiAction::Send,
        KeyCode::Backspace => UiAction::DelChar,
        KeyCode::Char(c) => UiAction::NewChar(c),
        _ => UiAction::Nothing,
    }
}

fn redraw(messages: &Vec<Msg>, msg_inp_buf: &str, screen: &mut dyn Write) -> io::Result<()> {
    let size = size()?;

    // Display an error and stop and return if the terminal is too small
    if size.0 < 10 || size.1 < 3 {
        screen.queue(terminal::Clear(ClearType::All))?
            .queue(cursor::MoveTo(0,0))?
            .queue(Print("small terminal!"))?;
        return Ok(());
    }

    let (mut printable_messages, overflow_lines) = 
        get_visible_messages(size, msg_inp_buf, messages);
    
    screen.queue(terminal::Clear(ClearType::All))?
        .queue(cursor::MoveTo(0,0))?;
    
    draw_messages(screen, &mut printable_messages, msg_inp_buf, overflow_lines, size)
}

fn get_visible_messages<'a>(term_size: (u16,u16), msg_inp_buf: &str, messages: &'a [Msg])
    -> (Vec<&'a Msg>,i32)
{
    let inp_buf_line_no = msg_inp_buf.lines().count() as u16;

    let mut lines = (term_size.1 - inp_buf_line_no) as i32;
    let mut printable_messages: Vec<&Msg> = vec![];
    for m in messages.iter().rev() {
        printable_messages.push(&m);
        lines -= m.content.len() as i32 / term_size.0 as i32 + 1;
        if lines <= 0 {
            break;
        }
    }
    (printable_messages, -lines)
}

fn draw_messages(screen: &mut dyn Write, messages: &mut Vec<&Msg>,
    msg_inp_buf: &str, overflow_lines: i32, term_size: (u16,u16)) -> io::Result<()>
{
    // Draw the visible part of the top most message
    if overflow_lines>0 && messages.len() > 0 {
        let msg = messages.pop().unwrap();

        if msg.from==0 {
            screen.queue(style::SetForegroundColor(RECIPIENT_COLOR))?;
        } else {
            screen.queue(style::SetForegroundColor(USER_COLOR))?;
        }
       screen.queue(Print(msg.content.chars().rev()
                          .take(term_size.0 as usize).collect::<String>()))?
            .queue(style::SetForegroundColor(Color::Reset))?
            .queue(Print("\n\r"))?;
    }

    // Draw the rest of the visible messages
    for msg in messages.iter().rev() {
        if msg.from==0 {
            screen.queue(style::SetForegroundColor(USER_COLOR))?;
        } else {
            screen.queue(style::SetForegroundColor(RECIPIENT_COLOR))?;
        }
        screen.queue(Print(&msg.content))?
            .queue(SetForegroundColor(Color::Reset))?
            .queue(Print("\n\r"))?;
    }

    // Draw the input box
    screen.queue(cursor::MoveTo(0,term_size.1-1))?
        .queue(style::SetForegroundColor(Color::Green))?
        .queue(Print(msg_inp_buf.trim()))?
        .queue(style::SetForegroundColor(Color::Reset))?;
    Ok(())
}
