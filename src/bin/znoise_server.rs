use clap::{arg, Command};
use lazy_static::lazy_static;
use snow::{params::NoiseParams, Builder, HandshakeState};
use std::fs::read;
use std::path::PathBuf;
use tokio::task::{JoinHandle, JoinSet};
use zenoh::query::Query;
use zenoh::sample::SampleKind;
use zenoh::{Session, Wait};

static SECRET: &[u8; 32] = b"i don't care for fidget spinners";
static NOISE_FOR_NEWBY_CLIENT: &str = "Noise_IXpsk2_25519_ChaChaPoly_BLAKE2s";
static NOISE_FOR_PRO_CLIENT: &str = "Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s";
lazy_static! {
    static ref PARAMS_NEWBY: NoiseParams = NOISE_FOR_NEWBY_CLIENT.parse().unwrap();
    static ref PARAMS_PRO: NoiseParams = NOISE_FOR_PRO_CLIENT.parse().unwrap();
}

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let matches = cli().get_matches();
    let builder = Builder::new(PARAMS_NEWBY.clone());
    let pk_preexists = matches
        .get_one::<PathBuf>("private_key")
        .is_some_and(|path| path.is_file() && read(path).is_ok());
    let pk = samp::prepare_private_key(matches.get_one("private_key"), &builder);
    let device = hear_for_device(
        session.clone(),
        "secure_registration".to_string(),
        PARAMS_NEWBY.clone(),
        pk.clone(),
    )
    .await;
    if pk_preexists {
        let device2 = hear_for_device(
            session.clone(),
            format!("secure_registration/{}", NOISE_FOR_PRO_CLIENT),
            PARAMS_PRO.clone(),
            pk,
        )
        .await;
        device2.await.unwrap();
    }
    // let windows = hear_for_windows(session.clone()).await;
    // let perimeters = hear_for_perimeters(session.clone()).await;
    device.await.unwrap();
    // windows.await.unwrap();
    // perimeters.await.unwrap();
    session.close().await.unwrap();
}

fn cli() -> Command {
    Command::new("server")
        .about("Zenoh-communicating device receiving telemetries from a client")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .allow_external_subcommands(false)
        // TODO
        //  make a ValueParser https://docs.rs/clap/latest/clap/builder/trait.ValueParserFactory.html
        //  for rustls-pemfile
        .arg(arg!(server_mode: --server "Only for backward compat").num_args(0))
        .arg(arg!(private_key: --"key-file" <FILE> "If provided but not found, will write the private key here").num_args(1).value_parser(clap::value_parser!(PathBuf)))
}
fn initialize_noise_server(
    params: NoiseParams,
    static_key: &[u8],
    secret: &[u8],
) -> HandshakeState {
    Builder::new(params)
        .local_private_key(static_key)
        .psk(2, secret)
        .build_responder()
        .unwrap()
}

async fn hear_for_device(
    session: Session,
    key_expr: String,
    prms: NoiseParams,
    pk: Vec<u8>,
) -> JoinHandle<()> {
    let subscriber = session.declare_queryable(key_expr).wait().unwrap();
    tokio::task::spawn(async move {
        let mut set = JoinSet::new();
        while let Ok(query) = subscriber.recv() {
            if let Some(_) = query.payload() {
                let session = session.clone();
                let prms = prms.clone();
                let pk = pk.clone();
                set.spawn_blocking(move || device_handshake(session, prms, pk, query));
            }
        }
    })
}

fn device_handshake(
    session: Session,
    prms: NoiseParams,
    pk: Vec<u8>,
    query: Query,
) -> JoinHandle<()> {
    let Some(payload) = query.payload() else { unreachable!() };
    let mut buf: Vec<u8>;
    let mut noise: HandshakeState;
    noise = initialize_noise_server(prms, &*pk, SECRET);
    buf = vec![0u8; 65535];
    // <- e
    noise.read_message(&*payload.to_bytes(), &mut buf).unwrap();

    // -> e, ee, s, es
    let len = noise
        .write_message(&[], &mut buf)
        .expect("failed to prepare handshake first response");
    let rsh256 =
        base16ct::lower::encode_string(noise.get_remote_static().expect("should know it by now"));
    let device_keyexpr = format!("secure_comm/{}", rsh256);

    // TODO check z_liveliness, z_sub_liveliness and z_get_liveliness examples
    //  to avoid multiple listeners when a client disconnected
    let device_sub = session
        .declare_subscriber(device_keyexpr.clone())
        .wait()
        .unwrap();
    let ds_for_live = device_sub.clone();
    let abort_secure_comm = tokio::runtime::Handle::current().spawn_blocking(move || {
        let mut noise_transport;
        noise_transport = noise.into_transport_mode().unwrap();
        while let Ok(sample) = ds_for_live.recv() {
            let mut buf = vec![0u8; 65535];
            let decrypt = noise_transport.read_message(&*sample.payload().to_bytes(), &mut buf);
            if decrypt.is_err() {
                // this is split to avoid DoS
                eprintln!(
                    "received invalid payload on reserved channel for {}",
                    rsh256
                );
                continue;
            }
            let len = decrypt.unwrap();
            println!(
                "client {} said: {}",
                rsh256,
                String::from_utf8_lossy(&buf[..len])
            );
        }
        println!("client {} disconnected", rsh256);
    });
    let live_ke = device_keyexpr.clone();
    let hear_for_liveliness = tokio::runtime::Handle::current().spawn_blocking(move || {
        let subscriber = session
            .liveliness()
            .declare_subscriber(live_ke.clone())
            .wait()
            .unwrap();
        while let Ok(sample) = subscriber.recv() {
            match sample.kind() {
                SampleKind::Put => println!("New liveliness: {}", sample.key_expr()),
                SampleKind::Delete => {
                    println!("Lost liveliness: {}", sample.key_expr());
                    // this actually depends on the scheduler: we might lose some messages
                    abort_secure_comm.abort();
                    device_sub.undeclare().wait().unwrap();
                    return;
                }
            }
        }
    });

    query
        .reply(device_keyexpr, &buf[..len])
        .wait()
        .expect("failed to send handshake first response");
    hear_for_liveliness
}
