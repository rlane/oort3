use axum::Router;
use clap::{Parser, Subcommand};
use http::Method;
use oort_backend_service::{leaderboard, project_id, rescore, shortcode, telemetry, tournament};
use tower_http::cors::{Any, CorsLayer};

#[derive(Parser, Debug)]
#[clap()]
struct Arguments {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Serve,
    Rescore {
        #[clap(short = 'n', long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    stackdriver_logger::init_with_cargo!();
    let args = Arguments::parse();
    match args.cmd {
        SubCommand::Serve => serve().await,
        SubCommand::Rescore { dry_run } => rescore::rescore(dry_run).await,
    }
}

async fn serve() -> anyhow::Result<()> {
    let mut port: u16 = 8080;
    match std::env::var("PORT") {
        Ok(p) => {
            match p.parse::<u16>() {
                Ok(n) => {
                    port = n;
                }
                Err(_e) => {}
            };
        }
        Err(_e) => {}
    };

    log::info!("Starting oort_backend_service");
    log::info!("Using project ID {}", project_id());
    log::info!(
        "hashed envelope secret: {:?}",
        &oort_envelope::hashed_secret()
    );

    let leaderboard_cache: leaderboard::SharedLeaderboardCache =
        std::sync::Arc::new(leaderboard::LeaderboardCache::new());

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    let router = {
        use axum::routing::{get, post};
        Router::new()
            .route("/shortcode/:id", get(shortcode::get))
            .route("/shortcode", post(shortcode::post))
            .route("/telemetry", post(telemetry::post))
            .route("/tournament/submit", post(tournament::submit))
            .route("/tournament/results/:id", get(tournament::get_results))
            .route("/leaderboard/:scenario_name", get(leaderboard::get))
            .route("/leaderboard", post(leaderboard::post))
            .with_state(leaderboard_cache)
            .layer(cors)
            .layer(tower_http::trace::TraceLayer::new_for_http())
    };

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
