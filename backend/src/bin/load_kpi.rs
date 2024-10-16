use actix_web::web::Buf;
use mission_backend_rs::client::*;
use mission_backend_rs::kpi::*;
use mission_backend_rs::APIResponse;
use mission_backend_rs::ClientConfig;
use reqwest::blocking::ClientBuilder;
use reqwest::cookie::Jar;
use reqwest::StatusCode;
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Deserialize)]
struct EntityRecord {
    pub entity_game_id: String,
    pub priority: f64,
    pub driller: f64,
    pub gunner: f64,
    pub engineer: f64,
    pub scout: f64,
    pub scout_special: f64,
}

#[derive(Deserialize)]
struct ResourceRecord {
    pub resource_game_id: String,
    pub weight: f64,
}

fn main() -> Result<(), String> {
    author_info();
    let config_file_path = match env::var("CONFIG_PATH") {
        Ok(val) => PathBuf::from_str(&val).expect("invalid CONFIG_PATH"),
        Err(_) => PathBuf::from_str("./config.json").unwrap(),
    };

    let file_content = fs::read(&config_file_path).map_err(|e| {
        format!(
            "cannot read config file {}: {}",
            config_file_path.to_string_lossy(),
            e
        )
    })?;

    let config: ClientConfig = serde_json::from_slice(&file_content[..]).map_err(|e| {
        format!(
            "cannot parse config file {}: {}",
            config_file_path.to_string_lossy(),
            e
        )
    })?;

    if config.access_token.is_none() {
        println!("warning: no access token specified!");
    }

    let access_token = config.access_token.unwrap_or("Rock and stone!".to_string());

    let kpi_config_path = match config.kpi_config_path {
        Some(mut path) => {
            if !(path.ends_with('/') || path.ends_with('\\')) {
                path.push('/');
            }
            PathBuf::from_str(&path).expect("invalid kpi path")
        }
        None => PathBuf::from_str("./kpi/").unwrap(),
    };

    let character_component_weight =
        load_character_component_weight(&kpi_config_path.join("character_component_weight.txt"))
            .map_err(|e| format!("cannot load character component weight: {}", e))?;

    let (character_weight_table, priority_table) =
        load_damage_weight_table(&kpi_config_path.join("entity_list_combined.csv"))
            .map_err(|e| format!("cannot load damage weight table: {}", e))?;

    let resource_weight_table = load_resource_table(&kpi_config_path.join("resource_table.csv"))
        .map_err(|e| format!("cannot load resource weight table: {}", e))?;

    let transform_range = load_transform_range(&kpi_config_path.join("transform_range.txt"))
        .map_err(|e| format!("cannot load transform range: {}", e))?;

    let kpi_config = KPIConfig {
        character_weight_table,
        priority_table,
        resource_weight_table,
        character_component_weight,
        transform_range,
    };

    let serialized = serde_json::to_vec(&kpi_config).unwrap();

    let cookie_jar = Arc::new(Jar::default());

    let http_client = ClientBuilder::new()
        .cookie_provider(cookie_jar.clone())
        .build()
        .unwrap();

    let endpoint_url = &config.endpoint_url;

    let upload_endpoint = format!("{}/admin/load_kpi", endpoint_url);

    println!("upload endpoint: {}", upload_endpoint);

    cookie_jar.add_cookie_str(
        &format!("access_token = {};", access_token).as_str(),
        &upload_endpoint
            .parse::<Url>()
            .expect("failed parsing load kpi url"),
    );

    match http_client
        .post(
            upload_endpoint
                .parse::<Url>()
                .expect("failed parsing load kpi url"),
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
                        &[CacheType::MissionKPIRawCache, CacheType::GlobalKPIState],
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
                    return Err(format!(
                        "Server returned {}: {}",
                        api_response.code, api_response.message
                    ));
                }
            }
            other => {
                println!("unexpected status code from server: {}", other);
                println!("body: {:?}", response.text());
                return Err("cannot load kpi config".into());
            }
        },
        Err(e) => {
            return Err(format!("failed sending request: {}", e));
        }
    }

    Ok(())
}

fn load_character_component_weight(
    file_path: &Path,
) -> Result<HashMap<CharacterKPIType, HashMap<KPIComponent, f64>>, Box<dyn Error>> {
    let file_content = fs::read_to_string(file_path)?;

    let mut result = HashMap::new();

    for valid_line in file_content.lines().filter(|&x| !x.trim().starts_with('#')) {
        let valid_line_split = valid_line.split(' ').collect::<Vec<&str>>();

        let character_type_id = valid_line_split[0].parse::<i16>()?;

        let character_type = CharacterKPIType::try_from(character_type_id)?;

        let mut character_component_map = HashMap::new();

        for (index, value) in valid_line_split[1..].iter().enumerate() {
            let component = KPIComponent::try_from(index)?;
            let weight = value.parse::<f64>()?;

            character_component_map.insert(component, weight);
        }

        result.insert(character_type, character_component_map);
    }

    Ok(result)
}

// Return: (character_weight_table, priority_table)
fn load_damage_weight_table(
    file_path: &Path,
) -> Result<
    (
        HashMap<CharacterKPIType, HashMap<String, f64>>,
        HashMap<String, f64>,
    ),
    Box<dyn Error>,
> {
    let input_file = fs::File::open(file_path)?;

    let mut reader = csv::Reader::from_reader(input_file);

    let mut character_weight_table = HashMap::new();
    let mut priority_table = HashMap::new();

    for result in reader.deserialize() {
        let record: EntityRecord = result?;

        priority_table.insert(record.entity_game_id.clone(), record.priority);

        character_weight_table
            .entry(CharacterKPIType::Driller)
            .or_insert(HashMap::new())
            .insert(record.entity_game_id.clone(), record.driller);

        character_weight_table
            .entry(CharacterKPIType::Gunner)
            .or_insert(HashMap::new())
            .insert(record.entity_game_id.clone(), record.gunner);

        character_weight_table
            .entry(CharacterKPIType::Engineer)
            .or_insert(HashMap::new())
            .insert(record.entity_game_id.clone(), record.engineer);

        character_weight_table
            .entry(CharacterKPIType::Scout)
            .or_insert(HashMap::new())
            .insert(record.entity_game_id.clone(), record.scout);

        character_weight_table
            .entry(CharacterKPIType::ScoutSpecial)
            .or_insert(HashMap::new())
            .insert(record.entity_game_id.clone(), record.scout_special);
    }

    Ok((character_weight_table, priority_table))
}

fn load_resource_table(file_path: &Path) -> Result<HashMap<String, f64>, Box<dyn Error>> {
    let input_file = fs::File::open(file_path)?;

    let mut reader = csv::Reader::from_reader(input_file);

    let mut resource_table = HashMap::new();

    for result in reader.deserialize() {
        let record: ResourceRecord = result?;

        resource_table.insert(record.resource_game_id.clone(), record.weight);
    }

    Ok(resource_table)
}

fn load_transform_range(
    file_path: &Path,
) -> Result<Vec<IndexTransformRangeConfig>, Box<dyn Error>> {
    let file_content = fs::read_to_string(file_path)?;

    let mut file_lines_iter = file_content.lines().filter(|&x| !x.trim().starts_with('#'));

    let source_split = file_lines_iter
        .next()
        .ok_or("missing source line")?
        .split(' ')
        .collect::<Vec<_>>();

    let transformed_split = file_lines_iter
        .next()
        .ok_or("missing transformed line")?
        .split(' ')
        .collect::<Vec<_>>();

    if source_split.len() != transformed_split.len() {
        return Err("source and transformed line length mismatch".into());
    }

    let mut result = Vec::with_capacity(source_split.len() - 1);

    for i in 0..source_split.len() - 1 {
        let source_begin = source_split[i].parse::<f64>()?;
        let source_end = source_split[i + 1].parse::<f64>()?;

        let transformed_begin = transformed_split[i].parse::<f64>()?;
        let transformed_end = transformed_split[i + 1].parse::<f64>()?;

        result.push(IndexTransformRangeConfig {
            rank_range: (source_begin, source_end),
            transform_range: (transformed_begin, transformed_end),
        });
    }

    Ok(result)
}
