use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ssh2::Session;

pub struct SshTunnel {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl SshTunnel {
    pub fn start(
        ssh_host: String,
        ssh_port: Option<u16>,
        ssh_user: Option<String>,
        ssh_password: Option<String>,
        ssh_key_path: Option<String>,
        remote_host: String,
        remote_port: u16,
        local_port: u16,
    ) -> Result<(Self, u16), String> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port))
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
            while running_clone.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((local_stream, _)) => {
                        let ssh_host = ssh_host.clone();
                        let ssh_user = ssh_user.clone();
                        let ssh_password = ssh_password.clone();
                        let ssh_key_path = ssh_key_path.clone();
                        let remote_host = remote_host.clone();

                        thread::spawn(move || {
                            if let Err(e) = handle_connection(
                                local_stream,
                                &ssh_host,
                                ssh_port,
                                ssh_user.as_deref(),
                                ssh_password.as_deref(),
                                ssh_key_path.as_deref(),
                                &remote_host,
                                remote_port,
                            ) {
                                eprintln!("SSH Tunnel Error: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Listener accept error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok((
            Self {
                running,
                handle: Some(handle),
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
}

use ssh2_config::SshConfig;
use std::fs::File;
use std::io::BufReader;

fn handle_connection(
    local_stream: TcpStream,
    ssh_host: &str,
    ssh_port: Option<u16>,
    ssh_user: Option<&str>,
    ssh_password: Option<&str>,
    ssh_key_path: Option<&str>,
    remote_host: &str,
    remote_port: u16,
) -> Result<(), String> {
    // 1. Load and Parse ~/.ssh/config
    let mut config = SshConfig::default();
    if let Some(home_dir) = dirs::home_dir() {
        let config_path = home_dir.join(".ssh").join("config");
        if config_path.exists() {
            if let Ok(file) = File::open(&config_path) {
                let mut reader = BufReader::new(file);
                if let Ok(parsed) = SshConfig::default()
                    .parse(&mut reader, ssh2_config::ParseRule::ALLOW_UNKNOWN_FIELDS)
                {
                    config = parsed;
                }
            }
        }
    }

    // 2. Query Config
    let host_params = config.query(ssh_host);

    // 3. Resolve Parameters
    let final_host = host_params.host_name.unwrap_or(ssh_host.to_string());
    let final_port = ssh_port.or(host_params.port).unwrap_or(22);
    let final_user = ssh_user
        .map(|s| s.to_string())
        .or(host_params.user)
        .unwrap_or("root".to_string());

    // Resolve Identity File
    // 1. UI Path (takes precedence)
    let final_key_path = if let Some(ui_path) = ssh_key_path {
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
        // 2. Config Path
        // Take the first one. If relative, prepend ~/.ssh/
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
                // Relative to ~/.ssh/
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

    // 4. Apply Algorithms from Config
    {
        let kex = host_params.kex_algorithms.algorithms();
        if !kex.is_empty() {
            sess.method_pref(ssh2::MethodType::Kex, &kex.join(","))
                .map_err(|e| format!("Failed to set Kex: {}", e))?;
        }

        let algos = host_params.host_key_algorithms.algorithms();
        if !algos.is_empty() {
            sess.method_pref(ssh2::MethodType::HostKey, &algos.join(","))
                .map_err(|e| format!("Failed to set HostKey: {}", e))?;
        }

        let ciphers = host_params.ciphers.algorithms();
        if !ciphers.is_empty() {
            sess.method_pref(ssh2::MethodType::CryptCs, &ciphers.join(","))
                .map_err(|e| format!("Failed to set CryptCs: {}", e))?;
            sess.method_pref(ssh2::MethodType::CryptSc, &ciphers.join(","))
                .map_err(|e| format!("Failed to set CryptSc: {}", e))?;
        }

        let macs = host_params.mac.algorithms();
        if !macs.is_empty() {
            sess.method_pref(ssh2::MethodType::MacCs, &macs.join(","))
                .map_err(|e| format!("Failed to set MacCs: {}", e))?;
            sess.method_pref(ssh2::MethodType::MacSc, &macs.join(","))
                .map_err(|e| format!("Failed to set MacSc: {}", e))?;
        }
    }

    eprintln!("SSH Tunnel: Handshaking...");
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;

    eprintln!("SSH Tunnel: Authenticating...");
    if let Some(password) = ssh_password {
        sess.userauth_password(&final_user, password)
            .map_err(|e| format!("SSH password auth failed: {}", e))?;
    } else if let Some(key_path) = final_key_path {
        sess.userauth_pubkey_file(&final_user, None, std::path::Path::new(&key_path), None)
            .map_err(|e| format!("SSH key auth failed: {}", e))?;
    } else {
        // Try agent or none
        sess.userauth_agent(&final_user)
            .map_err(|e| format!("SSH agent auth failed: {}", e))?;
    }

    if !sess.authenticated() {
        return Err("SSH authentication failed".to_string());
    }
    eprintln!("SSH Tunnel: Authenticated.");

    eprintln!(
        "SSH Tunnel: Opening direct-tcpip channel to {}:{}",
        remote_host, remote_port
    );
    let mut channel = sess
        .channel_direct_tcpip(remote_host, remote_port, None)
        .map_err(|e| format!("Failed to open direct-tcpip channel: {}", e))?;
    eprintln!("SSH Tunnel: Channel opened.");

    // Bidirectional copy
    // Since channel implements Read/Write, we can just loop and copy.
    // However, channel.read/write are blocking.
    // We need to handle both directions.

    // Clone local stream for the other direction
    let mut local_reader = local_stream.try_clone().map_err(|e| e.to_string())?;
    let mut local_writer = local_stream;

    // We can't easily clone the Channel because it's tied to the Session which is not thread-safe (mostly).
    // But ssh2::Channel is Send.
    // Actually, ssh2::Channel is a handle.
    // The issue is that we need to read from local and write to channel, AND read from channel and write to local.
    // Standard approach: set non-blocking on both and select, or spawn threads if possible.
    // ssh2::Channel read/write are blocking by default.
    // Let's use non-blocking mode for the session?
    // Or just use simple read/write with timeouts?

    // Simpler approach for this demo:
    // Set streams to non-blocking and poll loop.

    local_reader
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;
    // Channel doesn't exactly support set_nonblocking directly easily without session config.
    // But we can use `sess.set_blocking(false)`.
    sess.set_blocking(false);

    let mut buf_local_to_remote = [0u8; 16384];
    let mut buf_remote_to_local = [0u8; 16384];

    // Buffers to hold data that has been read but not yet written
    let mut pending_write_to_remote: Vec<u8> = Vec::new();
    let mut pending_write_to_local: Vec<u8> = Vec::new();

    let mut local_eof = false;
    let mut remote_eof = false;

    loop {
        let mut did_work = false;

        // 1. Read from Local -> Buffer
        if !local_eof && pending_write_to_remote.is_empty() {
            match local_reader.read(&mut buf_local_to_remote) {
                Ok(0) => {
                    // EOF from local
                    eprintln!("SSH Tunnel: Local EOF");
                    local_eof = true;
                    let _ = channel.send_eof();
                }
                Ok(n) => {
                    // eprintln!("SSH Tunnel: Read {} bytes from local", n);
                    pending_write_to_remote.extend_from_slice(&buf_local_to_remote[..n]);
                    did_work = true;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(format!("Local read error: {}", e)),
            }
        }

        // 2. Write Buffer -> Remote
        if !pending_write_to_remote.is_empty() {
            match channel.write(&pending_write_to_remote) {
                Ok(n) => {
                    // eprintln!("SSH Tunnel: Wrote {} bytes to remote", n);
                    pending_write_to_remote.drain(0..n);
                    did_work = true;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(format!("Remote write error: {}", e)),
            }
        }

        // 3. Read from Remote -> Buffer
        if !remote_eof && pending_write_to_local.is_empty() {
            match channel.read(&mut buf_remote_to_local) {
                Ok(0) => {
                    // EOF from remote
                    eprintln!("SSH Tunnel: Remote EOF");
                    remote_eof = true;
                }
                Ok(n) => {
                    // eprintln!("SSH Tunnel: Read {} bytes from remote", n);
                    pending_write_to_local.extend_from_slice(&buf_remote_to_local[..n]);
                    did_work = true;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(format!("Remote read error: {}", e)),
            }
        }

        // 4. Write Buffer -> Local
        if !pending_write_to_local.is_empty() {
            match local_writer.write(&pending_write_to_local) {
                Ok(n) => {
                    // eprintln!("SSH Tunnel: Wrote {} bytes to local", n);
                    pending_write_to_local.drain(0..n);
                    did_work = true;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(format!("Local write error: {}", e)),
            }
        }

        if !did_work {
            thread::sleep(Duration::from_millis(1));
        }

        if (local_eof && pending_write_to_remote.is_empty())
            && (remote_eof && pending_write_to_local.is_empty())
        {
            eprintln!("SSH Tunnel: Closing connection");
            break;
        }
    }

    Ok(())
}
