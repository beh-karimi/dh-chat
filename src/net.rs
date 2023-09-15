use std::{
    process::exit,
    net::{TcpListener, TcpStream},
    io::BufReader,
    io::prelude::*,
    sync::Arc,
};

use crate::{
    utils::{self, AppError},
    crypt::dh::{KeyPair, self},
    crypt::{self, decrypt, encrypt},
};

pub async fn make_server() {
    let key = match utils::read_key("server.key") {
        Ok(k) => k,
        Err(AppError::Io(_)) => {
            println!("No server key found, make one? (y/n)");
            loop {
                let ans = utils::get_inp().unwrap().to_lowercase();
                if ans=="y" || ans=="yes" { break; }
                else if ans=="n" || ans=="no" { exit(0); }
                else {
                    println!("Invalid answer.");
                }
            }
            let k = utils::customize_and_generate_key();
            utils::save_key("server.key", &k).unwrap();
            k
        },
        Err(AppError::Parse(_)) => panic!(),
    };
    
    let listener = TcpListener::bind("0.0.0.0:25565").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_server_connection(stream, &key).await;
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
    println!("shared key established.");

    let skey_ref = Arc::new(shared_key);

    let mut stream_cp = stream.try_clone().unwrap();
    let key_c = Arc::clone(&skey_ref);
    tokio::spawn(async move {
        receive_msg(&mut stream, &key_c).await;
    });
    send_msg(&mut stream_cp, &skey_ref).await;
}

pub async fn client_mode() {
    println!("address to connect to (ip:port)");
    let addr = utils::get_inp().unwrap();
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
    println!("got the shared key");

    let skey_ref = Arc::new(shared_key);

    let mut stream_cp = stream.try_clone().unwrap();
    let key_c = Arc::clone(&skey_ref);
    tokio::spawn(async move {
        receive_msg(&mut stream, &key_c).await;
    });
    send_msg(&mut stream_cp, &skey_ref).await;


}

async fn receive_msg(stream: &mut TcpStream, shared_key: &[u8]) {
    loop {
        let mut inc_msg_len: u64 = 0;
        let mut len_buf = [0u8];
        loop {
            stream.read_exact(&mut len_buf).unwrap();
            inc_msg_len += len_buf[0] as u64;
            if len_buf[0]!=255 {break;}
        }

        let mut inc_msg = vec![0u8; inc_msg_len as usize];
        stream.read_exact(&mut inc_msg).unwrap();
        let inc_msg = decrypt(shared_key, &inc_msg);
        println!("{}", inc_msg.trim());
    }
}

async fn send_msg(stream: &mut TcpStream, shared_key: &[u8]) {
    loop {
        let inp = utils::get_inp().unwrap();
        let mut inp_len = inp.len();
        let mut msg = vec![];
        while inp_len >= 255 {
            inp_len -= 255;
            msg.push(255u8);
        }
        msg.push(inp_len as u8);
        let mut enc = encrypt(shared_key, &inp);
        msg.append(&mut enc);
        stream.write_all(&msg).unwrap();
    }
}
