pub mod delete_mission;

use crate::kpi::KPIConfig;
use crate::{db::schema::player, APIResponse, AppState, DbPool, Mapping};
use actix_web::{
    post,
    web::{self, Buf, Bytes, Data, Json},
    HttpRequest,
};
use diesel::prelude::*;
use diesel::{insert_into, update};
use log::{error, warn};
use std::fs;

#[derive(Insertable)]
#[diesel(table_name = player)]
struct NewPlayer {
    pub player_name: String,
    pub friend: bool,
}

#[post("/load_mapping")]
async fn load_mapping(
    requests: HttpRequest,
    app_state: Data<AppState>,
    body: Bytes,
) -> Json<APIResponse<()>> {
    if let Some(access_token) = app_state.access_token.clone() {
        if let Some(provieded_access_token) = requests.cookie("access_token") {
            if provieded_access_token.value() != access_token {
                return Json(APIResponse::unauthorized());
            }
        } else {
            return Json(APIResponse::unauthorized());
        }
    }

    let mapping: Mapping = match serde_json::from_reader(body.reader()) {
        Ok(x) => x,
        Err(e) => {
            warn!("cannot parse payload body as json: {}", e);
            return Json(APIResponse::bad_request(
                "cannot parse payload body as json",
            ));
        }
    };

    let write_path = app_state.instance_path.as_path().join("./mapping.json");

    match fs::write(&write_path, serde_json::to_vec(&mapping).unwrap()) {
        Err(e) => {
            error!(
                "cannot write mapping to {}: {}",
                write_path.to_string_lossy(),
                e
            );
            return Json(APIResponse::internal_error());
        }
        Ok(()) => {
            let mut state_mapping = app_state.mapping.lock().unwrap();
            *state_mapping = mapping;
            return Json(APIResponse::ok(()));
        }
    }
}

#[post("/load_watchlist")]
async fn load_watchlist(
    requests: HttpRequest,
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    body: Bytes,
) -> Json<APIResponse<()>> {
    if let Some(access_token) = app_state.access_token.clone() {
        if let Some(provieded_access_token) = requests.cookie("access_token") {
            if provieded_access_token.value() != access_token {
                return Json(APIResponse::unauthorized());
            }
        } else {
            return Json(APIResponse::unauthorized());
        }
    }

    let watchlist: Vec<String> = match serde_json::from_reader(body.reader()) {
        Ok(x) => x,
        Err(e) => {
            return Json(APIResponse::bad_request(&format!(
                "cannot parse payload as json: {}",
                e
            )));
        }
    };

    let watchlist = watchlist
        .into_iter()
        .map(|player_name| NewPlayer {
            player_name,
            friend: true,
        })
        .collect::<Vec<_>>();

    let result = web::block(move || {
        let mut conn = match db_pool.get() {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get db connection from pool: {}", e);
                return Err(());
            }
        };

        match update(player::table)
            .set(player::friend.eq(false))
            .execute(&mut conn)
        {
            Ok(_) => {}
            Err(e) => {
                error!("cannot update db: {}", e);
                return Err(());
            }
        };

        match insert_into(player::table)
            .values(&watchlist)
            .on_conflict(player::player_name)
            .do_update()
            .set(player::friend.eq(true))
            .execute(&mut conn)
        {
            Ok(_) => {}
            Err(e) => {
                error!("cannot update db: {}", e);
                return Err(());
            }
        };

        Ok(())
    })
    .await
    .unwrap();

    match result {
        Ok(()) => Json(APIResponse::ok(())),
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[post("/load_kpi")]
async fn load_kpi(
    requests: HttpRequest,
    app_state: Data<AppState>,
    body: Bytes,
) -> Json<APIResponse<()>> {
    if let Some(access_token) = app_state.access_token.clone() {
        if let Some(provieded_access_token) = requests.cookie("access_token") {
            if provieded_access_token.value() != access_token {
                return Json(APIResponse::unauthorized());
            }
        } else {
            return Json(APIResponse::unauthorized());
        }
    }

    let kpi_config: KPIConfig = match serde_json::from_reader(body.reader()) {
        Ok(x) => x,
        Err(e) => {
            warn!("cannot parse payload body as json: {}", e);
            return Json(APIResponse::bad_request(
                "cannot parse payload body as json",
            ));
        }
    };

    let write_path = app_state.instance_path.as_path().join("./kpi_config.json");

    match fs::write(&write_path, serde_json::to_vec(&kpi_config).unwrap()) {
        Err(e) => {
            error!(
                "cannot write kpi config to {}: {}",
                write_path.to_string_lossy(),
                e
            );
            return Json(APIResponse::internal_error());
        }
        Ok(()) => {
            let mut state_kpi_config = app_state.kpi_config.lock().unwrap();
            *state_kpi_config = Some(kpi_config);
            return Json(APIResponse::ok(()));
        }
    }
}

#[post("/delete_mission")]
async fn api_delete_mission(
    requests: HttpRequest,
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    body: Bytes,
) -> Json<APIResponse<()>> {
    if let Some(access_token) = app_state.access_token.clone() {
        if let Some(provieded_access_token) = requests.cookie("access_token") {
            if provieded_access_token.value() != access_token {
                return Json(APIResponse::unauthorized());
            }
        } else {
            return Json(APIResponse::unauthorized());
        }
    }

    let to_delete_mission_list: Vec<i32> = match serde_json::from_reader(body.reader()) {
        Ok(x) => x,
        Err(e) => {
            warn!("cannot parse payload body as json: {}", e);
            return Json(APIResponse::bad_request(
                "cannot parse payload body as json",
            ));
        }
    };

    let result = web::block(move || {
        let mut conn = match db_pool.get() {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get db connection from pool: {}", e);
                return Err(());
            }
        };

        for mission_id in to_delete_mission_list {
            delete_mission::delete_mission(&mut conn, mission_id)?;
        }

        Ok(())
    })
    .await
    .unwrap();

    match result {
        Ok(()) => Json(APIResponse::ok(())),
        Err(()) => Json(APIResponse::internal_error()),
    }
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(load_mapping);
    cfg.service(load_watchlist);
    cfg.service(load_kpi);
    cfg.service(api_delete_mission);
}
