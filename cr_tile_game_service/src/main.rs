use cr_tile_game_common::leader_board_stat::{LeaderBoardEntry, LeaderBoardList};
use cr_tile_game_common::packet::{ClientPacket, LoginInfo, ServerPacket};
use smol_db_client::{Client, DBPacketResponse};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

const DB_NAME: &str = "cr_tile_game_db";
const DB_KEY: &str = "cr_tile_game_service";

fn setup_client(client: &mut Client) {
    // set access key to client
    match client.set_access_key(DB_KEY.to_string()) {
        Ok(response) => match response {
            DBPacketResponse::SuccessNoData => {
                println!("Key set successfully.");
            }
            DBPacketResponse::SuccessReply(reply) => {
                panic!("This should not happen: {:?}", reply);
            }
            DBPacketResponse::Error(err) => {
                panic!("Error setting access key: {:?}", err);
            }
        },
        Err(err) => {
            panic!("Failed to set access key to smol_db: {:?}", err);
        }
    }

    // validate that the db is present in the list
    match client.list_db() {
        Ok(list) => {
            assert!(list.iter().any(|db_info| db_info.get_db_name() == DB_NAME));
        }
        Err(err) => {
            panic!("Failed to list db from db: {:?}", err);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8114").unwrap();

    let mut client = Client::new("localhost:8222").unwrap();

    setup_client(&mut client);

    let db_client = Arc::new(Mutex::new(client));
    let mut thread_vec = vec![];

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

fn handle_client(mut stream: TcpStream, client: Arc<Mutex<Client>>) {
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

                                    dbg!(packet);

                                    let content_opt = {
                                        match lock
                                            .list_db_contents_generic::<LeaderBoardEntry>(DB_NAME)
                                        {
                                            Ok(mut resp) => {
                                                match resp.get(db_location.as_str()) {
                                                    None => {}
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
                                                                        DBPacketResponse::Error(
                                                                            _,
                                                                        ) => {
                                                                            break;
                                                                        }
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
                                            Err(_) => {
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
