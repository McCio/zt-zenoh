use clap::{arg, Command};
use lazy_static::lazy_static;
use snow::{params::NoiseParams, Builder, Error, HandshakeState};
use std::fs::read;
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::task::{JoinError, JoinHandle};
use zenoh::query::Query;
use zenoh::sample::{Sample, SampleKind};
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
) -> Result<HandshakeState, Error> {
    Builder::new(params)
        .local_private_key(static_key)
        .psk(2, secret)
        .build_responder()
}

async fn hear_for_device(
    session: Session,
    key_expr: String,
    prms: NoiseParams,
    pk: Vec<u8>,
) -> JoinHandle<()> {
    let subscriber = session.declare_queryable(key_expr).wait().unwrap();
    tokio::task::spawn_blocking(move || {
        let mut runtimes: Vec<tokio::runtime::Runtime> = Vec::new();
        // let mut set = JoinSet::new();
        while let Ok(query) = subscriber.recv() {
            if let Some(_) = query.payload() {
                let session = session.clone();
                let prms = prms.clone();
                let pk = pk.clone();

                let expr = query.key_expr().clone();
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    // .worker_threads(8)
                    .thread_name(expr.clone())
                    // .max_blocking_threads(4)
                    .build()
                    .unwrap();
                //rt.handle().clone()
                tokio::runtime::Handle::current().spawn(async {
                    println!("starting all correctly");
                    match device_handshake(session, prms, pk, query).await {
                        Ok(_) => println!("done all correctly"),
                        Err(e) => eprintln!("{:?}", e),
                    };
                    println!("done all correctly");
                });
                runtimes.push(rt);
            } else {
                match query.reply_err([]).wait() {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
        // tokio::runtime::Handle::current().block_on(set.join_all());
        for rt in runtimes {
            rt.shutdown_timeout(Duration::from_secs(1));
        }
    })
}

async fn device_handshake(
    session: Session,
    prms: NoiseParams,
    pk: Vec<u8>,
    query: Query,
) -> Result<(), JoinError> {
    println!("starting device handshake");
    let Some(payload) = query.payload() else {
        unreachable!()
    };
    let mut buf: Vec<u8> = vec![0u8; 65535];
    let mut noise: HandshakeState;
    match initialize_noise_server(prms, &*pk, SECRET) {
        Ok(n) => noise = n,
        Err(e) => {
            query.reply_err([]).await.unwrap();
            panic!("Cannot initialize noise server: {}", e);
        }
    }
    // <- e
    match noise.read_message(&*payload.to_bytes(), &mut buf) {
        Ok(_) => println!("received first message OK"),
        Err(e) => {
            query.reply_err([]).await.unwrap();
            panic!("Cannot resolve initial message: {}", e);
        }
    }

    // -> e, ee, s, es
    let len = noise
        .write_message(&[], &mut buf)
        .expect("failed to prepare handshake first response");
    println!("prepared first message to send");

    let mut noise_transport;
    noise_transport = noise.into_transport_mode().unwrap();
    let rsh256 = base16ct::lower::encode_string(
        noise_transport
            .get_remote_static()
            .expect("should know it by now"),
    );
    let device_keyexpr = format!("secure_comm/{}", rsh256);
    let remote_sha256 = rsh256.clone();

    let device_sub = session
        .declare_subscriber(device_keyexpr.clone())
        .await
        .unwrap();
    let ds_for_live = device_sub.clone();
    let (sched_writer, mut run_watch) = tokio::sync::mpsc::channel::<Sample>(1);
    let handle = tokio::runtime::Handle::current();
    let actual_consumer = handle.spawn_blocking(move || {
        while let Some(sample) = run_watch.blocking_recv() {
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
            println!("{} said: {}", rsh256, String::from_utf8_lossy(&buf[..len]));
        }
    });

    let abort_secure_comm = handle.spawn_blocking(move || {
        while let Ok(sample) = ds_for_live.recv() {
            sched_writer.blocking_send(sample).unwrap();
        }
        println!("{} disconnected", remote_sha256);
    });
    let life_subscriber = session
        .liveliness()
        .declare_subscriber(device_keyexpr.clone())
        .wait()
        .unwrap();
    let hear_for_liveliness = handle.spawn_blocking(move || {
        while let Ok(sample) = life_subscriber.recv() {
            match sample.kind() {
                SampleKind::Put => println!("{} is alive", sample.key_expr()),
                SampleKind::Delete => {
                    println!("{} is dead", sample.key_expr());
                    // we might lose some messages: it actually depends on the scheduler
                    device_sub.undeclare().wait().unwrap();
                    // maybe aborting the other thread is safer? don't like to kill
                    // abort_secure_comm.abort();
                    break;
                }
            }
        }
    });

    println!("sending final handshake message");
    query
        .reply(device_keyexpr, &buf[..len])
        .wait()
        .expect("failed to send handshake first response");
    println!("sending final handshake message sent");
    hear_for_liveliness.await
        .and(abort_secure_comm.await)
        .and(actual_consumer.await)
}
