use crate::db::schema::*;
use diesel::prelude::*;
use log::{error, info};

pub fn delete_mission(db_conn: &mut PgConnection, mission_id: i32) -> Result<(), ()> {
    info!("deleting mission {}", mission_id);
    diesel::delete(mission::table.filter(mission::id.eq(mission_id)))
        .execute(db_conn)
        .map_err(|e| {
            error!("cannot delete mission {}: {}", mission_id, e);
        })?;
    diesel::delete(player_info::table.filter(player_info::mission_id.eq(mission_id)))
        .execute(db_conn)
        .map_err(|e| {
            error!(
                "cannot delete player_info for mission {}: {}",
                mission_id, e
            );
        })?;
    diesel::delete(damage_info::table.filter(damage_info::mission_id.eq(mission_id)))
        .execute(db_conn)
        .map_err(|e| {
            error!(
                "cannot delete damage_info for mission {}: {}",
                mission_id, e
            );
        })?;

    diesel::delete(kill_info::table.filter(kill_info::mission_id.eq(mission_id)))
        .execute(db_conn)
        .map_err(|e| {
            error!("cannot delete kill_info for mission {}: {}", mission_id, e);
        })?;

    diesel::delete(resource_info::table.filter(resource_info::mission_id.eq(mission_id)))
        .execute(db_conn)
        .map_err(|e| {
            error!(
                "cannot delete resource_info for mission {}: {}",
                mission_id, e
            );
        })?;

    diesel::delete(supply_info::table.filter(supply_info::mission_id.eq(mission_id)))
        .execute(db_conn)
        .map_err(|e| {
            error!(
                "cannot delete supply_info for mission {}: {}",
                mission_id, e
            );
        })?;
    Ok(())
}
