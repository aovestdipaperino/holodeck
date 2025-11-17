use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper_util::rt::TokioIo;
use indicatif::{ProgressBar, ProgressStyle};
use reverse_ssh::{ReverseSshClient, ReverseSshConfig};
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

const SHARED_DIR: &str = ".";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing only if RUST_LOG is set
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();
    }

    // Create shared directory if it doesn't exist
    fs::create_dir_all(SHARED_DIR).await?;

    // Bind to a random available port
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?;
    let local_port = local_addr.port();

    // Get absolute path of shared directory
    let shared_path =
        std::fs::canonicalize(SHARED_DIR).unwrap_or_else(|_| PathBuf::from(SHARED_DIR));

    println!("HTTP File Server running on http://{}", local_addr);
    println!("Shared directory: {}", shared_path.display());

    // Spawn reverse SSH tunnel if configuration is provided
    if let Some(external_url) = setup_reverse_tunnel(local_port).await {
        // Wait a moment for the tunnel to be fully established
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        println!("\n=== Reverse SSH Tunnel Active ===");
        println!("Your server is now accessible externally!");

        // Print usage with external URL
        println!("\nUsage:");
        println!("  GET file:  curl {}/<filename>", external_url);
        println!("  POST file: curl -X POST --data-binary @<file> {}/<filename>", external_url);
        println!("  List files: curl {}/", external_url);
    } else {
        // Print usage with local URL
        println!("\nUsage:");
        println!(
            "  GET file:  curl http://localhost:{}/<filename>",
            local_port
        );
        println!(
            "  POST file: curl -X POST --data-binary @<file> http://localhost:{}/<filename>",
            local_port
        );
        println!("  List files: curl http://localhost:{}/", local_port);

        println!("\n=== Running in Local Mode ===");
        println!("To enable external access, set these environment variables:");
        println!("  SSH_SERVER   - SSH server address (e.g., ssh.localhost.run)");
        println!("  SSH_USER     - SSH username (optional, defaults to 'localhost')");
        println!("  SSH_PORT     - SSH server port (optional, defaults to 22)");
        println!("  SSH_KEY_PATH - Path to SSH private key (required for key auth)");
        println!("  SSH_PASSWORD - SSH password (alternative to key auth)");
        println!("  REMOTE_PORT  - Remote port to listen on (optional, defaults to 80)");
        println!("\nExample with localhost.run:");
        println!("  SSH_SERVER=ssh.localhost.run SSH_KEY_PATH=~/.ssh/id_ed25519 cargo run");
    }

    // Run HTTP server
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn setup_reverse_tunnel(local_port: u16) -> Option<String> {
    // Check if SSH server is configured
    let server_addr = env::var("SSH_SERVER").ok()?;

    // Get SSH key path from environment variable only
    let key_path = env::var("SSH_KEY_PATH").ok();

    let config = ReverseSshConfig {
        server_addr: server_addr.clone(),
        server_port: env::var("SSH_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(22),
        username: env::var("SSH_USER").unwrap_or_else(|_| "localhost".to_string()),
        key_path: key_path.clone(),
        password: env::var("SSH_PASSWORD").ok(),
        remote_port: env::var("REMOTE_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(80),
        local_addr: "127.0.0.1".to_string(),
        local_port,
    };

    println!(
        "\nConnecting to SSH server: {}:{}",
        config.server_addr, config.server_port
    );
    if let Some(ref key) = key_path {
        println!("Using SSH key: {}", key);
    } else {
        println!("Using password authentication");
    }
    println!(
        "Forwarding remote port {} to local port {}",
        config.remote_port, local_port
    );

    // Create a channel to receive the URL from the spawned task
    let (url_tx, mut url_rx) = tokio::sync::mpsc::channel::<String>(1);

    tokio::spawn(async move {
        let mut client = ReverseSshClient::new(config);
        let mut url_sent = false;
        match client
            .run_with_message_handler(move |message| {
                // Extract and display the tunnel URL prominently
                for line in message.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Check if this line contains the tunnel URL
                        if (trimmed.contains("http://") || trimmed.contains("https://"))
                           && (trimmed.contains(".lhr.life") || trimmed.contains(".lhr.rocks") || trimmed.contains(".localhost.run"))
                        {
                            // Extract the URL
                            if let Some(url_start) = trimmed.find("http") {
                                let url_part = &trimmed[url_start..];
                                // Find the end of the URL
                                let url_end = url_part.find(|c: char| c.is_whitespace() || c == ',' || c == ';')
                                    .unwrap_or(url_part.len());
                                let url = &url_part[..url_end];

                                if !url_sent {
                                    println!("\n╔════════════════════════════════════════════════════════════════╗");
                                    println!("║                    TUNNEL ACTIVE                               ║");
                                    println!("╠════════════════════════════════════════════════════════════════╣");
                                    println!("║  External URL: {:<48}║", url);
                                    println!("╚════════════════════════════════════════════════════════════════╝\n");
                                    let _ = url_tx.try_send(url.to_string());
                                    url_sent = true;
                                }
                            }
                        }
                    }
                }
            })
            .await
        {
            Ok(_) => println!("Reverse SSH tunnel closed"),
            Err(e) => eprintln!("Reverse SSH tunnel error: {}", e),
        }
    });

    // Wait for the URL with a timeout
    tokio::select! {
        result = url_rx.recv() => result,
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
            eprintln!("Warning: Timed out waiting for tunnel URL");
            None
        }
    }
}

async fn handle_request(req: Request<Incoming>) -> Result<Response<BoxBody>, hyper::Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    match (method, path.as_str()) {
        (Method::GET, "/") => list_files().await,
        (Method::GET, path) => get_file(path).await,
        (Method::POST, path) => post_file(req, path).await,
        _ => Ok(not_found()),
    }
}

async fn list_files() -> Result<Response<BoxBody>, hyper::Error> {
    match fs::read_dir(SHARED_DIR).await {
        Ok(mut entries) => {
            let mut files = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_name) = entry.file_name().into_string() {
                    files.push(file_name);
                }
            }

            let body = if files.is_empty() {
                "No files available\n".to_string()
            } else {
                format!("Available files:\n{}\n", files.join("\n"))
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full(body))
                .unwrap())
        }
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full(format!("Error listing files: {}", e)))
                .unwrap())
        }
    }
}

async fn get_file(path: &str) -> Result<Response<BoxBody>, hyper::Error> {
    let filename = path.trim_start_matches('/');

    if filename.is_empty() {
        return list_files().await;
    }

    // Prevent directory traversal attacks
    if filename.contains("..") || filename.contains('/') {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(full("Invalid filename"))
            .unwrap());
    }

    let file_path = PathBuf::from(SHARED_DIR).join(filename);

    // Create progress spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    spinner.set_message(format!("Sending file '{}'", filename));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    match fs::read(&file_path).await {
        Ok(contents) => {
            let size = contents.len();
            spinner.finish_with_message(format!("GET: Served file '{}' ({} bytes)", filename, size));
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(full(contents))
                .unwrap())
        }
        Err(_) => {
            spinner.finish_and_clear();
            eprintln!("GET: File '{}' not found", filename);
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(format!("File '{}' not found", filename)))
                .unwrap())
        }
    }
}

async fn post_file(req: Request<Incoming>, path: &str) -> Result<Response<BoxBody>, hyper::Error> {
    let filename = path.trim_start_matches('/');

    if filename.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(full("Filename required in path"))
            .unwrap());
    }

    // Prevent directory traversal attacks
    if filename.contains("..") || filename.contains('/') {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(full("Invalid filename"))
            .unwrap());
    }

    let file_path = PathBuf::from(SHARED_DIR).join(filename);

    // Create progress spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message(format!("Receiving file '{}'", filename));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Collect the request body
    let body = req.collect().await?.to_bytes();

    match fs::File::create(&file_path).await {
        Ok(mut file) => match file.write_all(&body).await {
            Ok(_) => {
                spinner.finish_with_message(format!("POST: Received file '{}' ({} bytes)", filename, body.len()));
                Ok(Response::builder()
                    .status(StatusCode::CREATED)
                    .body(full(format!(
                        "File '{}' uploaded successfully ({} bytes)",
                        filename,
                        body.len()
                    )))
                    .unwrap())
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!("POST: Error writing file '{}': {}", filename, e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(full(format!("Error writing file: {}", e)))
                    .unwrap())
            }
        },
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("POST: Error creating file '{}': {}", filename, e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full(format!("Error creating file: {}", e)))
                .unwrap())
        }
    }
}

fn not_found() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(full("Not found"))
        .unwrap()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
