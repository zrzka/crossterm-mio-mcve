use std::io::{ErrorKind, Result};
use std::time::{Duration, Instant};

use crossterm::screen::RawScreen;
use libc;
use mio::{unix::EventedFd, Events, Poll, PollOpt, Ready, Token};

const TTY_TOKEN: Token = Token(0);
const TTY_BUFFER_SIZE: usize = 1_024;
const EXCLAMATION_MARK: u8 = b'!';

fn main() -> Result<()> {
    let start = Instant::now();

    println!("Paste something big, if ! is found, it quits...");

    let mut _raw = RawScreen::into_raw_mode().map_err(|_| ErrorKind::Other)?;

    let poll = Poll::new()?;

    let tty_raw_fd = libc::STDIN_FILENO;
    let tty_evented = EventedFd(&tty_raw_fd);
    poll.register(&tty_evented, TTY_TOKEN, Ready::readable(), PollOpt::level())?;

    let mut events = Events::with_capacity(16);
    let mut buffer = [0u8; TTY_BUFFER_SIZE];

    let mut poll_call_count: usize = 0;
    let mut poll_tty_event_count: usize = 0;
    let mut total_bytes_count: usize = 0;

    loop {
        print!(
            "\x1B[1GPoll call count: {} Tty event count: {} Bytes read: {}",
            poll_call_count, poll_tty_event_count, total_bytes_count
        );

        if total_bytes_count > 0 {
            poll_call_count += 1;
        }

        let count = poll.poll(&mut events, Some(Duration::from_secs(0)))?;

        if count > 0 {
            let tokens = events.iter().map(|e| e.token()).collect::<Vec<Token>>();

            if !tokens.contains(&TTY_TOKEN) {
                continue;
            }

            poll_tty_event_count += 1;

            let read_count = unsafe {
                libc::read(
                    tty_raw_fd,
                    buffer.as_mut_ptr() as *mut libc::c_void,
                    TTY_BUFFER_SIZE as libc::size_t,
                ) as isize
            };

            if read_count == -1 {
                continue;
            }

            total_bytes_count += read_count as usize;

            if buffer[..read_count as usize].contains(&EXCLAMATION_MARK) {
                break;
            }
        }
    }

    println!(
        "\x1B[1GPoll: {} Tty event: {} Total bytes: {} Time: {:?}\r",
        poll_call_count,
        poll_tty_event_count,
        total_bytes_count,
        start.elapsed(),
    );

    Ok(())
}
