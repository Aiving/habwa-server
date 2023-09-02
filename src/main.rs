pub(crate) mod data;
pub(crate) mod loaders;
pub(crate) mod parsers;
pub(crate) mod routes;
pub(crate) mod utils;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use clap::{arg, command, Parser};
use core::panic;
use data::app;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    server_path: String,

    #[arg(short, long)]
    password: Option<String>,

    #[arg(short, long)]
    encryption_key: Option<String>,

    #[arg(short, long)]
    ssl: bool,

    #[arg(short, long)]
    ssl_key: Option<String>,

    #[arg(short, long)]
    ssl_cert: Option<String>,

    #[arg(long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    if !utils::scan_folder(
        args.server_path.clone(),
        vec![
            "config",
            "libraries",
            "mods",
            "world",
            "eula.txt",
            "server.properties",
        ],
    ) {
        panic!("Invalid server path!")
    }

    let server_path = Path::new(&args.server_path).to_path_buf();

    let server_properties = {
        let file = fs::read_to_string(server_path.join("server.properties")).await?;

        data::server::Properties::new(file)
    };

    let client = utils::connect_rcon(
        server_properties.clone().ip,
        server_properties.clone().rcon.port,
        server_properties.clone().rcon.password,
    )
    .await;

    let mut access_token = None;

    if let Some(password) = args.password {
        if let Some(encryption_key) = args.encryption_key.clone() {
            access_token = Some(utils::encode_password(password, encryption_key));
        }
    }

    let mods = utils::load_mods(server_path.clone()).await;

    let state = Arc::new(app::State::new(
        client,
        server_path,
        server_properties,
        access_token.as_ref(),
        args.encryption_key.as_ref(),
        mods,
    ));

    let app = Router::new()
        .route("/auth", post(routes::auth::execute))
        .route("/server/config", get(routes::server::config::execute))
        .route("/server/mods", get(routes::server::mods::execute))
        .route("/server/mods/upload", post(routes::server::mods::upload))
        .route("/server/players", get(routes::server::players::execute))
        .route("/server/restart", get(routes::server::restart::execute))
        .layer(DefaultBodyLimit::disable())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    let mut config = None;

    if args.ssl {
        if args.ssl_key.is_none() || args.ssl_cert.is_none() {
            panic!("IMAGINE A GOOD WEB SERVER WITH SSL AND WITHOUT A KEY OR CERTIFICATE");
        }

        config = Some(
            RustlsConfig::from_pem_file(
                PathBuf::from(args.ssl_cert.unwrap()),
                PathBuf::from(args.ssl_key.unwrap()),
            )
            .await
            .unwrap(),
        );
    }

    println!("Running on 0.0.0.0:{}", args.port);

    if let Some(config) = config {
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    Ok(())
}
