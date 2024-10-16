use actix_web::web;
pub mod brothers;
pub mod weapon;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(brothers::get_brothers_info);
    cfg.service(weapon::get_weapon_preference);
}
