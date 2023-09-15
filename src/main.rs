use dh_chat::{utils, net};

#[tokio::main]
async fn main() {
    println!("choose operation mode:\n\
             c  client mode\n\
             s  server mode");

    let ans = utils::get_inp().unwrap();

    if ans=="c" { net::client_mode().await; }
    else if ans=="s" { net::make_server().await; }
}
