use color_eyre::Section;
use color_eyre::eyre::Context;
use log::{debug, error, info};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, sleep};
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    net::{IpAddr, Ipv4Addr, TcpListener, TcpStream},
    time::Duration,
};

use crate::progress::InstallEndGuard;
use crate::{InstallProgressEvent, InstallProgressSender};

// TODO: listen to random high-range port instead, it really doesn't matter!
// also maybe keep trying to find available port if collision happens
const HOST_HTTP_PORT: u16 = 8080;

fn urlencode(input: &str) -> String {
    const FRAGMENT_PERCENT_ENCODE_SET: &AsciiSet =
        &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
    utf8_percent_encode(input, FRAGMENT_PERCENT_ENCODE_SET).to_string()
}

#[allow(clippy::too_many_lines)]
fn serve_http(
    game_paths: &[PathBuf],
    host_ip: IpAddr,
    run_http_server: &AtomicBool,
    progress_tx: InstallProgressSender,
) -> color_eyre::Result<()> {
    let _ended_guard = InstallEndGuard { tx: &progress_tx };
    let listener = TcpListener::bind((host_ip, HOST_HTTP_PORT)).wrap_err_with(|| {
        format!(
            "Unable to bind HTTP server to host IP and port ({}:{}).",
            host_ip, HOST_HTTP_PORT
        )
    })?;
    listener.set_nonblocking(true)?;

    info!("started http server");

    let mut all_files_offset_bytes = 0;

    while run_http_server.load(Ordering::Relaxed) {
        let (mut stream, _addr) = match listener.accept() {
            Ok(tuple) => tuple,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    thread::sleep(Duration::from_millis(100));
                } else {
                    error!("error accepting incoming HTTP connection: {:?}", e);
                }
                continue;
            }
        };
        let buf_reader = BufReader::new(&stream);
        let mut lines = buf_reader.lines().map(|result| result.unwrap());
        let start_line = lines.next().unwrap();

        let mut parts = start_line.split(' ');
        let method = parts.next().unwrap();

        let Ok(requested_game_path) = percent_decode_str(&parts.next().unwrap()[1..]).decode_utf8()
        else {
            error!("invalid UTF-8 in requested game path: {}", start_line);
            deny_request(&mut stream);
            continue;
        };
        info!("Requested '{}'", requested_game_path);
        if parts.next().is_none_or(|part| part != "HTTP/1.1") {
            error!("invalid HTTP request start line: {}", start_line);
            deny_request(&mut stream);
            continue;
        }
        if !game_paths
            .iter()
            .any(|path| path == requested_game_path.as_ref())
        {
            error!(
                "requested game backup ({}) not present on host",
                requested_game_path
            );
            deny_request(&mut stream);
            continue;
        }

        let _ = progress_tx.send(InstallProgressEvent::CurrentFileName(
            Path::new(requested_game_path.as_ref())
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        ));

        let game_size = std::fs::metadata(requested_game_path.as_ref())
            .unwrap()
            .len();

        let _ = progress_tx.send(InstallProgressEvent::CurrentFileLengthBytes(game_size));

        match method {
            "GET" => {
                debug!("got GET");
                let range_line = lines.find(|line| line.starts_with("Range: ")).unwrap();
                const RANGE_PREFIX: usize = "Range: bytes=".len();
                let mut range_parts = range_line[RANGE_PREFIX..].split('-');

                let range_start: u64 = range_parts.next().unwrap().parse().unwrap();
                let range_end: u64 = range_parts.next().unwrap().parse().unwrap();
                let range_length = range_end - range_start + 1;

                let file = File::open(requested_game_path.as_ref()).unwrap();
                let mut file_reader = BufReader::new(file);
                file_reader.seek(SeekFrom::Start(range_start)).unwrap();

                let mut current_file_offset_bytes = range_start;
                let mut remaining_bytes_in_file = range_length as usize;
                let mut send_buf = [0u8; 2usize.pow(14)];

                write!(
                    stream,
                    "HTTP/1.1 206 Partial Content\r\nAccept-Ranges: bytes\r\nContent-Range: bytes: {}-{} / {}\r\nContent-Length: {}\r\n\r\n",
                    range_start, range_end, game_size, range_length
                )?;
                while remaining_bytes_in_file > 0 {
                    let chunk_size = remaining_bytes_in_file.min(send_buf.len());
                    let chunk = &mut send_buf[..chunk_size];

                    file_reader.read_exact(chunk)?;
                    stream.write_all(chunk)?;

                    remaining_bytes_in_file -= chunk_size;

                    current_file_offset_bytes += chunk_size as u64;
                    let _ = progress_tx.send(InstallProgressEvent::CurrentFileOffsetBytes(
                        current_file_offset_bytes,
                    ));

                    all_files_offset_bytes += chunk_size as u64;
                    let _ = progress_tx.send(InstallProgressEvent::AllFilesOffsetBytes(
                        all_files_offset_bytes,
                    ));
                }
                stream.flush()?;
            }
            "HEAD" => {
                debug!("got HEAD");
                respond_to_request(
                    &mut stream,
                    format!(
                        "HTTP/1.1 200 OK\r\nAccept-Ranges: bytes\r\nContent-Range: bytes: 0-{game_size}\r\nContent-Length: {game_size}\r\n\r\n"
                    ),
                );
            }
            _ => {
                error!("invalid HTTP method: {}", method);
                deny_request(&mut stream);
            }
        }
    }
    info!("CLOSING http server");
    Ok(())
}

pub fn perform_tinfoil_network_install(
    game_paths: Vec<PathBuf>,
    target_ip: Ipv4Addr,
    progress_tx: InstallProgressSender,
    cancel: Option<Arc<AtomicBool>>,
) -> color_eyre::Result<()> {
    info!("Performing network install to {}", target_ip);

    let mut keepalive_stream = TcpStream::connect((target_ip, 2000)).wrap_err_with(|| format!("Target device at {target_ip} (hopefully Nintendo Switch!?) is refusing connections"))
        .suggestion("Ensure the Nintendo Switch is awake and in Awoo Installer 'Install Over LAN or internet'")
        .with_suggestion(|| format!(
            "Ensure the Nintendo Switch is connected to the same network as this computer, and that the target IP ({target_ip}) seems correct.",
        )
        ).suggestion("Restart Awoo Installer, sometimes it enters a fucked up state..")?;
    keepalive_stream.set_read_timeout(Some(Duration::from_secs(1)))?;

    let host_ip = keepalive_stream.local_addr().map(|addr| addr.ip())?;
    debug!("found host ip: {}", host_ip);
    let base_url = format!("http://{}:{}/", host_ip, HOST_HTTP_PORT);
    let urls_with_newlines = game_paths.iter().fold(String::new(), |acc, path| {
        acc + &base_url + &urlencode(path.to_str().unwrap()) + "\n"
    });

    let all_files_length_bytes = game_paths
        .iter()
        .map(|path| path.metadata().unwrap().len())
        .sum::<u64>();
    let _ = progress_tx.send(InstallProgressEvent::AllFilesLengthBytes(
        all_files_length_bytes,
    ));

    let run_http_server = Arc::new(AtomicBool::new(true));
    let run_http_server_thread = Arc::clone(&run_http_server);
    let http_thread = thread::spawn(move || {
        serve_http(
            &game_paths,
            host_ip,
            run_http_server_thread.as_ref(),
            progress_tx,
        )
    });
    info!("spawned HTTP thread");

    keepalive_stream.write_all(
        &[
            &(u32::try_from(urls_with_newlines.len()).unwrap()).to_be_bytes(),
            urls_with_newlines.as_bytes(),
        ]
        .concat(),
    )?;

    debug!("sent initiating data {}", &urls_with_newlines);

    let mut keepalive_buf = Vec::new();
    loop {
        info!("polling keepalive");
        if cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed)) {
            info!("cancelling network install");
            break;
        }
        match keepalive_stream.read_to_end(&mut keepalive_buf) {
            Ok(_) => {
                debug!("keepalive stream closed by peer");
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                sleep(Duration::from_millis(600));
            }
            Err(e) => {
                error!("error reading from keepalive stream: {:?}", e);
                break;
            }
        }
    }
    run_http_server.store(false, Ordering::Relaxed);
    http_thread.join().expect("joining http server thread")
}

fn deny_request(stream: &mut TcpStream) {
    respond_to_request(stream, "HTTP/1.1 400 Bad Request\r\n\r\n");
}

fn respond_to_request(stream: &mut TcpStream, buf: impl AsRef<[u8]>) {
    if let Err(e) = stream.write_all(buf.as_ref()) {
        error!("responding to HTTP request: {:?}", e);
    }
}
