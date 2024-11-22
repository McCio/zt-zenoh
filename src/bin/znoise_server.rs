use keybased::utils::signals::{PerimeterStatus, WindowStatus};
use lazy_static::lazy_static;
use snow::{params::NoiseParams, Builder, HandshakeState};
use tokio::task::{JoinHandle, JoinSet};
use zenoh::Session;

static SECRET: &[u8; 32] = b"i don't care for fidget spinners";
static NOISE_DEF_PARAMS: &str = "Noise_IXpsk2_25519_ChaChaPoly_BLAKE2s";
lazy_static! {
    static ref PARAMS: NoiseParams = NOISE_DEF_PARAMS.parse().unwrap();
}

#[cfg(all(feature = "noise", feature = "zenoh"))]
#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let device = hear_for_device(session.clone()).await;
    // let windows = hear_for_windows(session.clone()).await;
    // let perimeters = hear_for_perimeters(session.clone()).await;
    device.await.unwrap();
    // windows.await.unwrap();
    // perimeters.await.unwrap();
    session.close().await.unwrap();
}

fn initialize_noise(params: NoiseParams, secret: &[u8]) -> HandshakeState {
    // Initialize our responder using a builder.
    let builder = Builder::new(params);
    let static_key = builder.generate_keypair().unwrap().private;
    builder
        .local_private_key(&static_key)
        .psk(2, secret)
        .build_responder()
        .unwrap()
}

async fn hear_for_device(session: Session) -> JoinHandle<()> {
    let subscriber = session.declare_queryable(format!("secure_registration/{}", NOISE_DEF_PARAMS)).await.unwrap();
    tokio::task::spawn(async move {
        let mut set = JoinSet::new();
        while let Ok(query) = subscriber.recv_async().await {
            let mut buf: Vec<u8>;
            let mut noise: HandshakeState;
            if let Some(payload) = query.payload() {
                noise = initialize_noise(PARAMS.clone(), SECRET);
                buf = vec![0u8; 65535];
                // <- e
                noise
                    .read_message(&*payload.to_bytes(), &mut buf)
                    .unwrap();
            } else {
                continue;
            }

            // -> e, ee, s, es
            let len = noise.write_message(&[], &mut buf).expect("failed to prepare handshake first response");
            let rsh256 = base16ct::lower::encode_string(noise.get_remote_static().expect("should know it by now"));
            let device_keyexpr = format!("secure_comm/{}", rsh256);

            let device_sub = session.clone().declare_subscriber(device_keyexpr.clone()).await.unwrap();
            set.spawn(async move {
                let mut buf = vec![0u8; 65535];
                let mut noise_transport;
                noise_transport = noise.into_transport_mode().unwrap();
                while let Ok(sample) = device_sub.recv_async().await {
                    let len = noise_transport.read_message(&*sample.payload().to_bytes(), &mut buf).unwrap();
                    println!("client {} said: {}", rsh256, String::from_utf8_lossy(&buf[..len]));
                }
                println!("client {} disconnected", rsh256);
                device_sub.undeclare().await.unwrap();
            });

            query.reply(device_keyexpr, &buf[..len]).await.expect("failed to send handshake first response");
        }
    })
}

async fn hear_for_windows(session: Session) -> JoinHandle<()> {
    let key = session.declare_keyexpr("window/*").await.unwrap();
    let subscriber = session.declare_subscriber(key).await.unwrap();
    tokio::task::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            println!(
                "Received on {}: {}",
                sample.key_expr(),
                match TryInto::<WindowStatus>::try_into(sample.payload()) {
                    Ok(status) => status.into(),
                    Err(_) => "<Invalid value>",
                }
            );
        }
    })
}

async fn hear_for_perimeters(session: Session) -> JoinHandle<()> {
    let key = session.declare_keyexpr("perimeter/*").await.unwrap();
    let subscriber = session.declare_subscriber(key).await.unwrap();
    tokio::task::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            println!(
                "Received on {}: {}",
                sample.key_expr(),
                match TryInto::<PerimeterStatus>::try_into(sample.payload()) {
                    Ok(x) => x.into(),
                    Err(_) => "<Invalid value>",
                }
            );
        }
    })
}

#[cfg(not(all(feature = "noise", feature="zenoh")))]
fn main() {
    panic!("Cannot start zenoh noise server without noise enabled.");
}
