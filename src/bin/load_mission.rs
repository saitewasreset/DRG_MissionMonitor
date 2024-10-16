use actix_web::web::Buf;
use encoding_rs::{DecoderResult, UTF_16LE, UTF_8};
use mission_backend_rs::client::*;
use mission_backend_rs::db::mission_log::*;
use mission_backend_rs::mission::APIMission;
use mission_backend_rs::APIResponse;
use mission_backend_rs::ClientConfig;
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use reqwest::cookie::Jar;
use reqwest::{StatusCode, Url};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time;

const MAX_LOG_LENGTH: usize = 64 * 1024 * 1024;

fn main() {
    author_info();
    let config_file_path = match env::var("CONFIG_PATH") {
        Ok(val) => PathBuf::from_str(&val).expect("invalid CONFIG_PATH"),
        Err(_) => PathBuf::from_str("./config.json").unwrap(),
    };

    let file_content = match fs::read(&config_file_path) {
        Ok(val) => val,
        Err(e) => {
            panic!(
                "cannot read config file {}: {}",
                config_file_path.as_os_str().to_str().unwrap(),
                e
            );
        }
    };

    let config: ClientConfig = match serde_json::from_slice(&file_content[..]) {
        Ok(val) => val,
        Err(e) => {
            panic!(
                "cannot parse config file {}: {}",
                config_file_path.as_os_str().to_str().unwrap(),
                e
            );
        }
    };

    if config.access_token.is_none() {
        println!("warning: no access token specified!");
    }

    let access_token = config.access_token.unwrap_or("Rock and stone!".to_string());

    let endpoint_url = config.endpoint_url;

    let upload_url = format!("{}/mission/load_mission", endpoint_url);
    let mission_list_url = format!("{}/mission/api_mission_list", endpoint_url);

    println!("upload url: {}", upload_url);
    println!("mission list url: {}", mission_list_url);

    let cookie_jar = Arc::new(Jar::default());

    let http_client = ClientBuilder::new()
        .cookie_provider(cookie_jar.clone())
        .build()
        .unwrap();

    let upload_url = upload_url
        .parse::<Url>()
        .expect("failed parsing endpoint url");

    cookie_jar.add_cookie_str(
        &format!("access_token = {};", access_token).as_str(),
        &upload_url,
    );

    let response: APIResponse<Vec<APIMission>> = match http_client
        .get(
            mission_list_url
                .parse::<Url>()
                .expect("failed parsing mission list url"),
        )
        .send()
    {
        Ok(response) => match response.status() {
            StatusCode::OK => {
                let body = response.bytes().expect("failed fetching response body");
                match serde_json::from_reader(body.reader()) {
                    Ok(x) => x,
                    Err(e) => panic!("failed parsing response body {}", e),
                }
            }
            other => {
                println!("unexpected status code from server: {}", other);
                println!("body: {:?}", response.text());
                panic!("cannot get mission list");
            }
        },
        Err(e) => {
            println!("failed sending request: {}", e);
            panic!("cannot get mission list");
        }
    };

    let mission_list = response.data.unwrap();

    println!("remote mission count: {}", mission_list.len());

    let mut mission_timestamp_list = mission_list
        .iter()
        .map(|item| item.begin_timestamp)
        .collect::<Vec<i64>>();

    mission_timestamp_list.sort_unstable();

    let start = time::Instant::now();
    let mission_list = parse_mission_log(Path::new("./raw_log")).ok().unwrap();
    println!(
        "loaded {} missions in {:?}",
        mission_list.len(),
        start.elapsed()
    );

    let to_upload_mission_list = mission_list
        .into_iter()
        .filter(|item| {
            mission_timestamp_list
                .binary_search(&item.mission_info.begin_timestamp)
                .is_err()
        })
        .collect::<Vec<LogContent>>();

    println!("to upload mission count: {}", to_upload_mission_list.len());

    let serialized = rmp_serde::to_vec(&to_upload_mission_list).unwrap();

    let compressed = compress(&serialized);

    println!("sending request and waiting for mission loading..");
    match http_client.post(upload_url).body(compressed).send() {
        Ok(response) => match response.status() {
            StatusCode::OK => {
                match update_cache(
                    &[
                        CacheType::MissionRawCache,
                        CacheType::MissionKPIRawCache,
                        CacheType::GlobalKPIState,
                    ],
                    &endpoint_url,
                    &http_client,
                ) {
                    Ok(_) => {
                        println!("Success. Rock and stone!");
                    }
                    Err(e) => {
                        println!("failed updating cache: {}", e);
                    }
                }
            }
            other => {
                println!("unexpected status code from server: {}", other);
                println!("body: {:?}", response.text());
            }
        },
        Err(e) => {
            println!("failed sending request: {}", e);
        }
    }
}

fn compress(data: &[u8]) -> Vec<u8> {
    println!("Serialized len = {}", format_size(data.len()));

    let compressed = Vec::with_capacity(data.len());

    let start = time::Instant::now();

    let mut encoder = zstd::Encoder::new(compressed, 15).unwrap();

    encoder.write_all(&data).unwrap();
    let mut compressed = encoder.finish().unwrap();

    let finish = time::Instant::now();

    println!(
        "Compressed using zstd, compressed len = {} with level 15, time: {:?}",
        format_size(compressed.len()),
        finish.duration_since(start)
    );

    compressed.shrink_to_fit();
    compressed
}

fn get_log_file_list(base_path: &Path) -> Vec<PathBuf> {
    let re = Regex::new("MissionMonitor_([0-9]+).txt").unwrap();
    std::fs::read_dir(base_path)
        .unwrap()
        .into_iter()
        .filter(|r| {
            re.is_match(
                r.as_ref()
                    .unwrap()
                    .file_name()
                    .as_os_str()
                    .to_str()
                    .unwrap(),
            )
        })
        .map(|r| r.unwrap().path())
        .collect()
}

fn parse_mission_log(base_path: &Path) -> Result<Vec<LogContent>, String> {
    let file_path_list = get_log_file_list(base_path);

    let mut parsed_mission_list = Vec::new();
    for file_path in file_path_list {
        parsed_mission_list.push(get_file_content_parted(&file_path).map_err(|e| {
            format!(
                "cannot parse log: {}: {}",
                &file_path.as_os_str().to_str().unwrap(),
                e
            )
        })?);
    }

    parsed_mission_list.sort_unstable_by(|a, b| {
        a.mission_info
            .begin_timestamp
            .cmp(&b.mission_info.begin_timestamp)
    });

    let mut deep_dive_mission_list = Vec::new();

    for mission in &parsed_mission_list {
        let first_player_join_time = mission
            .player_info
            .iter()
            .map(|p| p.join_mission_time)
            .min()
            .unwrap();

        if first_player_join_time > 0 {
            deep_dive_mission_list.push(mission.mission_info.begin_timestamp);
        }
    }

    for i in 0..parsed_mission_list.len() {
        let list_ptr = parsed_mission_list.as_mut_ptr();

        // SAFETY: 0 <= i < parsed_mission_list.len()

        let current_mission = unsafe { &mut *list_ptr.add(i) };

        let prev_mission = match i {
            0 => None,
            // SAFETY:
            // 1. 0 <= x - 1 < parsed_mission_list.len()
            // 2. x - 1 = i - 1 != i
            x => unsafe { Some(&mut *list_ptr.add(x - 1)) },
        };

        // 对于深潜，第一层对应的first_player_join_time为0，而二、三层不为0
        // 对于普通深潜，每一层的难度都显示为0.75（3）
        if deep_dive_mission_list
            .binary_search(&current_mission.mission_info.begin_timestamp)
            .is_ok()
        {
            // 若当前任务first_player_join_time不为0，但前一任务为0，说明当前是第二层，前一任务是第一层
            // 若当前任务first_player_join_time不为0，前一任务也不为0，说明当前是第三层，前一任务是第二层
            // 注：除非在第一层手动放弃任务，否则不论第二层是否胜利，都会有第二层的数据
            // 若在第一层手动放弃任务，则第一层无法识别为深潜
            if let Some(prev_mission) = prev_mission {
                match deep_dive_mission_list
                    .binary_search(&prev_mission.mission_info.begin_timestamp)
                {
                    Ok(_) => {
                        // 前一层是第二层，当前是第三层
                        if prev_mission.mission_info.hazard_id == 3
                            || prev_mission.mission_info.hazard_id == 101
                        {
                            // 普通深潜
                            prev_mission.mission_info.hazard_id = 101;
                            current_mission.mission_info.hazard_id = 102;
                        } else {
                            // 精英深潜
                            prev_mission.mission_info.hazard_id = 104;
                            current_mission.mission_info.hazard_id = 105;
                        }
                    }
                    Err(_) => {
                        // 前一层是第一层，当前是第二层
                        if prev_mission.mission_info.hazard_id == 3
                            || prev_mission.mission_info.hazard_id == 100
                        {
                            // 普通深潜
                            prev_mission.mission_info.hazard_id = 100;
                            current_mission.mission_info.hazard_id = 101;
                        } else {
                            // 精英深潜
                            prev_mission.mission_info.hazard_id = 103;
                            current_mission.mission_info.hazard_id = 104;
                        }
                    }
                }
            }
        }
    }

    Ok(parsed_mission_list)
}

fn get_file_content_parted(file_path: &Path) -> Result<LogContent, Box<dyn std::error::Error>> {
    let raw_file_content = std::fs::read(file_path)?;

    let mut file_content = String::with_capacity(MAX_LOG_LENGTH);

    if raw_file_content[0] == 0xFF && raw_file_content[1] == 0xFE {
        // UTF-16-LE
        let mut decoder = UTF_16LE.new_decoder();

        let (result, _) = decoder.decode_to_string_without_replacement(
            &raw_file_content,
            &mut file_content,
            false,
        );
        if let DecoderResult::Malformed(_, _) = result {
            panic!(
                "Cannot decode input: {} with UTF-16-LE",
                file_path.file_name().unwrap().to_str().unwrap()
            );
        }
    } else {
        let mut decoder = UTF_8.new_decoder();
        let (result, _) = decoder.decode_to_string_without_replacement(
            &raw_file_content,
            &mut file_content,
            true,
        );
        if let DecoderResult::Malformed(_, _) = result {
            panic!(
                "Cannot decode input: {} with UTF-8",
                file_path.file_name().unwrap().to_str().unwrap()
            );
        }
    }

    file_content.shrink_to_fit();

    let file_part_list = file_content.split("______").collect::<Vec<&str>>();

    let mission_info = LogMissionInfo::try_from(file_content.as_str())
        .map_err(|e| format!("load mission info: {}", e))?;

    let player_info_part = file_part_list[1];

    let mut player_info: Vec<LogPlayerInfo> = Vec::new();

    for player_info_line in player_info_part.lines() {
        if player_info_line.trim().len() == 0 {
            continue;
        }
        player_info.push(
            player_info_line
                .try_into()
                .map_err(|e| format!("load player info: {}", e))?,
        );
    }

    let damage_info_part = file_part_list[2];

    let mut damage_info: Vec<LogDamageInfo> = Vec::new();

    for damage_info_line in damage_info_part.lines() {
        if damage_info_line.trim().len() == 0 {
            continue;
        }
        damage_info.push(
            damage_info_line
                .try_into()
                .map_err(|e| format!("load player info: {}", e))?,
        );
    }

    let mut range_begin_idx: usize = 0;
    let mut range_begin_item = &damage_info[range_begin_idx];

    let mut combined_damage_info: Vec<LogDamageInfo> = Vec::with_capacity(damage_info.len());

    for (i, current_damage_info) in damage_info.iter().enumerate() {
        if !current_damage_info.combine_eq(range_begin_item) {
            let range_end_idx = i;

            let damage_sum = damage_info[range_begin_idx..range_end_idx]
                .iter()
                .map(|item| item.damage)
                .sum::<f64>();

            combined_damage_info.push(LogDamageInfo {
                mission_time: range_begin_item.mission_time,
                damage: damage_sum,
                taker: range_begin_item.taker.clone(),
                causer: range_begin_item.causer.clone(),
                weapon: range_begin_item.weapon.clone(),
                causer_type: range_begin_item.causer_type,
                taker_type: range_begin_item.taker_type,
            });

            range_begin_idx = i;
            range_begin_item = &damage_info[range_begin_idx];
        }
    }

    let range_end_idx = damage_info.len();

    let damage_sum = damage_info[range_begin_idx..range_end_idx]
        .iter()
        .map(|item| item.damage)
        .sum::<f64>();

    combined_damage_info.push(LogDamageInfo {
        mission_time: range_begin_item.mission_time,
        damage: damage_sum,
        taker: range_begin_item.taker.clone(),
        causer: range_begin_item.causer.clone(),
        weapon: range_begin_item.weapon.clone(),
        causer_type: range_begin_item.causer_type,
        taker_type: range_begin_item.taker_type,
    });

    let kill_info_part = file_part_list[3];

    let mut kill_info: Vec<LogKillInfo> = Vec::new();

    for kill_info_line in kill_info_part.lines() {
        if kill_info_line.trim().len() == 0 {
            continue;
        }
        kill_info.push(
            kill_info_line
                .try_into()
                .map_err(|e| format!("load kill info: {}", e))?,
        );
    }

    let resource_info_part = file_part_list[4];

    let mut resource_info: Vec<LogResourceInfo> = Vec::new();

    for resource_info_line in resource_info_part.lines() {
        if resource_info_line.trim().len() == 0 {
            continue;
        }
        resource_info.push(
            resource_info_line
                .try_into()
                .map_err(|e| format!("load resource info: {}", e))?,
        );
    }

    let supply_info_part = file_part_list[5];
    let mut supply_info: Vec<LogSupplyInfo> = Vec::new();

    for supply_info_line in supply_info_part.lines() {
        if supply_info_line.trim().len() == 0 {
            continue;
        }
        supply_info.push(
            supply_info_line
                .try_into()
                .map_err(|e| format!("load supply info: {}", e))?,
        );
    }

    let mission_time = mission_info.mission_time;

    // Fix total present time

    for current_player_info in &mut player_info {
        if current_player_info.total_present_time == 0 {
            current_player_info.total_present_time = mission_time;
        }
    }

    let first_player_join_time = player_info
        .iter()
        .map(|player| player.join_mission_time)
        .min()
        .ok_or(String::from("player count is 0"))?;

    // Fix time for damage info, killed info, resource info, supply info

    for current_damage_info in &mut damage_info {
        current_damage_info.mission_time -= first_player_join_time;
    }

    for current_kill_info in &mut kill_info {
        current_kill_info.mission_time -= first_player_join_time;
    }

    for current_resource_info in &mut resource_info {
        current_resource_info.mission_time -= first_player_join_time;
    }

    for current_supply_info in &mut supply_info {
        current_supply_info.mission_time -= first_player_join_time;
    }

    Ok(LogContent {
        mission_info,
        player_info,
        damage_info: combined_damage_info,
        kill_info,
        resource_info,
        supply_info,
    })

    // Identify Deep Dive in get_mission_list
}

fn format_size(size: usize) -> String {
    match size {
        0..1024 => format!("{}B", size),
        1024..1048576 => format!("{:.2}KiB", size as f64 / 1024.0),
        1048576.. => format!("{:.2}MiB", size as f64 / (1024.0 * 1024.0)),
    }
}
