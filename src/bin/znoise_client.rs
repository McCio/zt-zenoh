use lazy_static::lazy_static;
use snow::{params::NoiseParams, Builder, HandshakeState};
use zenoh::query::ReplyKeyExpr;

static SECRET: &[u8; 32] = b"i don't care for fidget spinners";
static NOISE_DEF_PARAMS: &str = "Noise_IXpsk2_25519_ChaChaPoly_BLAKE2s";
lazy_static! {
    static ref PARAMS: NoiseParams = NOISE_DEF_PARAMS.parse().unwrap();
}

fn initialize_noise(params: NoiseParams, secret: &[u8]) -> HandshakeState {
    // Initialize our responder using a builder.
    let builder = Builder::new(params);
    let keypair = builder.generate_keypair().unwrap();
    let static_key = keypair.private;
    builder
        .local_private_key(&static_key)
        .psk(2, secret)
        .build_initiator()
        .unwrap()
}

#[cfg(all(feature = "noise", feature = "zenoh"))]
#[tokio::main]
async fn main() {
    let mut buf = vec![0u8; 65535];
    let mut noise = initialize_noise(PARAMS.clone(), SECRET);
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();

    // -> e
    let len = noise.write_message(&[], &mut buf).unwrap();
    let query = session
        .get(format!("secure_registration/{}", NOISE_DEF_PARAMS))
        .accept_replies(ReplyKeyExpr::Any)
        .payload(&buf[..len]).await.unwrap();

    let reply = query.recv().unwrap();
    let result = reply.into_result().unwrap();
    // <- e, ee, se, s, es
    noise.read_message(&*result.payload().to_bytes(), &mut buf).unwrap();

    let comm_keyexpr = result.key_expr();

    let mut noise = noise.into_transport_mode().unwrap();
    println!("session established...");

    // Get to the important business of sending secured data.
    for _ in 0..10 {
        let len = noise.write_message(b"HACK THE PLANET", &mut buf).unwrap();
        session.put(comm_keyexpr, &buf[..len]).await.unwrap();
    }
    println!("notified server of intent to hack planet.");
}

#[cfg(not(all(feature = "noise", feature="zenoh")))]
fn main() {
    panic!("Cannot start zenoh noise server without noise enabled.");
}
