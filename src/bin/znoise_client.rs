use lazy_static::lazy_static;
use snow::{params::NoiseParams, Builder, HandshakeState};
use std::fs::{read, write};
use std::path::PathBuf;
use zenoh::query::{Reply, ReplyKeyExpr};

static SECRET: &[u8; 32] = b"i don't care for fidget spinners";
static NOISE_DEF_PARAMS: &str = "Noise_IXpsk2_25519_ChaChaPoly_BLAKE2s";
static NOISE_DEF_PARAMS_SERVERPUB_KNOWN: &str = "Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s";
lazy_static! {
    static ref PARAMS: NoiseParams = NOISE_DEF_PARAMS.parse().unwrap();
    static ref PARAMS_SERVERPUB_KNOWN: NoiseParams =
        NOISE_DEF_PARAMS_SERVERPUB_KNOWN.parse().unwrap();
}

use clap::{arg, Command};
use zenoh::handlers::FifoChannelHandler;
use zenoh::Session;

fn cli() -> Command {
    Command::new("client")
        .about("Zenoh-communicating device pushing telemetries to a server")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .allow_external_subcommands(false)
        // TODO
        //  make a ValueParser https://docs.rs/clap/latest/clap/builder/trait.ValueParserFactory.html
        //  for rustls-pemfile
        .arg(arg!(client_mode: --client "Only for backward compat").num_args(0))
        .arg(arg!(private_key: --"key-file" <FILE> "If provided but not found, will write the private key here").num_args(1).value_parser(clap::value_parser!(PathBuf)))
        .arg(arg!(remote_public_key: --"remote-public-key-file" <FILE> "If provided but not found, will write the public key here").num_args(1).value_parser(clap::value_parser!(PathBuf)))
}

fn initialize_noise_client(
    params: NoiseParams,
    private_key: Option<&PathBuf>,
    remote_public_key: Option<&PathBuf>,
    secret: &[u8],
) -> HandshakeState {
    // if provided, load remote public key
    let rpk = remote_public_key
        .filter(|path| path.exists())
        .and_then(|path| {
            read(path)
                .and_then(|k| Ok(Some(k)))
                .unwrap_or_else(|_| None)
        });
    // Initialize our responder using a builder.
    // let mut builder = if rpk.is_none() { Builder::new(params) } else { Builder::new(PARAMS_SERVERPUB_KNOWN.clone()) };
    let mut builder = Builder::new(params);
    let static_key = samp::prepare_private_key(private_key, &builder);
    builder = builder.local_private_key(&static_key).psk(2, secret);
    match rpk {
        Some(key) => builder.remote_public_key(&key).build_initiator(),
        _ => builder.build_initiator(),
    }
    .unwrap()
}

#[cfg(all(feature = "noise", feature = "zenoh"))]
#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    let private_key: Option<&PathBuf> = matches.get_one("private_key");
    let remote_public_key: Option<&PathBuf> = matches.get_one("remote_public_key");
    let rpk = remote_public_key
        .filter(|path| path.exists())
        .and_then(|path| {
            read(path)
                .and_then(|k| Ok(Some(k)))
                .unwrap_or_else(|_| None)
        });
    let mut noise = initialize_noise_client(
        if rpk.is_some() {
            PARAMS_SERVERPUB_KNOWN.clone()
        } else {
            PARAMS.clone()
        },
        private_key,
        remote_public_key,
        SECRET,
    );

    let mut buf = vec![0u8; 65535];
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();

    // -> e
    let len = noise.write_message(&[], &mut buf).unwrap();
    let reply = loop {
        let reply = register(&rpk, &buf, &session, len).await.recv();
        match reply {
            Ok(reply) => break reply,
            Err(e) => eprintln!("Still nothing: {:?}", e),
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };
    let result = reply.into_result().unwrap();
    // <- e, ee, se, s, es
    noise
        .read_message(&*result.payload().to_bytes(), &mut buf)
        .unwrap();

    let comm_keyexpr = result.key_expr();
    // TODO should be able to to this in advance, as this is just secure_comm/{public_key}
    let _ = session
        .liveliness()
        .declare_token(comm_keyexpr)
        .await
        .unwrap();

    let mut noise = noise.into_transport_mode().unwrap();
    println!("session established...");
    if let Some(path) = remote_public_key {
        if !path.exists() {
            let _ = write(path, noise.get_remote_static().unwrap()).is_ok();
        } else {
            assert!(
                read(path).unwrap().eq(noise.get_remote_static().unwrap()),
                "talking to the wrong server"
            );
        }
    }

    // Get to the important business of sending secured data.
    for _ in 0..10 {
        let len = noise.write_message(b"HACK THE PLANET", &mut buf).unwrap();
        session.put(comm_keyexpr, &buf[..len]).await.unwrap();
    }
    println!("notified server of intent to hack planet.");
}

async fn register(rpk: &Option<Vec<u8>>, buf: &Vec<u8>, session: &Session, len: usize) -> FifoChannelHandler<Reply> {
    session
        .get(if rpk.is_some() {
            format!("secure_registration/{}", NOISE_DEF_PARAMS_SERVERPUB_KNOWN)
        } else {
            "secure_registration".to_string()
        })
        // .get(format!("secure_registration/{}", NOISE_DEF_PARAMS))
        .accept_replies(ReplyKeyExpr::Any)
        .payload(&buf[..len])
        .await
        .unwrap()
}

#[cfg(not(all(feature = "noise", feature = "zenoh")))]
fn main() {
    panic!("Cannot start zenoh noise server without noise enabled.");
}
