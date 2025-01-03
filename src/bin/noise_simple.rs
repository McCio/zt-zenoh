#![cfg_attr(
    not(feature = "noise"),
    allow(dead_code, unused_extern_crates, unused_imports)
)]

use lazy_static::lazy_static;
#[cfg(feature = "noise")]
use snow::{params::NoiseParams, Builder};
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};
use std::process::exit;

static SECRET: &[u8; 32] = b"i don't care for fidget spinners";
#[cfg(feature = "noise")]
lazy_static! {
    static ref PARAMS: NoiseParams = "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s".parse().unwrap();
}

#[cfg(feature = "noise")]
fn main() {
    let server_mode = std::env::args()
        .next_back()
        .map_or(false, |arg| arg == "-s" || arg == "--server" || arg == "server");
    let client_mode = std::env::args()
        .next_back()
        .map_or(false, |arg| arg == "-c" || arg == "--client" || arg == "client");

    if server_mode {
        run_server();
    } else if client_mode {
        run_client();
    } else {
        eprintln!("Usage: noise-simple [server|client]");
        exit(1);
    }
    println!("all done.");
}

#[cfg(feature = "noise")]
fn run_server() {
    let mut buf = vec![0u8; 65535];

    // Initialize our responder using a builder.
    let builder = Builder::new(PARAMS.clone());
    let static_key = builder.generate_keypair().unwrap().private;
    let mut noise = builder
        .local_private_key(&static_key)
        .psk(3, SECRET)
        .build_responder()
        .unwrap();

    // Wait on our client's arrival...
    println!("listening on 127.0.0.1:9999");
    let (mut stream, _) = TcpListener::bind("127.0.0.1:9999")
        .unwrap()
        .accept()
        .unwrap();

    // <- e
    noise
        .read_message(&recv(&mut stream).unwrap(), &mut buf)
        .unwrap();

    // -> e, ee, s, es
    let len = noise.write_message(&[], &mut buf).unwrap();
    send(&mut stream, &buf[..len]);

    // <- s, se
    noise
        .read_message(&recv(&mut stream).unwrap(), &mut buf)
        .unwrap();

    // Transition the state machine into transport mode now that the handshake is complete.
    let mut noise = noise.into_transport_mode().unwrap();

    while let Ok(msg) = recv(&mut stream) {
        let len = noise.read_message(&msg, &mut buf).unwrap();
        println!("client said: {}", String::from_utf8_lossy(&buf[..len]));
    }
    println!("connection closed.");
}

#[cfg(feature = "noise")]
fn run_client() {
    let mut buf = vec![0u8; 65535];

    // Initialize our initiator using a builder.
    let builder = Builder::new(PARAMS.clone());
    let static_key = builder.generate_keypair().unwrap().private;
    let mut noise = builder
        .local_private_key(&static_key)
        .psk(3, SECRET)
        .build_initiator()
        .unwrap();

    // Connect to our server, which is hopefully listening.
    let mut stream = TcpStream::connect("127.0.0.1:9999").unwrap();
    println!("connected...");

    // -> e
    let len = noise.write_message(&[], &mut buf).unwrap();
    send(&mut stream, &buf[..len]);

    // <- e, ee, s, es
    noise
        .read_message(&recv(&mut stream).unwrap(), &mut buf)
        .unwrap();

    // -> s, se
    let len = noise.write_message(&[], &mut buf).unwrap();
    send(&mut stream, &buf[..len]);

    let mut noise = noise.into_transport_mode().unwrap();
    println!("session established...");

    // Get to the important business of sending secured data.
    for _ in 0..10 {
        let len = noise.write_message(b"HACK THE PLANET", &mut buf).unwrap();
        send(&mut stream, &buf[..len]);
    }
    println!("notified server of intent to hack planet.");
}

/// Hyper-basic stream transport receiver. 16-bit BE size followed by payload.
fn recv(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut msg_len_buf = [0u8; 2];
    stream.read_exact(&mut msg_len_buf)?;
    let msg_len = usize::from(u16::from_be_bytes(msg_len_buf));
    let mut msg = vec![0u8; msg_len];
    stream.read_exact(&mut msg[..])?;
    Ok(msg)
}

/// Hyper-basic stream transport sender. 16-bit BE size followed by payload.
fn send(stream: &mut TcpStream, buf: &[u8]) {
    let len = u16::try_from(buf.len()).expect("message too large");
    stream.write_all(&len.to_be_bytes()).unwrap();
    stream.write_all(buf).unwrap();
}

#[cfg(not(feature = "noise"))]
fn main() {
    panic!("Example must be compiled with some cryptographic provider.");
}
