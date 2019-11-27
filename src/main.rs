use std::io::{stdin, ErrorKind, Result};
use std::os::unix::io::AsRawFd;
use std::time::{Duration, Instant};

use crossterm::screen::RawScreen;
use libc;
use mio::{unix::SourceFd, Events, Interests, Poll, Token};

const STDIN_TOKEN: Token = Token(0);
const STDIN_BUFFER_SIZE: usize = 1_024;
const EXCLAMATION_MARK: u8 = b'!';

fn main() -> Result<()> {
    let start = Instant::now();

    println!("Paste something big, if ! is found, it quits...");

    let mut _raw = RawScreen::into_raw_mode().map_err(|_| ErrorKind::Other)?;

    let mut poll = Poll::new()?;

    let stdin = stdin();
    let stdin_raw_fd = stdin.as_raw_fd();

    let stdin_source = SourceFd(&stdin_raw_fd);

    poll.registry()
        .register(&stdin_source, STDIN_TOKEN, Interests::READABLE)?;

    let mut events = Events::with_capacity(16);
    let mut buffer = [0u8; STDIN_BUFFER_SIZE];

    let mut poll_call_count: usize = 0;
    let mut poll_stdin_event_count: usize = 0;
    let mut total_bytes_count: usize = 0;

    loop {
        print!(
            "\x1B[1GPoll call count: {} STDIN event count: {} Bytes read: {}",
            poll_call_count, poll_stdin_event_count, total_bytes_count
        );

        if total_bytes_count > 0 {
            poll_call_count += 1;
        }

        poll.poll(&mut events, Some(Duration::from_secs(0)))?;
        let tokens = events.iter().map(|e| e.token()).collect::<Vec<Token>>();

        if !tokens.contains(&STDIN_TOKEN) {
            continue;
        }

        poll_stdin_event_count += 1;

        let read_count = unsafe {
            libc::read(
                stdin_raw_fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                STDIN_BUFFER_SIZE as libc::size_t,
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

    println!(
        "\x1B[1GPoll call: {} STDIN event count: {} Bytes read: {} Time: {:?}\r",
        poll_call_count,
        poll_stdin_event_count,
        total_bytes_count,
        start.elapsed(),
    );

    Ok(())
}
