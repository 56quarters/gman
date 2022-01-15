use clap::Parser;
use gman::client::WeatherGovClient;
use gman::http::http_route;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use reqwest::Client;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::process;
use std::time::{Duration, Instant};
use tokio::signal::unix::{self, SignalKind};
use tracing::{event, Level};

/*
const UNIT_METERS: &str = "wmoUnit:m";
const UNIT_DEGREES_C: &str = "wmoUnit:degC";
const UNIT_PERCENT: &str = "wmoUnit:percent";
const UNIT_DEGREES_ANGLE: &str = "wmoUnit:degree_(angle)";
const UNIT_KPH: &str = "wmoUnit:km_h-1";
const UNIT_PASCALS: &str = "wmoUnit:Pa";
 */

const DEFAULT_LOG_LEVEL: Level = Level::INFO;
const DEFAULT_BIND_ADDR: ([u8; 4], u16) = ([0, 0, 0, 0], 9782);
const DEFAULT_REFERSH_SECS: u64 = 300;
const DEFAULT_API_URL: &'static str = "https://api.weather.gov/";

#[derive(Debug, Parser)]
#[clap(name = "gman", version = clap::crate_version ! ())]
struct GmanApplication {
    /// NWS weather station ID to fetch forecasts for
    #[clap(long)]
    station: String,

    /// Base URL for the Weather.gov API
    #[clap(long, default_value_t = DEFAULT_API_URL.into())]
    api_url: String,

    /// Logging verbosity. Allowed values are 'trace', 'debug', 'info', 'warn', and 'error'
    /// (case insensitive)
    #[clap(long, default_value_t = DEFAULT_LOG_LEVEL)]
    log_level: Level,

    /// Fetch weather forecasts from the Weather.gov API at this interval, in seconds.
    #[clap(long, default_value_t = DEFAULT_REFERSH_SECS)]
    refresh_secs: u64,

    /// Address to bind to. By default, gman will bind to public address since
    /// the purpose is to expose metrics to an external system (Prometheus or another
    /// agent for ingestion)
    #[clap(long, default_value_t = DEFAULT_BIND_ADDR.into())]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let opts = GmanApplication::parse();
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(opts.log_level)
            .finish(),
    )
    .expect("failed to set tracing subscriber");

    let startup = Instant::now();
    // TODO(56quarters): Put a registry into a request context instead of using the global one?
    // TODO(56quarters): Add a trace to the request handler future
    let service = make_service_fn(move |_| async move { Ok::<_, hyper::Error>(service_fn(http_route)) });
    let server = Server::try_bind(&opts.bind).unwrap_or_else(|e| {
        event!(
            Level::ERROR,
            message = "server failed to start",
            error = %e,
            address = %opts.bind,
            api_url = %opts.api_url,
        );

        process::exit(1);
    });

    // TODO(56quarters): Do a client.station() call to make sure the station supplied by the
    //  user is valid before going into a loop making requests for it.
    let client = WeatherGovClient::new(Client::new(), &opts.api_url);
    let station = opts.station.clone();
    let interval = Duration::from_secs(opts.refresh_secs);

    tokio::spawn(async move {
        let mut interval_stream = tokio::time::interval(interval);

        loop {
            // TODO(56quarters): Something that owns a bunch of metrics and updates them
            //  based on the results of the API call.

            let _ = interval_stream.tick().await;

            // TODO(56quarters): Handle errors here and log them
            println!("{:?}", client.observation(&station).await);
        }
    });

    event!(
        Level::INFO,
        message = "server started",
        address = %opts.bind,
        api_url = %opts.api_url,
    );

    server
        .serve(service)
        .with_graceful_shutdown(async {
            // Wait for either SIGTERM or SIGINT to shutdown
            tokio::select! {
                _ = sigterm() => {}
                _ = sigint() => {}
            }
        })
        .await?;

    event!(
        Level::INFO,
        message = "server shutdown",
        runtime_secs = %startup.elapsed().as_secs(),
    );

    Ok(())
}

/// Return after the first SIGTERM signal received by this process
async fn sigterm() -> io::Result<()> {
    unix::signal(SignalKind::terminate())?.recv().await;
    Ok(())
}

/// Return after the first SIGINT signal received by this process
async fn sigint() -> io::Result<()> {
    unix::signal(SignalKind::interrupt())?.recv().await;
    Ok(())
}
