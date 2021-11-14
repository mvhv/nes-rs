use std::io::stodout;
use tui::{Terminal, backend::CrosstermBackend};

pub fn start_tui() -> Result<(), Error>{
    // terminal setup
    crosterm::terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()));
    terminal.clear()?;
    terminal.hide_cursor()?;

    loop {
        // TODO: TUI Event loop
        terminal.draw(|frame| draw_tui(frame, ))?;
        break;
    }

    // terminal tear-down
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    
    Ok(())
}