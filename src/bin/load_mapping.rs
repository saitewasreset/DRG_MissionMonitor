use actix_web::web::Buf;
use mission_backend_rs::client::*;
use mission_backend_rs::{APIResponse, ClientConfig, Mapping};
use reqwest::cookie::Jar;
use reqwest::{blocking::ClientBuilder, StatusCode, Url};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs, path::PathBuf};

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
                config_file_path.to_string_lossy(),
                e
            );
        }
    };

    let config: ClientConfig = match serde_json::from_slice(&file_content[..]) {
        Ok(val) => val,
        Err(e) => {
            panic!(
                "cannot parse config file {}: {}",
                config_file_path.to_string_lossy(),
                e
            );
        }
    };

    if config.access_token.is_none() {
        println!("warning: no access token specified!");
    }

    let access_token = config.access_token.unwrap_or("Rock and stone!".to_string());

    let mapping_path = match config.mapping_path {
        Some(mut path) => {
            if !(path.ends_with('/') || path.ends_with('\\')) {
                path.push('/');
            }
            PathBuf::from_str(&path).expect("invalid mapping path")
        }
        None => PathBuf::from_str("./mapping/").unwrap(),
    };

    let entity_black_list_path = mapping_path.as_path().join("entity_blacklist.txt");

    let entity_black_list_file_content = match fs::read_to_string(&entity_black_list_path) {
        Ok(content) => content,
        Err(e) => {
            println!(
                "failed reading mapping file {}: {}, default value will be used",
                entity_black_list_path.as_os_str().to_str().unwrap(),
                e
            );
            String::new()
        }
    };

    let scout_special_list_path = mapping_path.as_path().join("scout_special.txt");

    let scout_special_list_file_content = match fs::read_to_string(&scout_special_list_path) {
        Ok(content) => content,
        Err(e) => {
            println!(
                "failed reading mapping file {}: {}, default value will be used",
                scout_special_list_path.as_os_str().to_str().unwrap(),
                e
            );
            String::new()
        }
    };

    let entity_blacklist = entity_black_list_file_content
        .lines()
        .filter(|&x| !x.trim().starts_with('#'))
        .map(|x| String::from(x))
        .collect::<Vec<String>>();

    let scout_special_list = scout_special_list_file_content
        .lines()
        .filter(|&x| !x.trim().starts_with('#'))
        .map(|x| String::from(x))
        .collect::<Vec<String>>();
    let character_mapping = parse_mapping_file(&mapping_path.join("character.txt"));
    let entity_mapping = parse_mapping_file(&mapping_path.join("entity.txt"));
    let entity_combine = parse_mapping_file(&mapping_path.join("entity_combine.txt"));
    let mission_type_mapping = parse_mapping_file(&mapping_path.join("mission_type.txt"));
    let resource_mapping = parse_mapping_file(&mapping_path.join("resource.txt"));
    let weapon_mapping = parse_mapping_file(&mapping_path.join("weapon.txt"));
    let weapon_combine = parse_mapping_file(&mapping_path.join("weapon_combine.txt"));
    let weapon_character = parse_mapping_file(&mapping_path.join("weapon_hero.txt"));

    let mapping = Mapping {
        character_mapping,
        entity_mapping,
        entity_combine,
        entity_blacklist_set: HashSet::from_iter(entity_blacklist.into_iter()),
        mission_type_mapping,
        resource_mapping,
        weapon_mapping,
        weapon_combine,
        weapon_character,
        scout_special_player_set: scout_special_list.into_iter().collect(),
    };

    let serialized = serde_json::to_vec(&mapping).unwrap();

    let cookie_jar = Arc::new(Jar::default());

    let http_client = ClientBuilder::new()
        .cookie_provider(cookie_jar.clone())
        .build()
        .unwrap();

    let endpoint_url = &config.endpoint_url;

    let upload_endpoint = format!("{}/admin/load_mapping", endpoint_url);

    println!("upload url: {}", upload_endpoint);

    cookie_jar.add_cookie_str(
        &format!("access_token = {};", access_token).as_str(),
        &upload_endpoint
            .parse::<Url>()
            .expect("failed parsing load mapping url"),
    );

    match http_client
        .post(
            upload_endpoint
                .parse::<Url>()
                .expect("failed parsing load mapping url"),
        )
        .body(serialized)
        .send()
    {
        Ok(response) => match response.status() {
            StatusCode::OK => {
                let body = response.bytes().expect("failed fetching response body");
                let api_response: APIResponse<()> = match serde_json::from_reader(body.reader()) {
                    Ok(x) => x,
                    Err(e) => panic!("failed parsing response body {}", e),
                };

                if api_response.code == 200 {
                    match update_cache(
                        &[
                            CacheType::MissionRawCache,
                            CacheType::MissionKPIRawCache,
                            CacheType::GlobalKPIState,
                        ],
                        endpoint_url,
                        &http_client,
                    ) {
                        Ok(_) => {
                            println!("Success. Rock and stone!");
                        }
                        Err(e) => {
                            println!("failed updating cache: {}", e);
                        }
                    }
                } else {
                    println!(
                        "Server returned {}: {}",
                        api_response.code, api_response.message
                    );
                }
            }
            other => {
                println!("unexpected status code from server: {}", other);
                println!("body: {:?}", response.text());
                panic!("cannot load mapping");
            }
        },
        Err(e) => {
            println!("failed sending request: {}", e);
            panic!("cannot load mapping");
        }
    }
}

fn parse_mapping_file(file_path: &Path) -> HashMap<String, String> {
    println!(
        "loading mapping: {}",
        file_path.as_os_str().to_str().unwrap()
    );
    let file_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            println!(
                "failed reading mapping file {}: {}, default value will be used",
                file_path.as_os_str().to_str().unwrap(),
                e
            );
            return HashMap::new();
        }
    };

    let mut result = HashMap::new();

    for split_line in file_content
        .lines()
        .filter(|&x| !x.trim().starts_with('#'))
        .map(|x| x.trim().split('|'))
    {
        let split_line = split_line.collect::<Vec<&str>>();

        if split_line.len() != 2 {
            continue;
        }

        result.insert(String::from(split_line[0]), String::from(split_line[1]));
    }

    result
}
