use cr_tile_game_common::leader_board_stat::{LeaderBoardEntry, LeaderBoardList};
use cr_tile_game_common::packet::{ClientPacket, LoginInfo, ServerPacket};
use smol_db_client::client_error::ClientError;
use smol_db_client::db_settings::DBSettings;
use smol_db_client::DBPacketResponseError::DBAlreadyExists;
use smol_db_client::{DBSuccessResponse, SmolDbClient};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;

const DB_NAME: &str = "cr_tile_game_db";
const DB_KEY: &str = "cr_tile_game_service";

fn setup_client(client: &mut SmolDbClient) {
    // set access key to client
    match client.set_access_key(DB_KEY.to_string()) {
        Ok(response) => match response {
            DBSuccessResponse::SuccessNoData => {
                println!("Key set successfully.");
            }
            DBSuccessResponse::SuccessReply(reply) => {
                panic!("This should not happen: {:?}", reply);
            }
        },
        Err(err) => {
            panic!("Failed to set access key to smol_db: {:?}", err);
        }
    }

    // validate that the db is present in the list
    match client.list_db() {
        Ok(list) => {
            if !list.iter().any(|db_info| db_info.get_db_name() == DB_NAME) {
                match client.create_db(
                    DB_NAME,
                    DBSettings::new(
                        Duration::from_secs(30),
                        (false, false, false),
                        (false, false, false),
                        vec![DB_KEY.to_string()],
                        vec![],
                    ),
                ) {
                    Ok(_) => {
                        println!("DB Created...");
                    }
                    Err(err) => {
                        assert_eq!(err, ClientError::DBResponseError(DBAlreadyExists));
                    }
                }
            }
        }
        Err(err) => {
            panic!("Failed to list db from db: {:?}", err);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8114").unwrap();

    let mut client = {
        let mut count = 0;
        loop {
            if count >= 10 {
                panic!("Unable to connect to db after 10 retries");
            }
            let client_result = SmolDbClient::new("localhost:8222");
            match client_result {
                Ok(client) => {
                    break client;
                }
                Err(_) => {
                    let client_result_docker_attempt = SmolDbClient::new("db:8222");
                    if let Ok(client) = client_result_docker_attempt {
                        break client;
                    }
                    println!("Waiting one second for db connection and attempting to try again...");
                    count += 1;
                    sleep(Duration::from_secs(1));
                }
            }
        }
    };

    setup_client(&mut client);

    let db_client = Arc::new(Mutex::new(client));
    let mut thread_vec = vec![];

    println!("Listening for players on port 8114");

    for income in listener.incoming() {
        thread_vec.retain(|thread: &JoinHandle<()>| !thread.is_finished());

        let db_client_clone = db_client.clone();
        let handle = thread::spawn(move || {
            let stream = income.expect("Failed to receive tcp stream");
            handle_client(stream, db_client_clone);
        });

        thread_vec.push(handle);
        println!(
            "New client connected, current number of connected clients: {}",
            thread_vec.len()
        );
    }
}

fn handle_client(mut stream: TcpStream, client: Arc<Mutex<SmolDbClient>>) {
    let mut buf: [u8; 1024] = [0; 1024];
    let ip = stream.peer_addr().expect("Unable to get peer address").ip();

    let mut login_info: LoginInfo;

    dbg!(ip.to_string());

    loop {
        match stream.read(&mut buf) {
            Ok(read_length) => {
                if read_length > 0 {
                    match serde_json::from_slice::<ClientPacket>(&buf[0..read_length]) {
                        Ok(ref packet) => {
                            match packet {
                                ClientPacket::SubmitDataPacket(packet) => {
                                    // db here
                                    login_info = packet.login_info.clone();
                                    let db_location = login_info.hash().to_string();
                                    let mut lock = client.lock().unwrap();
                                    let discriminator = db_location[0..6].to_string();
                                    let entry = LeaderBoardEntry::new(
                                        login_info.user_name.clone(),
                                        packet.score,
                                        discriminator,
                                    );

                                    let content_opt = {
                                        match lock
                                            .list_db_contents_generic::<LeaderBoardEntry>(DB_NAME)
                                        {
                                            Ok(mut resp) => {
                                                match resp.get(db_location.as_str()) {
                                                    None => {
                                                        match lock.write_db_generic(
                                                            DB_NAME,
                                                            db_location.as_str(),
                                                            entry.clone(),
                                                        ) {
                                                            Ok(resp_write) => match resp_write {
                                                                _ => {
                                                                    resp.insert(
                                                                        db_location.to_string(),
                                                                        entry,
                                                                    );
                                                                }
                                                            },
                                                            Err(_) => {
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    Some(db_entry) => {
                                                        if entry.get_score() > db_entry.get_score()
                                                        {
                                                            match lock.write_db_generic(
                                                                DB_NAME,
                                                                db_location.as_str(),
                                                                entry.clone(),
                                                            ) {
                                                                Ok(resp_write) => {
                                                                    match resp_write {
                                                                        _ => {
                                                                            resp.insert(
                                                                                db_location
                                                                                    .to_string(),
                                                                                entry,
                                                                            );
                                                                        }
                                                                    }
                                                                }
                                                                Err(_) => {
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                let list = resp
                                                    .into_values()
                                                    .collect::<Vec<LeaderBoardEntry>>();
                                                Some(LeaderBoardList::new(list))
                                            }
                                            Err(err) => {
                                                eprintln!("{:?}", err);
                                                let _ = lock.delete_data(
                                                    DB_NAME,
                                                    login_info.hash().to_string().as_str(),
                                                );
                                                None
                                            }
                                        }
                                    };

                                    match content_opt {
                                        None => {
                                            let ser =
                                                serde_json::to_string(&ServerPacket::ErrorState)
                                                    .unwrap();
                                            match stream.write(ser.as_bytes()) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    println!("{}", err);
                                                    break;
                                                }
                                            }
                                        }
                                        Some(content) => {
                                            let ser = serde_json::to_string(
                                                &ServerPacket::LeaderBoard(content),
                                            )
                                            .unwrap();
                                            match stream.write(ser.as_bytes()) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    println!("{}", err);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                ClientPacket::GetLeaderBoardsList => {
                                    let mut lock = client.lock().unwrap();
                                    match lock.list_db_contents_generic::<LeaderBoardEntry>(DB_NAME)
                                    {
                                        Ok(resp) => {
                                            let list = resp
                                                .into_values()
                                                .collect::<Vec<LeaderBoardEntry>>();
                                            let ser =
                                                serde_json::to_string(&ServerPacket::LeaderBoard(
                                                    LeaderBoardList::new(list),
                                                ))
                                                .unwrap();
                                            match stream.write(ser.as_bytes()) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    println!("{}", err);
                                                    break;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            let ser =
                                                serde_json::to_string(&ServerPacket::ErrorState)
                                                    .unwrap();
                                            match stream.write(ser.as_bytes()) {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    println!("{}", err);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            println!("{}", err);
                        }
                    }
                }
            }
            Err(_) => {
                println!("Stream read failed.");
                break;
            }
        }
    }
}
