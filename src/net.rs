use std::{
    process::exit,
    net::{TcpListener, TcpStream},
    io::BufReader,
    io::prelude::*,
    sync::Arc, 
};
use tokio::{
    select,
    sync:: watch,
};
use crate::{
    utils::{self, AppError, get_inp},
    crypt::dh::{KeyPair, self},
    crypt::{self, decrypt, encrypt},
};

pub async fn make_server() {
    let key = read_server_keys().await;
    let listener = create_server().await;
    println!("Waiting for someone to connect...");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(_) => {
                println!("Connection failed.");
                panic!();
            }
        };
        handle_server_connection(stream, &key).await;
        println!("Client disconnected.\nWaiting for someone else to connect...\n");
    }
}

pub async fn client_mode() {
    println!("Server Address (ip:port):");
    loop {
        let addr = get_inp().await;
        let mut stream = TcpStream::connect(addr).unwrap();

        let mut key_details = [0u64;3];
        let buf = BufReader::new(&mut stream);
        for (i,line) in buf.lines().take(3).enumerate() {
            key_details[i] = line.unwrap().trim().parse().unwrap();
        }

        let key = crypt::dh::gen_key_pair(key_details[1], key_details[2]);
        let key_str = key.public.to_string()+"\n";
        stream.write_all(key_str.as_bytes()).unwrap();
        let shared_key = crypt::dh::gen_common_key(key.modulus, key.private, key_details[0]);
        println!("Shared key established.");

        communicate(stream, shared_key).await;
        println!("Disconnected from the server. Enter a new one (ctrl+c to exit):");
    }
}

async fn create_server() -> TcpListener {
    println!("What port should the server be on?");
    loop {
        let port = get_inp().await;
        match port.trim().parse::<u16>() {
            Ok(i) => i,
            Err(_) => {
                println!("Please Enter a valid port");
                continue;
            }
        };

        match TcpListener::bind("0.0.0.0:".to_string()+&port) {
            Ok(l) => return l,
            Err(_) => {
                println!("Problen while binding to port {}, maybe another process is using it.\n", port);
                println!("What port should the server be on?");
                continue;
            }
        };
    }
}

async fn read_server_keys() -> KeyPair {
    match utils::read_key("server.key") {
        Ok(k) => k,
        Err(AppError::Io(_)) => {
            println!("No server key found, make one? (y/n)");
            loop {
                let ans = get_inp().await.to_lowercase();
                if ans=="y" || ans=="yes" { break; }
                else if ans=="n" || ans=="no" { exit(0); }
                else {
                    println!("Invalid answer.");
                }
            }
            let k = utils::customize_and_generate_key().await;
            utils::save_key("server.key", &k).unwrap();
            k
        },
        Err(AppError::Parse(_)) => panic!(),
    }
}

async fn handle_server_connection(mut stream: TcpStream, key: &KeyPair) {
    let pubkey_string = format!("{}\n{}\n{}\n", key.public, key.modulus, key.gen);
    stream.write_all(pubkey_string.as_bytes()).unwrap();

    let mut buf = BufReader::new(&mut stream);
    let mut cl_pubkey = String::new();
    buf.read_line(&mut cl_pubkey).unwrap();
    drop(buf);
    let cl_pubkey: u64 = cl_pubkey.trim().parse().unwrap();
    let shared_key = dh::gen_common_key(key.modulus, key.private, cl_pubkey);
    println!("Shared key established.");
    communicate(stream, shared_key).await;
}

async fn communicate(mut stream: TcpStream, shared_key: Vec<u8>) {
    let skey_ref = Arc::new(shared_key);

    let mut stream_cp = stream.try_clone().unwrap();
    let key_c = Arc::clone(&skey_ref);

    let (cancel_tx, cancel_rx) = watch::channel(false);
    tokio::spawn(async move {
        send_msg(&mut stream_cp, &skey_ref, cancel_rx).await;
    });
    receive_msg(&mut stream, &key_c).await;
    cancel_tx.send(true).unwrap();
}

async fn receive_msg(stream: &mut TcpStream, shared_key: &[u8]) {
    'm: loop {
        let mut inc_msg_len: u64 = 0;
        let mut len_buf = [0u8];
        loop {
            if let Err(_) = stream.read_exact(&mut len_buf) { break 'm; }
            inc_msg_len += len_buf[0] as u64;
            if len_buf[0]!=255 {break;}
        }

        let mut inc_msg = vec![0u8; inc_msg_len as usize];
        if let Err(_) = stream.read_exact(&mut inc_msg) { panic!("msg_rec"); };
        let inc_msg = decrypt(shared_key, &inc_msg);
        println!("{}", inc_msg.trim());
    }
}

async fn send_msg(stream: &mut TcpStream, shared_key: &[u8], mut cancel_rx: watch::Receiver<bool>) {
    loop {
        let inp: String;
        select! {
            s = get_inp() => { inp = s; }
            _ = cancel_rx.changed() => { break; }
        }
        let mut inp_len = inp.len();
        let mut msg = vec![];
        while inp_len >= 255 {
            inp_len -= 255;
            msg.push(255u8);
        }
        msg.push(inp_len as u8);
        let mut enc = encrypt(shared_key, &inp);
        msg.append(&mut enc);
        if let Err(_) = stream.write_all(&msg) { break; }
    }
}
