use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use diesel::{Connection, PgConnection};
use env_logger::Env;
use log::{error, info, warn};
use mission_backend_rs::cache;
use mission_backend_rs::damage;
use mission_backend_rs::general;
use mission_backend_rs::get_mapping;
use mission_backend_rs::info;
use mission_backend_rs::kpi;
use mission_backend_rs::kpi::KPIConfig;
use mission_backend_rs::mission;
use mission_backend_rs::AppState;
use mission_backend_rs::DbPool;
use mission_backend_rs::Mapping;
use mission_backend_rs::{admin, echo_heartbeat};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

const MAX_BODY_LENGTH: usize = 64 * 1024 * 1024;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = read_file_env("DATABASE_URL").expect("cannot get database url");
    let redis_url = read_file_env("REDIS_URL").expect("cannot get redis url");

    let access_token = read_file_env("ACCESS_TOKEN");

    if access_token.is_none() {
        warn!("cannot get access token, any token would be accepted, check ACCESS_TOKEN_FILE or ACCESS_TOKEN enviroment variable");
    }

    let instance_dir = read_file_env("INSTANCE_DIR");

    let instance_dir = match instance_dir {
        Some(mut x) => {
            if !(x.ends_with('/') || x.ends_with('\\')) {
                x.push('/');
            }

            PathBuf::from_str(&x).expect("cannot parse instance dir")
        }
        None => PathBuf::from_str("instance/").unwrap(),
    };

    let _ = fs::create_dir(&instance_dir);

    let mapping = load_mapping(&instance_dir.as_path().join("mapping.json"));

    let kpi_config_path = instance_dir.as_path().join("kpi_config.json");

    let kpi_config: Option<KPIConfig> = match fs::read(&kpi_config_path) {
        Ok(x) => match serde_json::from_slice(&x[..]) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot parse kpi config: {}", e);
                None
            }
        },
        Err(e) => {
            error!("cannot read kpi config: {}", e);
            None
        }
    };

    let mut conn = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    conn.run_pending_migrations(MIGRATIONS)
        .expect("cannot run migrations");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let db_pool: DbPool = match Pool::new(manager) {
        Ok(x) => x,
        Err(e) => {
            panic!("cannot build database pool: {}", e);
        }
    };

    let redis_client = match redis::Client::open(redis_url) {
        Ok(x) => x,
        Err(e) => {
            panic!("cannot connect to redis: {}", e);
        }
    };

    let inner_mapping = Mutex::new(mapping);
    let inner_kpi_config = Mutex::new(kpi_config);

    let app_state = web::Data::new(AppState {
        access_token,
        instance_path: instance_dir.clone(),
        mapping: inner_mapping,
        kpi_config: inner_kpi_config,
    });
    let db_pool = web::Data::new(db_pool);
    let redis_client = web::Data::new(redis_client);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(app_state.clone())
            .app_data(db_pool.clone())
            .app_data(redis_client.clone())
            .app_data(web::PayloadConfig::default().limit(MAX_BODY_LENGTH))
            .service(
                web::scope("/api")
                    .service(echo_heartbeat)
                    .service(get_mapping)
                    .service(web::scope("/mission").configure(mission::scoped_config))
                    .service(web::scope("/admin").configure(admin::scoped_config))
                    .service(web::scope("/cache").configure(cache::scoped_config))
                    .service(web::scope("/damage").configure(damage::scoped_config))
                    .service(web::scope("/general").configure(general::scoped_config))
                    .service(web::scope("/kpi").configure(kpi::scoped_config))
                    .service(web::scope("/info").configure(info::scoped_config)),
            )
            .service(actix_files::Files::new("/", "/static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn read_file_env(target_env: &str) -> Option<String> {
    let mut result: Option<String> = None;
    if let Ok(file_path) = env::var(format!("{}_FILE", target_env)) {
        match fs::read_to_string(&file_path) {
            Ok(val) => result = Some(val.trim().to_string()),
            Err(e) => {
                warn!("cannot read env file {}: {}", file_path, e)
            }
        }
    }

    if result.is_none() {
        if let Ok(env_str) = env::var(target_env) {
            result = Some(env_str);
        }
    }

    return result;
}

fn load_mapping(mapping_path: &Path) -> Mapping {
    info!("loading mapping from: {}", mapping_path.to_string_lossy());
    let file_content = match fs::read(mapping_path) {
        Ok(x) => x,
        Err(e) => {
            error!(
                "failed loading mapping {}: {}, default value will be used",
                mapping_path.to_string_lossy(),
                e
            );
            return Mapping::default();
        }
    };

    match serde_json::from_slice(&file_content[..]) {
        Ok(x) => x,
        Err(e) => {
            error!(
                "failed parsing mapping {}: {}, default value will be used",
                mapping_path.to_string_lossy(),
                e
            );
            return Mapping::default();
        }
    }
}
