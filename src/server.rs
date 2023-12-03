use http::Method;
use hyper::{
    client::{Client, HttpConnector},
    Body as HyperBody, Request as HyperRequest,
};
use lambda_http::{service_fn, Error, Request, RequestExt, Response};
use std::{process::Stdio, time::Duration};
use std::{
    process::{Child, Command},
    sync::Arc,
};
use std::{thread, time};
use url::Url;

/*
    _HANDLER – The location to the handler, from the function configuration. The standard format is file.method, where file is the name of the file without an extension, and method is the name of a method or function that’s defined in the file.
    LAMBDA_TASK_ROOT – The directory that contains the function code.
    AWS_LAMBDA_RUNTIME_API – The host and port of the runtime API.
*/

//  "buffered"
//  "response_stream"

async fn starter() -> Child {
    // Run `node server.js` to start the server.
    // Wait for the server to be ready.

    let mut child: Child = Command::new("node")
        .arg("/var/task/server.js")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to execute process");

    let ten_millis = time::Duration::from_millis(250);
    thread::sleep(ten_millis);

    // If process fails within 10ms something is wrong, abort without waiting for server.

    match child.try_wait() {
        Ok(None) => {
            println!("[Adapter] Server is starting ...");
        }
        Ok(Some(status)) => {
            println!("[Adapter] Server exited with: {status}");
            std::process::exit(1);
        }
        Err(e) => {
            println!("[Adapter] Error attempting to wait for server: {e}");
            std::process::exit(1);
        }
    }

    child
}

async fn checker() {
    // @TODO: Pass string in env vars.
    let schema = String::from("http");
    let host = String::from("localhost");
    let port = String::from("3000");
    let readiness_path = String::from("/api");

    let timeout = Duration::from_millis(10);
    let sleep_time = time::Duration::from_millis(50);

    let domain: String = format!("{}://{}:{}{}", schema, host, port, readiness_path)
        .parse()
        .unwrap();

    let mut is_ready = false;

    for _ in 0..180 {
        //
        // Try to connect to the server up to 10 times.
        // If we fail to get 200 OK, we throw error and exit.
        print!("[Adapter] Trying to connect to the server... ");

        let req = HyperRequest::builder()
            .method(Method::GET)
            .uri(domain.clone())
            .body(HyperBody::empty())
            .unwrap();

        let status_res = Client::new().request(req);

        match tokio::time::timeout(timeout, status_res).await {
            Ok(result) => match result {
                Ok(response) => {
                    println!("[Adapter] Status: {}", response.status());
                    is_ready = true;
                    break;
                }
                Err(e) => {
                    // We got response, but it's error.
                    println!("[Adapter] Network error: {:?}", e);
                }
            },
            Err(_) => {
                println!(
                    "[Adapter] Timeout: no response in {} milliseconds. Trying again...",
                    timeout.as_millis()
                );
            }
        };

        // Sleep, waiting for the server to start.
        thread::sleep(sleep_time);
    }

    if !is_ready {
        println!("[Adapter] Server is not ready. Exiting...");
        std::process::exit(1);
    }
}

async fn translator(event: Request) -> Result<Response<HyperBody>, Error> {
    let _schema = String::from("http");
    let _host = String::from("localhost");
    let _port = String::from("3000");
    let _readiness_path = String::from("/api");
    let _timeout = Duration::from_millis(10);

    let domain = Url::parse("http://localhost:3000").unwrap();

    let request_context = event.request_context();
    let lambda_context = &event.lambda_context();
    let path = event.raw_http_path().to_string();
    let (parts, body) = event.into_parts();

    let raw_client: Client<HttpConnector> = Client::builder()
        .pool_idle_timeout(Duration::from_secs(4))
        .build(HttpConnector::new());

    let client = Arc::new(raw_client);

    let mut app_url = domain.clone();
    app_url.set_path(path.as_str());
    app_url.set_query(parts.uri.query());

    let mut builder = HyperRequest::builder()
        .method(parts.method)
        .uri(app_url.to_string())
        // include request context in http header "x-amzn-request-context"
        .header(
            "x-amzn-request-context",
            serde_json::to_string(&request_context)?.as_bytes(),
        )
        // include lambda context in http header "x-amzn-lambda-context"
        .header(
            "x-amzn-lambda-context",
            serde_json::to_string(&lambda_context)?.as_bytes(),
        );

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
    // Load .js file to start the server.

    // Wait for the server to be ready.

    // HEADERS="$(mktemp)"
    // Call http://${AWS_LAMBDA_RUNTIME_API}/2018-06-01/runtime/invocation/next
    // Translate event into http request.

    // EVENT_DATA=$(curl -sS -LD "$HEADERS" "http://${AWS_LAMBDA_RUNTIME_API}/2018-06-01/runtime/invocation/next")
    // REQUEST_ID=$(grep -Fi Lambda-Runtime-Aws-Request-Id "$HEADERS" | tr -d '[:space:]' | cut -d: -f2)

    // Call our server with the http request.

    // Translate http response into event.
    // Call http://${AWS_LAMBDA_RUNTIME_API}/2018-06-01/runtime/invocation/${REQUEST_ID}/response

    println!("[Adapter] Starting server...");
    starter().await;

    // @TODO: Graceful shutdown server on SIGTERM.
    // match signal::ctrl_c().await {
    //     Ok(()) => {},
    //     Err(err) => {
    //         eprintln!("Unable to listen for shutdown signal: {}", err);
    //         // we also shut down in case of error
    //     },
    // }

    println!("[Adapter] Waiting for server to start...");
    checker().await;

    println!("Starting handler...");
    let result = lambda_http::run(service_fn(translator)).await;

    match result {
        Ok(_) => println!("Success"),
        Err(err) => println!("[Adapter] Error: {}", err),
    }

    // @TODO: Allow for streaming response.
    // let invoke_mode = std::env::var("INVOKE_MODE");
    // let result = match invoke_mode {
    //     "buffered" => lambda_http::run(service_fn(translator)).await,
    //     "response_streaming" => lambda_http::run_with_streaming_response(service_fn(translator)).await,
    //     _ => panic!("Unknown invoke mode: {}", invoke_mode),
    // };

    Ok(())
}
