use hyper::{
    client::{Client, HttpConnector},
    Body as HyperBody, Request as HyperRequest,
};
use lambda_http::{service_fn, Error, Request, RequestExt, Response};
use serde::Deserialize;
use std::sync::Arc;
use std::{ffi::OsStr, path::Path, process::Stdio, time::Duration};
use std::{
    process::{Child, Command},
    time::Instant,
};
use std::{thread, time};
use tracing::{debug, error, info};
use url::Url;

mod logger;
mod pulse;

/*
    _HANDLER – The location to the handler, from the function configuration. The standard format is file.method, where file is the name of the file without an extension, and method is the name of a method or function that’s defined in the file.
    LAMBDA_TASK_ROOT – The directory that contains the function code.
    AWS_LAMBDA_RUNTIME_API – The host and port of the runtime API.
    USE_STREAM - optional, if set to true, the adapter will use streaming response instead of buffered response.
*/

static SERVER_COMMAND_TIMEOUT_MILLIS: u64 = 100; // 100ms wait to see if command has failed entirely.
static LAMBDA_INIT_TIMEOUT_MILLIS: u64 = 8500; // 8.5sec for starting the server.
static HEALTHCHECK_HTTP_TIMEOUT_MILLIS: u64 = 50; // How long to wait for the server to respond.

static HEALTH_CHECK_LOOPS: u64 = LAMBDA_INIT_TIMEOUT_MILLIS / HEALTHCHECK_HTTP_TIMEOUT_MILLIS;

static HEALTHCHECK_HTTP_TIMEOUT: Duration = Duration::from_millis(HEALTHCHECK_HTTP_TIMEOUT_MILLIS);
static SERVER_COMMAND_TIMEOUT: Duration = time::Duration::from_millis(SERVER_COMMAND_TIMEOUT_MILLIS);

fn default_use_stream() -> bool {
    false
}

#[derive(Deserialize, Debug)]
struct Configuration {
    _handler: String,
    server_url: String,
    #[serde(default = "default_use_stream")]
    use_stream: bool,
}

fn create_symlinks() {
    // @TODO: Implement later
    // create_dir("/tmp/cache");
    // symlink("/tmp/cache", "/var/task/.next/cache");
}

fn get_executable_from_filepath(filename: &str) -> Option<&str> {
    let ext = Path::new(filename).extension().and_then(OsStr::to_str);

    // Handle extension not existing.

    match ext {
        Some("js") => Some("node"),
        Some("py") => Some("python"),
        // @TODO: Add support for other languages.
        _ => panic!("Unsupported extension: {}", ext.unwrap_or("none")),
    }
}

async fn starter(filepath: &str) -> Child {
    // Run `node server.js` to start the server.
    // Wait for the server to be ready.

    let executable = get_executable_from_filepath(filepath).unwrap();

    let mut child: Child = Command::new(executable)
        // Applicable to NextJS, parametrize.
        .arg(filepath)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to execute process");

    thread::sleep(SERVER_COMMAND_TIMEOUT);

    match child.try_wait() {
        Ok(None) => {
            info!("[Adapter] Server is starting ...");
        }
        Ok(Some(status)) => {
            let _ = child.kill();
            panic!("[Adapter] Server exited with: {}", status)
        }
        Err(e) => {
            let _ = child.kill();
            panic!("[Adapter] Error attempting to wait for server: {}", e)
        }
    }

    child
}

// Waits for server to bootup. It periodically pings the server until it responds.
// Fails if server does not respond in time.
async fn checker(health_check_url: &str) {
    let domain = Url::parse(health_check_url).map(String::from).unwrap();
    info!("[Adapter] Health check url: {}", domain);

    let mut is_ready = false;

    for _ in 0..HEALTH_CHECK_LOOPS {
        let result = pulse::pulse(domain.clone(), HEALTHCHECK_HTTP_TIMEOUT).await;

        match result {
            Ok(_) => {
                info!("[Adapter] Server is ready!");
                is_ready = true;
                break;
            }
            Err(err) => match err {
                pulse::HealthCheckError::Timeout => {
                    debug!("[Adapter] Timeout");
                }
                pulse::HealthCheckError::NetworkError => {
                    debug!("[Adapter] Network error");
                    // Server responded immediately with unreachable.
                    thread::sleep(HEALTHCHECK_HTTP_TIMEOUT);
                }
            },
        }
    }

    if !is_ready {
        panic!("[Adapter] Server is not ready. Exiting...");
    }
}

async fn translator(server_url: &str, event: Request) -> Result<Response<HyperBody>, Error> {
    let mut domain = Url::parse(server_url).unwrap();
    domain.set_path("/");
    info!("[Adapter] App url: {}", domain);

    let request_context = event.request_context();
    let lambda_context = &event.lambda_context();
    let path = event.raw_http_path().to_string();
    let (parts, body) = event.into_parts();

    let raw_client: Client<HttpConnector> = Client::builder().pool_idle_timeout(Duration::from_secs(4)).build(HttpConnector::new());

    let client = Arc::new(raw_client);

    let mut app_url = domain.clone();
    app_url.set_path(path.as_str());
    app_url.set_query(parts.uri.query());

    let mut builder = HyperRequest::builder()
        .method(parts.method)
        .uri(app_url.to_string())
        // include request context in http header "x-amzn-request-context"
        .header("x-amzn-request-context", serde_json::to_string(&request_context)?.as_bytes())
        // include lambda context in http header "x-amzn-lambda-context"
        .header("x-amzn-lambda-context", serde_json::to_string(&lambda_context)?.as_bytes());

    if let Some(headers) = builder.headers_mut() {
        headers.extend(parts.headers);
    }

    let request = builder.body(HyperBody::from(body.to_vec()))?;
    let app_response = client.request(request).await?;

    Ok(Response::from(app_response))
}

// Rust uses separate thread. Main thread to be used by server.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let start_time = Instant::now();
    debug!("Starting up...");

    logger::init_logger();

    let config = &envy::from_env::<Configuration>().expect("Please provide necessary env vars.");

    info!("[Adapter] Symlinking cache...");
    debug!("Elapsed time: {:.2?}", start_time.elapsed());
    create_symlinks();

    info!("[Adapter] Starting server...");
    debug!("Elapsed time: {:.2?}", start_time.elapsed());
    starter(&config._handler).await;

    // @TODO: Graceful shutdown server on SIGTERM.
    // match signal::ctrl_c().await {
    //     Ok(()) => {},
    //     Err(err) => {
    //         warn!("Unable to listen for shutdown signal: {}", err);
    //         // we also shut down in case of error
    //     },
    // }

    info!("[Adapter] Waiting for server to start...");
    debug!("Elapsed time: {:.2?}", start_time.elapsed());
    checker(&config.server_url).await;

    info!("[Adapter] Starting handler...");
    debug!("Elapsed time: {:.2?}", start_time.elapsed());

    // @TODO: Allow for streaming response.
    let result = match &config.use_stream {
        true => lambda_http::run_with_streaming_response(service_fn(|event: Request| translator(&config.server_url, event))).await,
        false => lambda_http::run(service_fn(|event: Request| translator(&config.server_url, event))).await,
    };

    debug!("Elapsed time: {:.2?}", start_time.elapsed());

    match result {
        Ok(_) => info!("[Adapter] Success"),
        Err(err) => error!("[Adapter] Error: {}", err),
    }

    Ok(())
}
