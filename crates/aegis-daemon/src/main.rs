//! Aegis Core daemon (story S1.1, partial).
//!
//! Configuration is read from the environment for now:
//! - `AEGIS_API_KEY` (required): the gateway client key.
//! - `AEGIS_AUDIT_PATH` (required): JSON-lines audit file.
//! - `AEGIS_BIND` (optional, default `127.0.0.1:8731`): loopback addresses only.
//!
//! No upstream provider exists yet, so allowed requests receive `no_provider`.
//! Logging is limited to fixed strings: no request data is ever printed.

use aegis_audit::JsonlFileSink;
use aegis_detect_secrets::SecretScanner;
use aegis_gateway::{key_is_strong, router, ClientRegistry, GatewayState, Limits, NoUpstream};
use aegis_inspect::Inspector;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

fn fail(message: &str) -> ExitCode {
    eprintln!("aegis-daemon: {message}");
    ExitCode::FAILURE
}

#[tokio::main]
async fn main() -> ExitCode {
    // Panic payloads can contain request data; print only the location.
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map_or_else(|| "unknown".to_string(), ToString::to_string);
        eprintln!("aegis-daemon: internal error at {location}");
    }));

    let Ok(api_key) = std::env::var("AEGIS_API_KEY") else {
        return fail("AEGIS_API_KEY is required");
    };
    if !key_is_strong(&api_key) {
        return fail(
            "AEGIS_API_KEY must be at least 32 characters with at least 12 distinct characters",
        );
    }
    let Ok(audit_path) = std::env::var("AEGIS_AUDIT_PATH") else {
        return fail("AEGIS_AUDIT_PATH is required");
    };
    let bind: SocketAddr = match std::env::var("AEGIS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8731".to_string())
        .parse()
    {
        Ok(addr) => addr,
        Err(_) => return fail("AEGIS_BIND is not a valid socket address"),
    };
    if !bind.ip().is_loopback() {
        return fail("only loopback binds are supported in this build");
    }

    let Ok(scanner) = SecretScanner::new() else {
        return fail("secret scanner failed to initialise");
    };
    let Ok(audit) = JsonlFileSink::open(&PathBuf::from(audit_path)) else {
        return fail("cannot open the audit file");
    };

    let mut clients = ClientRegistry::new();
    clients.add_key(1, &api_key);
    let state = GatewayState::new(
        clients,
        Inspector::new(vec![Arc::new(scanner)]),
        Arc::new(audit),
        Arc::new(NoUpstream),
        Limits::default(),
    );

    let Ok(listener) = tokio::net::TcpListener::bind(bind).await else {
        return fail("cannot bind the gateway address");
    };
    eprintln!("aegis-daemon: gateway listening on {bind}");
    let served = axum::serve(listener, router(Arc::new(state)))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await;
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => fail("gateway stopped unexpectedly"),
    }
}
