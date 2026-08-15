use std::{fs::{self, File, read}, path::PathBuf, process::exit, sync::Arc, time::Duration};

use directories::UserDirs;
use clap::Parser;
use reqwest::{StatusCode, blocking::Client};
use reqwest_cookie_store::CookieStoreMutex;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: PathBuf
}
fn main() {

    let args = Args::parse();
    let dirs = directories::ProjectDirs::from("net", "voxany", "up").unwrap();

    let data_dir = dirs.data_dir();
    let cookie_path = data_dir.join("cookies.json");

    fs::create_dir_all(&data_dir).unwrap();

    println!("{:?}", &cookie_path);

    let mut need_to_login = false;

    // what the FUCK is this sytax
    let cookie_store = {
        if let Ok(file) = std::fs::File::open(&cookie_path)
            .map(std::io::BufReader::new) {
                cookie_store::serde::json::load(file).unwrap()
            }
        else {

            need_to_login = true;
            reqwest_cookie_store::CookieStore::new()

        
        }
    };

    let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);
        
    let client = reqwest::blocking::Client::builder()
        .cookie_provider(cookie_store.clone())
        .build().unwrap();

    if need_to_login {
        login(&client, &cookie_store, &cookie_path);
    }
    

    let multipart = reqwest::blocking::multipart::Form::new().file("file", args.file).unwrap();

    let resp = client.post("https://upload.voxany.net")
        .multipart(multipart)
        .send()
        .unwrap();

    println!("{:?}", resp.text().unwrap());
}

fn login(client: &Client, cookie_store: &Arc<CookieStoreMutex>, cookie_path: &PathBuf) {
    let login_url = client.get("https://upload.voxany.net/request_cli_login_url")
        .send()
        .unwrap()
        .text()
        .unwrap();

    println!("{}", login_url);

    let token = login_url
        .split("?token=")
        .collect::<Vec<&str>>()
        .get(1)
        .unwrap()
        .clone();


    loop {
        let claimed_resp = client
            .get(format!("https://upload.voxany.net/cli_login_claim?token={}", token))
            .send()
            .unwrap();
            
        
        if claimed_resp.status() == StatusCode::OK {
            println!("Logged in!");

            let store = cookie_store.lock().unwrap();

            println!("{}", claimed_resp.text().unwrap());

    
            let mut file = File::options().write(true).create(true).open(cookie_path).unwrap();
            cookie_store::serde::json::save(&store, &mut file).unwrap();
            break;
            
        } else {
            std::thread::sleep(Duration::from_secs_f32(0.5));
            continue;
        }

        
    }

    
}


