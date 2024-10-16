use actix_web::web::Buf;
use mission_backend_rs::client::*;
use mission_backend_rs::APIResponse;
use mission_backend_rs::ClientConfig;
use reqwest::{blocking::ClientBuilder, cookie::Jar, StatusCode, Url};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
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

    let watchlist_path =
        PathBuf::from_str(&config.watchlist_path.unwrap_or("./watchlist.txt".into()))
            .expect("invalid watchlist path");

    let file_content = match fs::read_to_string(&watchlist_path) {
        Ok(x) => x,
        Err(e) => {
            panic!(
                "cannot read watchlist file {}: {}",
                watchlist_path.to_string_lossy(),
                e
            );
        }
    };

    let watchlist = file_content.lines().collect::<Vec<_>>();

    let serialized = serde_json::to_vec(&watchlist).unwrap();

    let cookie_jar = Arc::new(Jar::default());

    let upload_url = format!("{}/admin/load_watchlist", config.endpoint_url);

    println!("upload url: {}", upload_url);

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

    match http_client.post(upload_url).body(serialized).send() {
        Ok(response) => match response.status() {
            StatusCode::OK => {
                let body = response.bytes().expect("failed fetching response body");
                let api_response: APIResponse<()> = match serde_json::from_reader(body.reader()) {
                    Ok(x) => x,
                    Err(e) => panic!("failed parsing response body {}", e),
                };

                if api_response.code == 200 {
                    println!("Success. Rock and stone!");
                } else {
                    panic!(
                        "Server returned {}: {}",
                        api_response.code, api_response.message
                    );
                }
            }
            other => {
                println!("unexpected status code from server: {}", other);
                println!("body: {:?}", response.text());
                panic!("cannot load watchlist");
            }
        },
        Err(e) => {
            panic!("failed sending request: {}", e);
        }
    };
}
