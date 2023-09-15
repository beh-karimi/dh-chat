use dh_chat::{utils, net};

fn main() {
    println!("choose operation mode:\n\
             c  client mode\n\
             s  server mode");

    let ans = utils::get_inp().unwrap();

    if ans=="c" { net::client_mode(); }
    else if ans=="s" { net::make_server(); }
}
