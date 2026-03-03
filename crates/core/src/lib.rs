use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::{
    Section,
    eyre::{bail, eyre},
};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info, warn};
use nusb::InterfaceInfo;
use nusb::{
    Endpoint, MaybeFuture, list_devices,
    transfer::{Buffer, Bulk, In, Out, TransferError},
};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, sleep};
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpListener, TcpStream, UdpSocket},
    os::unix::fs::MetadataExt,
    path::Path,
    time::Duration,
};

mod tinfoil_command_types {
    pub const RESPONSE: [u8; 4] = 0u32.to_le_bytes();
}

mod tinfoil_command_ids {
    pub const EXIT: [u8; 4] = 0u32.to_le_bytes();
    pub const FILE_RANGE: [u8; 4] = 1u32.to_le_bytes();
}

const USB_TIMEOUT: Duration = Duration::from_millis(500);
const HOST_HTTP_PORT: u16 = 8080;

fn read_game_paths(game_backup_path: &Path) -> color_eyre::Result<Vec<String>> {
    if !game_backup_path.exists() {
        bail!("Given path ({}) does not exist", game_backup_path.display())
    }

    let game_paths: Vec<_> = if game_backup_path.is_dir() {
        game_backup_path
            .read_dir()?
            .filter_map(|entry_result| {
                let entry = entry_result.ok()?;
                let path = entry.path();
                is_game_backup(&path).then_some(path.to_str()?.to_string())
            })
            .collect()
    } else if is_game_backup(game_backup_path)
        && let Some(path_str) = game_backup_path.to_str()
    {
        vec![path_str.to_string()]
    } else {
        bail!(
            "Given path ({}) is not a directory or a valid game backup file",
            game_backup_path.display()
        )
    };

    if game_paths.is_empty() {
        bail!(
            "No game backup files found in given directory ({})",
            game_backup_path.display()
        )
    }

    Ok(game_paths)
}

fn urlencode(input: &str) -> String {
    const FRAGMENT_PERCENT_ENCODE_SET: &AsciiSet =
        &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
    utf8_percent_encode(input, FRAGMENT_PERCENT_ENCODE_SET).to_string()
}

fn serve_http(
    game_paths: &[String],
    host_ip: IpAddr,
    run_http_server: Arc<AtomicBool>,
) -> color_eyre::Result<()> {
    let listener = TcpListener::bind((host_ip, HOST_HTTP_PORT))
        .wrap_err_with(|| {
            format!(
                "Unable to bind HTTP server to host IP and port ({}:{}).",
                host_ip, HOST_HTTP_PORT
            )
        })
        .suggestion(
            "Ensure no other process is using the same port, and that the host IP seems correct.",
        )?;
    listener.set_nonblocking(true)?;
    info!("sucesfulyl started http server");

    while run_http_server.load(Ordering::Relaxed) {
        let (mut stream, _addr) = match listener.accept() {
            Ok(tuple) => tuple,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                error!("error accepting incoming HTTP connection: {:?}", e);
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
        println!("Requested '{}'", requested_game_path);
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

        let game_size = std::fs::metadata(requested_game_path.as_ref())
            .unwrap()
            .size();

        match method {
            "GET" => {
                debug!("got GET");
                let range_line = lines.find(|line| line.starts_with("Range: ")).unwrap();
                const RANGE_PREFIX: usize = "Range: bytes=".len();
                let mut range_parts = range_line[RANGE_PREFIX..].split('-');
                let range_start: u64 = range_parts.next().unwrap().parse().unwrap();
                let range_end: u64 = range_parts.next().unwrap().parse().unwrap();
                let range_length = range_end - range_start + 1;

                let mut file = File::open(requested_game_path.as_ref()).unwrap();
                file.seek(SeekFrom::Start(range_start)).unwrap();
                let mut buf = vec![0u8; range_length as usize];
                file.read_exact(&mut buf).unwrap();

                respond_to_request(
                    &mut stream,
                    [
                    format!(
                        "HTTP/1.1 206 Partial Content\r\nAccept-Ranges: bytes\r\nContent-Range: bytes: {range_start}-{range_end}/{game_size}\r\nContent-Length: {range_length}\r\n\r\n",
                    ).as_bytes(),
                    &buf,
                ].concat()
                );
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

// TODO: maybe keep persistent BufReader of game file contents
pub fn perform_tinfoil_network_install(
    game_backup_path: &Path,
    target_ip: Ipv4Addr,
) -> color_eyre::Result<()> {
    let game_paths = read_game_paths(game_backup_path)?;
    println!("Performing network install to {}", target_ip);

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
        acc + &base_url + &urlencode(path) + "\n"
    });

    let run_http_server = Arc::new(AtomicBool::new(true));
    let run_http_server_thread = Arc::clone(&run_http_server);
    let http_thread =
        thread::spawn(move || serve_http(&game_paths, host_ip, run_http_server_thread));
    debug!("Spawned HTTP thread");

    keepalive_stream.write_all(
        &[
            &(u32::try_from(urls_with_newlines.len()).unwrap()).to_be_bytes(),
            urls_with_newlines.as_bytes(),
        ]
        .concat(),
    )?;

    debug!("Sent initiating data {}", &urls_with_newlines);

    let mut keepalive_buf = Vec::new();
    loop {
        info!("polling keepalive");
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
    http_thread.join().unwrap()?;

    Ok(())
}

fn deny_request(stream: &mut TcpStream) {
    respond_to_request(stream, "HTTP/1.1 400 Bad Request\r\n\r\n");
}

fn respond_to_request(stream: &mut TcpStream, buf: impl AsRef<[u8]>) {
    if let Err(e) = stream.write_all(buf.as_ref()) {
        error!("responding to HTTP request: {:?}", e);
    }
}

pub fn perform_tinfoil_usb_install(game_backup_path: &Path) -> color_eyre::Result<()> {
    let game_paths = read_game_paths(game_backup_path)?;
    let paths_with_newlines_string_length: usize =
        game_paths.iter().map(|path| path.len() + 1).sum();

    let device_info = list_devices()
        .wait()?
        .find(|dev| dev.vendor_id() == 0x57e && dev.product_id() == 0x3000)
        .wrap_err("Unable to discover Nintendo Switch through USB.")
        .suggestion(
            "Ensure the Nintendo Switch is awake and connected via cable to this computer.",
        )?;

    info!(
        "Nintendo Switch discovered at bus {} and address {}",
        device_info.bus_id(),
        device_info.device_address()
    );

    let device = device_info.open().wait()?;
    let interface = device.claim_interface(0).wait()?;
    let mut ep_out = interface.endpoint::<Bulk, Out>(0x01)?;
    ep_out.clear_halt().wait()?;
    let mut ep_in = interface.endpoint::<Bulk, In>(0x81)?;
    ep_in.clear_halt().wait()?;

    debug!("sending game backup list");
    write_usb(&mut ep_out, "TUL0")?;
    write_usb(
        &mut ep_out,
        &paths_with_newlines_string_length.to_le_bytes()[..4],
    )?; // FIXME: ugly slicing
    write_usb(&mut ep_out, [0u8; 8])?;
    for path in &game_paths {
        write_usb(&mut ep_out, [path.as_str(), "\n"].concat())?;
    }

    let mut pb = ProgressBar::no_length().with_style(
        ProgressStyle::with_template("ETA: {eta} ({binary_bytes_per_sec}) {wide_bar} {binary_bytes} of {binary_total_bytes} sent").unwrap(),
    );

    loop {
        debug!("waiting for header...");
        let command_header = ep_in
            .transfer_blocking(Buffer::new(512), Duration::MAX)
            .into_result()?;
        debug!("got header: {:#?}", &command_header);

        if &command_header[..4] != b"TUC0" {
            error!("invalid command header magic. continuing to next iteration...");
            continue;
        }
        debug!("correct command header magic");

        let command_type: [u8; 1] = command_header[4..5].try_into().unwrap();
        let command_id: [u8; 4] = command_header[8..12].try_into().unwrap();

        debug!(
            "Command type: {:?}, Command id: {:?}",
            &command_type, &command_id
        );

        match command_id {
            tinfoil_command_ids::EXIT => {
                debug!("got exit command, exiting...");
                pb.finish();
                break;
            }
            tinfoil_command_ids::FILE_RANGE => {
                debug!("got file range command");
                file_range_command(&mut ep_in, &mut ep_out, &mut pb, &game_paths)?
            }
            _ => bail!("invalid command ID encountered!"),
        }
    }

    Ok(())
}

fn is_game_backup(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext == "nsp" || ext == "xci" || ext == "nsz")
}

fn write_usb(
    ep_out: &mut Endpoint<Bulk, Out>,
    message: impl Into<Vec<u8>>,
) -> color_eyre::Result<()> {
    let buf = message.into();
    ep_out
        .transfer_blocking(buf.into(), USB_TIMEOUT)
        .status
        .map_err(|e| match e {
            TransferError::Cancelled => {
                eyre!("Nintendo Switch was discovered, but it is not accepting transfers.")
                    .suggestion(
                        "Ensure Awoo Installer is open, and in the menu 'Install Over USB'.",
                    )
            }
            TransferError::Disconnected => eyre!("USB has disconnected"),
            TransferError::Fault | TransferError::Stall | TransferError::InvalidArgument => {
                eyre!("Malformed data during transfer. {:?}", e)
            }
            TransferError::Unknown(i) => eyre!("Unknown error {}", i),
        })
}

fn read_usb(ep_in: &mut Endpoint<Bulk, In>) -> Result<Buffer, TransferError> {
    // TODO: avoid creating buffer everytime?
    // TODO: figure out if 512 is universal buffer size or just my machine?
    let buf = Buffer::new(512);
    ep_in.transfer_blocking(buf, USB_TIMEOUT).into_result()
}

fn file_range_command(
    ep_in: &mut Endpoint<Bulk, In>,
    ep_out: &mut Endpoint<Bulk, Out>,
    pb: &mut ProgressBar,
    game_paths: &[String],
) -> color_eyre::Result<()> {
    let file_range_header = read_usb(ep_in)?;

    let range_size = usize::from_le_bytes(file_range_header[..8].try_into().unwrap());
    let range_offset = u64::from_le_bytes(file_range_header[8..16].try_into().unwrap());
    let game_path_len = usize::from_le_bytes(file_range_header[16..24].try_into().unwrap());

    let game_name_buf = read_usb(ep_in)?;
    let game_path = str::from_utf8(&game_name_buf)?;

    if !game_paths.iter().any(|path| game_path == path) {
        bail!(
            "Nintendo Switch tried to request game backup ({}) not present on host",
            game_path
        );
    };

    info!("sending {}", &game_path);

    info!(
        "Range size: {}, Range offset: {}, Name len: {}, Name: {}",
        range_size, range_offset, game_path_len, game_path,
    );

    send_response_header(ep_out, range_size)?;

    let file = File::open(game_path)?;

    if let Ok(metadata) = file.metadata() {
        pb.set_length(metadata.size());
    }

    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(range_offset))?;

    let mut current_offset = 0;
    let end_offset = range_size;
    let mut read_size = 0x100000;

    let mut buf = vec![0u8; read_size];

    while current_offset < end_offset {
        if current_offset + read_size >= end_offset {
            debug!("too big read_size ({}), resizing...", read_size);
            read_size = end_offset - current_offset;
            buf.resize(read_size, 0u8);
        }
        reader.read_exact(&mut buf)?;

        ep_out.transfer_blocking(buf.clone().into(), Duration::MAX);

        debug!("sent {} bytes", read_size);

        current_offset += read_size;
        pb.set_position(current_offset as u64);
    }

    Ok(())
}

fn send_response_header(
    ep_out: &mut Endpoint<Bulk, Out>,
    range_size: usize,
) -> color_eyre::Result<()> {
    write_usb(ep_out, b"TUC0")?;
    write_usb(ep_out, tinfoil_command_types::RESPONSE)?;
    write_usb(ep_out, tinfoil_command_ids::FILE_RANGE)?;
    write_usb(ep_out, range_size.to_le_bytes())?;
    write_usb(ep_out, [0u8; 0xC])?; // padding?
    Ok(())
}
