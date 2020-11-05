// for input
use std::{error::Error, io};
use termion::{event::Key, raw::IntoRawMode, screen::AlternateScreen};
use termion::input::TermRead;

use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    text::Span,
    Terminal,
};


use std::time::Duration;

use std::sync::mpsc;
use std::collections::HashSet;
use std::{thread, time};

const END_PLAYING_SCREEN_X : usize = 74;
const END_SCREEN_Y : usize = 54;

const BLOCK_HEIGHT : usize = 3;
const BLOCK_WIDTH : usize = BLOCK_HEIGHT * 2;
const LEFT_THRESHOLD : usize = 0;
const RIGHT_THRESHOLD : usize = END_PLAYING_SCREEN_X / BLOCK_WIDTH;
const BOTTOM_THRESHOLD : usize = END_SCREEN_Y / BLOCK_HEIGHT;
const INITIAL_TICK_TIME_MS : i128 = 1000;
struct Events {
    receiver: mpsc::Receiver<Key>
}

impl Events {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        // just spawn a thread that will listen for key presses
        thread::spawn(move || {
            for key_press in std::io::stdin().keys() {
                if let Ok(key) = key_press {
                    if let Err(err) = sender.send(key) {
                        eprintln!("Whops {}", err);
                    }
                }
            }
        });

        Events {receiver}
    }
}

pub mod matrix;
pub mod piece;
pub mod pile;
pub mod tetris;

use pile::Pile;
use matrix::Matrix;
use piece::Piece;
use tetris::Tetris;

fn get_color_for_figure(piece_type: &piece::PieceType) -> Color {
    match piece_type {
        piece::PieceType::Square => Color::Red,
        piece::PieceType::L => Color::Green,
        piece::PieceType::Straight => Color::LightBlue,
        piece::PieceType::ReverseL => Color::Blue,
        piece::PieceType::T => Color::LightYellow,
        piece::PieceType::Worm => Color::Yellow,
        piece::PieceType::ReverseWorm => Color::Magenta,
    }
}

fn render_playing_piece(piece: &Piece, block: &Block, color_hint: Option<Color>,
                frame: &mut tui::terminal::Frame<TermionBackend<AlternateScreen
                <termion::raw::RawTerminal<std::io::Stdout>>>>) {
    piece.get_positions().iter().map(|(i, j)| {
        tui::layout::Rect{
            x: (*j * BLOCK_WIDTH) as u16,
            y: (*i * BLOCK_HEIGHT) as u16,
            width: BLOCK_WIDTH as u16, height: BLOCK_HEIGHT as u16}
    }).for_each(|rect| {
        frame.render_widget(block.clone()
             .style(Style::default()
                    .bg(color_hint.unwrap_or(
                            get_color_for_figure(&piece.piece_type)))), rect);
    });
}

fn render_utility_piece(piece: &Piece, block: &Block, color_hint: Option<Color>,
                frame: &mut tui::terminal::Frame<TermionBackend<AlternateScreen
                <termion::raw::RawTerminal<std::io::Stdout>>>>) {
    piece.get_positions().iter().map(|(i, j)| {
        tui::layout::Rect{
            x: END_PLAYING_SCREEN_X as u16+ 2 + (*j * 6) as u16,
            y: (*i * 3) as u16,
            width: 6 as u16, height: 3 as u16}
    }).for_each(|rect| {
        frame.render_widget(block.clone()
             .style(Style::default()
                    .bg(color_hint.unwrap_or(
                            get_color_for_figure(&piece.piece_type)))), rect);
    });
}

fn main() -> Result<(), Box<dyn Error>> {

    let stdout = io::stdout().into_raw_mode()?;
    //let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();
    let mut last = time::SystemTime::now();

    let pile: Pile =
        Pile {field: Matrix {col_count: RIGHT_THRESHOLD,
                             backing: vec![false; RIGHT_THRESHOLD * BOTTOM_THRESHOLD]},
                             set: HashSet::new()};

    let current_piece = Piece::new_random_piece_at(
        ((LEFT_THRESHOLD + RIGHT_THRESHOLD) / 2 - 2) as i16, 0);

    // utility pieces
    let next_piece = Piece::new_random_piece_at(0, 1);
    let spare_piece = Piece::new_random_piece_at(0, 7);

    let spare_used = false;

    let mut game = Tetris {
        pile,
        current_piece,
        next_piece,
        spare_piece,
        spare_used,
        score: 0,
        last_combo: 0,
        tick_time: INITIAL_TICK_TIME_MS,
    };

    loop {
        let mut projected = game.current_piece.clone();

        while !projected.touches_on_bottom(&game.pile) {
            projected.move_down_unsafe();
        }
    
        // render tui
        terminal.draw(|f| {
            let screen = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length((RIGHT_THRESHOLD * BLOCK_WIDTH) as u16),
                              Constraint::Length(30),
                              Constraint::Min(0)].as_ref())
                .split(f.size());

            let block = Block::default()
                .title("Tetris")
                .borders(Borders::ALL).border_type(BorderType::Rounded);
            f.render_widget(block, screen[0]);

            let other = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Ratio(1, 3),
                              Constraint::Ratio(1, 3),
                              Constraint::Ratio(1, 3)].as_ref())
                .split(screen[1]);

            let block = Block::default()
                .title("Next")
                .borders(Borders::ALL);
            f.render_widget(block, other[0]);

            let block = Block::default()
                .title("Spare")
                .borders(Borders::ALL);
            f.render_widget(block, other[1]);

            let block = Block::default()
                .title("Score")
                .borders(Borders::ALL);
            f.render_widget(block, other[2]);


            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(Color::DarkGray));

            render_utility_piece(&game.spare_piece, &block.clone(), None, f);
            render_utility_piece(&game.next_piece, &block.clone(), None, f);
            render_playing_piece(&projected,
                         &block.clone(), Some(Color::Gray), f);
            render_playing_piece(&game.current_piece, &block, None, f);

            let potentionally_completed_lines = game.pile.get_complete_lines_with(
                &projected.get_positions());

            game.pile.set.iter().map(|(i, j)| {
                let color : Color;
                if potentionally_completed_lines.contains(i) {
                    color = Color::Rgb(200, 200, 200);
                } else {
                    color = Color::DarkGray;
                }
                (tui::layout::Rect{
                    x: *j as u16 * BLOCK_WIDTH as u16,
                    y: *i as u16 * BLOCK_HEIGHT as u16,
                    width: BLOCK_WIDTH as u16, height: BLOCK_HEIGHT as u16},
                    color)
            }).for_each(|(rect, color)| {
                f.render_widget(block.clone().style(Style::default().bg(color)), rect);
            });

            let block = Block::default()
                .title(Span::styled(format!("Score: {}", game.score),
                Style::default().add_modifier(Modifier::BOLD)));
            f.render_widget(block.clone(), tui::layout::Rect {
                x: other[2].x + 2,
                y: other[2].y + 2,
                height: other[2].height - 2,
                width: other[2].width - 2,
            });

            let block = Block::default()
                .title(Span::styled(format!("Last combo: {}",  game.last_combo),
                Style::default().add_modifier(Modifier::BOLD)));
            f.render_widget(block.clone(), tui::layout::Rect {
                x: other[2].x + 2,
                y: other[2].y + 3,
                height: other[2].height - 3,
                width: other[2].width - 2,
            });
        })?;

        let elapsed = time::SystemTime::now().duration_since(last)?;
        let timeout = std::cmp::max(0, game.tick_time - elapsed.as_millis() as i128) as u64;

        match events.receiver.recv_timeout(Duration::from_millis(timeout)) {
            Ok(key) => {
                match key {
                    Key::Char('q') => break,
                    Key::Left => game.move_left(),
                    Key::Right => game.move_right(),
                    Key::Down => {
                        if game.can_move_down() {
                            last = time::SystemTime::now();
                        }
                        game.move_down();
                    },
                    Key::Char('a') => game.safe_rotate_counter_clockwise(),
                    Key::Char('d') => game.safe_rotate_clockwise(),
                    Key::Char('s') => game.use_spare(),
                    Key::Char(' ') => game.drop_to_bottom(),
                    _ => continue
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if game.finishing_move_down() {
                    break;
                }
                last = time::SystemTime::now();
            },
            _ => eprintln!("WTF")
        }
    }

    terminal.draw(|f| {
        let screen = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Ratio(1, 3),
                          Constraint::Ratio(1, 3),
                          Constraint::Ratio(1, 3)].as_ref())
            .split(f.size());

        let screen = Layout::default()
            .direction(Direction::Horizontal)
            .margin(5)
            .constraints([Constraint::Ratio(1, 3),
                          Constraint::Ratio(1, 3),
                          Constraint::Ratio(1, 3)].as_ref())
            .split(screen[1]);

        let paragraph = Paragraph::new(format!("Your score is {}", game.score))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, screen[1]);
    })?;

    std::thread::sleep(time::Duration::from_secs(2));
    Ok(())
}
