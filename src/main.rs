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
use std::{thread, time};

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

use piece::Piece;
use tetris::Tetris;

fn get_tui_color(piece_color: piece::PieceColor) -> Color {
    match piece_color {
        piece::PieceColor::Red => Color::Red,
        piece::PieceColor::Green => Color::Green,
        piece::PieceColor::LightBlue => Color::LightBlue,
        piece::PieceColor::Blue => Color::Blue,
        piece::PieceColor::LightYellow => Color::LightYellow,
        piece::PieceColor::Yellow => Color::Yellow,
        piece::PieceColor::Magenta => Color::Magenta,
    }
}

fn render_playing_piece(piece: &Piece, block: &Block, color_hint: Option<Color>,
                frame: &mut tui::terminal::Frame<TermionBackend<AlternateScreen
                <termion::raw::RawTerminal<std::io::Stdout>>>>) {
    piece.get_positions().iter().map(|(i, j)| {
        tui::layout::Rect{
            x: (*j * tetris::BLOCK_WIDTH) as u16,
            y: (*i * tetris::BLOCK_HEIGHT) as u16,
            width: tetris::BLOCK_WIDTH as u16, height: tetris::BLOCK_HEIGHT as u16}
    }).for_each(|rect| {
        frame.render_widget(block.clone()
             .style(Style::default()
                    .bg(color_hint.unwrap_or(
                            get_tui_color(piece::get_piece_color(&piece.piece_type))))), rect);
    });
}

fn render_utility_piece(piece: &Piece, block: &Block, color_hint: Option<Color>,
                frame: &mut tui::terminal::Frame<TermionBackend<AlternateScreen
                <termion::raw::RawTerminal<std::io::Stdout>>>>) {
    piece.get_positions().iter().map(|(i, j)| {
        tui::layout::Rect{
            x: tetris::END_PLAYING_SCREEN_X as u16+ 2 + (*j * 6) as u16,
            y: (*i * 3) as u16,
            width: 6 as u16, height: 3 as u16}
    }).for_each(|rect| {
        frame.render_widget(block.clone()
             .style(Style::default()
                    .bg(color_hint.unwrap_or(
                            get_tui_color(piece::get_piece_color(&piece.piece_type))))), rect);
    });
}

fn main() -> Result<(), Box<dyn Error>> {

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();
    let mut game = Tetris::new();

    loop {
        let mut projected = game.current_piece.clone();

        while !projected.touches_on_bottom(&game.pile) {
            projected.move_down_unsafe();
        }
    
        // render tui
        terminal.draw(|f| {
            let screen = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(
                        (tetris::RIGHT_THRESHOLD * tetris::BLOCK_WIDTH) as u16),
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

            game.pile.map.iter().map(|((i, j), piece_color)| {
                let color : Color;
                if potentionally_completed_lines.contains(i) {
                    color = Color::Rgb(200, 200, 200);
                } else {
                    color = get_tui_color(piece_color.clone());// change this to Color::DarkGrey if you don't like colored pile
                }
                (tui::layout::Rect{
                    x: *j as u16 * tetris::BLOCK_WIDTH as u16,
                    y: *i as u16 * tetris::BLOCK_HEIGHT as u16,
                    width: tetris::BLOCK_WIDTH as u16, height: tetris::BLOCK_HEIGHT as u16},
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

            let block = Block::default()
                .title(Span::styled(format!("Tick speed: {}",  game.get_tick_speed()),
                Style::default().add_modifier(Modifier::BOLD)));
            f.render_widget(block.clone(), tui::layout::Rect {
                x: other[2].x + 2,
                y: other[2].y + 4,
                height: other[2].height - 4,
                width: other[2].width - 2,
            });
        })?;

        if game.is_over() {
            break;
        }
        match events.receiver.recv_timeout(Duration::from_millis(
                game.get_timeout() as u64)) {
            Ok(key) => {
                match key {
                    Key::Char('q') => break,
                    Key::Left => game.move_left(),
                    Key::Right => game.move_right(),
                    Key::Down => game.move_down(),
                    Key::Char('a') => game.safe_rotate_counter_clockwise(),
                    Key::Char('d') => game.safe_rotate_clockwise(),
                    Key::Char('s') => game.use_spare(),
                    Key::Char(' ') => game.drop_to_bottom(),
                    _ => continue
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if game.can_move_down() {
                    game.move_down();
                } else if game.should_finish_turn() {
                    game.finish_turn();
                } else {
                    game.advance_stuck();
                }
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
