use oort_backend_service::{leaderboard, project_id, shortcode, telemetry, tournament};
use salvo::cors::Cors;
use salvo::prelude::*;

#[handler]
async fn nop(_req: &mut Request, res: &mut Response) {
    res.render("");
}

#[tokio::main]
async fn main() {
    stackdriver_logger::init_with_cargo!();

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

    let cors_handler = Cors::builder()
        .allow_any_origin()
        .allow_methods(vec!["POST", "OPTIONS"])
        .allow_header("content-type")
        .build();

    let router = Router::with_hoop(cors_handler)
        .push(
            Router::with_path("/leaderboard")
                .get(leaderboard::get_leaderboard)
                .options(nop),
        )
        .push(
            Router::with_path("/leaderboard")
                .post(leaderboard::PostLeaderboard {})
                .options(nop),
        )
        .push(
            Router::with_path("/telemetry")
                .post(telemetry::post_telemetry)
                .options(nop),
        )
        .push(
            Router::with_path("/shortcode/<id>")
                .get(shortcode::get_shortcode)
                .options(nop),
        )
        .push(
            Router::with_path("/shortcode")
                .post(shortcode::post_shortcode)
                .options(nop),
        )
        .push(
            Router::with_path("/tournament/submit")
                .post(tournament::submit_tournament)
                .options(nop),
        )
        .push(
            Router::with_path("/tournament/results/<id>")
                .get(tournament::get_tournament_results)
                .options(nop),
        );

    Server::new(TcpListener::bind(&format!("0.0.0.0:{port}")))
        .serve(router)
        .await;
}
