use ssh2::{Channel, Session};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TunnelConfig {
    pub ssh_host: String,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub ssh_password: Option<String>,
    pub ssh_key_path: Option<String>,
    pub remote_host: String,
    pub remote_port: u16,
}

pub struct SshTunnel {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    local_port: u16,
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        self.stop();
    }
}

impl SshTunnel {
    pub fn start(config: TunnelConfig) -> Result<(Self, u16), String> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Bind to port 0 to get a random available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind local port: {}", e))?;

        let bound_port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local addr: {}", e))?
            .port();

        // Set non-blocking so we can check the running flag
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

        let handle = thread::spawn(move || {
            // Establish SSH Session ONCE
            match connect_ssh(&config) {
                Ok(session) => {
                    run_multiplexer(
                        listener,
                        session,
                        &config.remote_host,
                        config.remote_port,
                        running_clone,
                    );
                }
                Err(e) => {
                    eprintln!("SSH Connection Failed: {}", e);
                }
            }
        });

        Ok((
            Self {
                running,
                handle: Some(handle),
                local_port: bound_port,
            },
            bound_port,
        ))
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn get_local_port(&self) -> u16 {
        self.local_port
    }
}

use ssh2_config::SshConfig;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn connect_ssh(config: &TunnelConfig) -> Result<Session, String> {
    // 1. Load and Parse ~/.ssh/config
    let mut ssh_config = SshConfig::default();
    if let Some(home_dir) = dirs::home_dir() {
        let config_path = home_dir.join(".ssh").join("config");
        if config_path.exists() {
            if let Ok(file) = File::open(&config_path) {
                let mut reader = BufReader::new(file);
                if let Ok(parsed) = SshConfig::default()
                    .parse(&mut reader, ssh2_config::ParseRule::ALLOW_UNKNOWN_FIELDS)
                {
                    ssh_config = parsed;
                }
            }
        }
    }

    // 2. Query Config
    let host_params = ssh_config.query(&config.ssh_host);

    // 3. Resolve Parameters
    let final_host = host_params.host_name.unwrap_or(config.ssh_host.clone());
    let final_port = config.ssh_port.or(host_params.port).unwrap_or(22);
    let final_user = config
        .ssh_user
        .clone()
        .or(host_params.user)
        .unwrap_or("root".to_string());

    // Resolve Identity File
    let final_key_path = if let Some(ui_path) = &config.ssh_key_path {
        let p = ui_path.to_string();
        if p.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                if p == "~" {
                    Some(home.to_string_lossy().to_string())
                } else if p.starts_with("~/") {
                    Some(home.join(&p[2..]).to_string_lossy().to_string())
                } else {
                    Some(p)
                }
            } else {
                Some(p)
            }
        } else {
            Some(p)
        }
    } else if let Some(config_paths) = host_params.identity_file {
        config_paths.first().and_then(|path| {
            let p = path.to_string_lossy().to_string();
            if p.starts_with("~") {
                if let Some(home) = dirs::home_dir() {
                    if p == "~" {
                        Some(home.to_string_lossy().to_string())
                    } else if p.starts_with("~/") {
                        Some(home.join(&p[2..]).to_string_lossy().to_string())
                    } else {
                        Some(p)
                    }
                } else {
                    Some(p)
                }
            } else if path.is_relative() {
                dirs::home_dir().map(|h| h.join(".ssh").join(path).to_string_lossy().to_string())
            } else {
                Some(p)
            }
        })
    } else {
        None
    };

    eprintln!(
        "SSH Tunnel: Connecting to {}:{} as '{}'",
        final_host, final_port, final_user
    );
    if let Some(ref kp) = final_key_path {
        eprintln!("SSH Tunnel: Using identity file: '{}'", kp);
    }

    let tcp = TcpStream::connect(format!("{}:{}", final_host, final_port))
        .map_err(|e| format!("Failed to connect to SSH server: {}", e))?;

    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);

    // 4. Apply Algorithms
    {
        let kex = host_params.kex_algorithms.algorithms();
        if !kex.is_empty() {
            let _ = sess.method_pref(ssh2::MethodType::Kex, &kex.join(","));
        }
        let algos = host_params.host_key_algorithms.algorithms();
        if !algos.is_empty() {
            let _ = sess.method_pref(ssh2::MethodType::HostKey, &algos.join(","));
        }
        let ciphers = host_params.ciphers.algorithms();
        if !ciphers.is_empty() {
            let _ = sess.method_pref(ssh2::MethodType::CryptCs, &ciphers.join(","));
            let _ = sess.method_pref(ssh2::MethodType::CryptSc, &ciphers.join(","));
        }
        let macs = host_params.mac.algorithms();
        if !macs.is_empty() {
            let _ = sess.method_pref(ssh2::MethodType::MacCs, &macs.join(","));
            let _ = sess.method_pref(ssh2::MethodType::MacSc, &macs.join(","));
        }
    }

    eprintln!("SSH Tunnel: Handshaking...");
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;

    eprintln!("SSH Tunnel: Authenticating...");
    if let Some(password) = &config.ssh_password {
        sess.userauth_password(&final_user, password)
            .map_err(|e| format!("SSH password auth failed: {}", e))?;
    } else if let Some(key_path) = final_key_path {
        sess.userauth_pubkey_file(&final_user, None, Path::new(&key_path), None)
            .map_err(|e| format!("SSH key auth failed: {}", e))?;
    } else {
        sess.userauth_agent(&final_user)
            .map_err(|e| format!("SSH agent auth failed: {}", e))?;
    }

    if !sess.authenticated() {
        return Err("SSH authentication failed".to_string());
    }
    eprintln!("SSH Tunnel: Authenticated.");

    Ok(sess)
}

struct ActiveChannel {
    stream: TcpStream,
    channel: Channel,
    pending_write_to_remote: Vec<u8>,
    pending_write_to_local: Vec<u8>,
    local_eof: bool,
    remote_eof: bool,
}

fn run_multiplexer(
    listener: TcpListener,
    session: Session,
    remote_host: &str,
    remote_port: u16,
    running: Arc<AtomicBool>,
) {
    let mut channels: Vec<ActiveChannel> = Vec::new();
    let mut buf = [0u8; 16384];

    // Set session to non-blocking for the loop
    session.set_blocking(false);

    while running.load(Ordering::SeqCst) {
        let mut did_work = false;

        // 1. Accept new connections
        match listener.accept() {
            Ok((stream, _)) => {
                // Temporarily set blocking to open channel (simplifies logic)
                session.set_blocking(true);
                eprintln!(
                    "SSH Tunnel: Opening direct-tcpip channel to {}:{}",
                    remote_host, remote_port
                );
                match session.channel_direct_tcpip(remote_host, remote_port, None) {
                    Ok(channel) => {
                        eprintln!("SSH Tunnel: Channel opened.");
                        let _ = stream.set_nonblocking(true);
                        channels.push(ActiveChannel {
                            stream,
                            channel,
                            pending_write_to_remote: Vec::new(),
                            pending_write_to_local: Vec::new(),
                            local_eof: false,
                            remote_eof: false,
                        });
                        did_work = true;
                    }
                    Err(e) => eprintln!("Failed to open channel: {}", e),
                }
                session.set_blocking(false);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                eprintln!("Listener accept error: {}", e);
                break;
            }
        }

        // 2. Process Channels
        for active in channels.iter_mut() {
            // Read from Local -> Buffer
            if !active.local_eof && active.pending_write_to_remote.is_empty() {
                match active.stream.read(&mut buf) {
                    Ok(0) => {
                        active.local_eof = true;
                        let _ = active.channel.send_eof();
                    }
                    Ok(n) => {
                        active.pending_write_to_remote.extend_from_slice(&buf[..n]);
                        did_work = true;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        // Explicitly handle error to help type inference if needed, though read returns Result<usize, io::Error>
                        eprintln!("Local read error: {}", e);
                        active.local_eof = true;
                    }
                }
            }

            // Write Buffer -> Remote
            if !active.pending_write_to_remote.is_empty() {
                match active.channel.write(&active.pending_write_to_remote) {
                    Ok(n) => {
                        active.pending_write_to_remote.drain(0..n);
                        did_work = true;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => active.remote_eof = true, // Treat write error as EOF/Close
                }
            }

            // Read from Remote -> Buffer
            if !active.remote_eof && active.pending_write_to_local.is_empty() {
                match active.channel.read(&mut buf) {
                    Ok(0) => {
                        active.remote_eof = true;
                    }
                    Ok(n) => {
                        active.pending_write_to_local.extend_from_slice(&buf[..n]);
                        did_work = true;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => active.remote_eof = true,
                }
            }

            // Write Buffer -> Local
            if !active.pending_write_to_local.is_empty() {
                match active.stream.write(&active.pending_write_to_local) {
                    Ok(n) => {
                        active.pending_write_to_local.drain(0..n);
                        did_work = true;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => active.local_eof = true,
                }
            }
        }

        // 3. Cleanup Closed Channels
        channels.retain(|c| {
            let closed = (c.local_eof && c.pending_write_to_remote.is_empty())
                && (c.remote_eof && c.pending_write_to_local.is_empty());
            !closed
        });

        if !did_work {
            thread::sleep(Duration::from_millis(5));
        }
    }
}
