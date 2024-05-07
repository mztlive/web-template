mod actors;
mod config;
mod database;
mod domain;
mod handles;
mod jwt;

use actors::{id_gen::IDGeneratorHandler, rbac::RbacActorHandler};
use clap::Parser;
use config::{AppConfig, AppState};
use database::repositories;
use handles::routes;
use mongodb::{Client, Database};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "./config.toml")]
    config_path: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let app_cfg = config::load_config(&args.config_path)
        .await
        .expect("Failed to load config");

    let (client, db) = database::mongodb::connect(&app_cfg.database.uri, &app_cfg.database.db_name)
        .await
        .expect("Failed to connect to database");

    let id_gen = IDGeneratorHandler::new();

    let jwt_engine = jwt::Engine::new(app_cfg.secret.clone()).expect("Failed to create jwt engine");

    let rbac_engine = RbacActorHandler::new(
        db.clone(),
        Box::new(repositories::role::RoleRepository::new()),
        Box::new(repositories::user::UserRepository::new()),
    )
    .await;

    start(app_cfg, client, db, id_gen, jwt_engine, rbac_engine).await
}

async fn start(
    cfg: AppConfig,
    client: Client,
    db: Database,
    id_gen: IDGeneratorHandler,
    jwt_engine: jwt::Engine,
    rbac: RbacActorHandler,
) {
    let state = AppState {
        client,
        db,
        config: cfg,
        id_gen,
        jwt: jwt_engine,
        rbac,
    };

    let app = routes::create(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:10001")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
