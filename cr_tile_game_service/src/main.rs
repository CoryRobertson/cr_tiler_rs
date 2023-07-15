use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;
use std::thread::JoinHandle;
use crate::packet::{GameDataPacket, LoginInfo};

pub mod packet;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8114").unwrap();
    let mut thread_vec = vec![];

    for income in listener.incoming() {
        thread_vec.retain(|thread: &JoinHandle<()>| !thread.is_finished() );

        let handle = thread::spawn(move || {
            let stream = income.expect("Failed to receive tcp stream");
            handle_client(stream);
        });

        thread_vec.push(handle);
        println!(
            "New client connected, current number of connected clients: {}",
            thread_vec.len()
        );
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buf: [u8 ; 1024] = [ 0 ; 1024 ];
    let ip = stream
        .peer_addr()
        .expect("Unable to get peer address")
        .ip();

    let mut login_info: LoginInfo = LoginInfo::default();

    dbg!(ip.to_string());

    loop {
        match stream.read(&mut buf) {
            Ok(read_length) => {
                if read_length > 0 {
                    match serde_json::from_slice::<GameDataPacket>(&buf[0..read_length]) {
                        Ok(ref packet) => {
                            // db here
                            login_info = packet.login_info.clone();
                            dbg!(packet);

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

        let packet = GameDataPacket::default();
        let ser = serde_json::to_string(&packet).unwrap();
        match stream.write(ser.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err);
                break;
            }
        }
    }
}