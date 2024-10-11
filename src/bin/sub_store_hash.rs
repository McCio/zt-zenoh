use samp::utils::signals::{PerimeterStatus, WindowStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::{AbortHandle, JoinSet};
use zenoh::key_expr::KeyExpr;
use zenoh::sample::SampleKind::Delete;
use zenoh::{Session, Wait};

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();
    let mut set = JoinSet::new();

    let window_store: HashMap<String, (WindowStatus, u64)> = HashMap::new();
    let window_store_lock = Arc::new(RwLock::new(window_store));
    hear_for_windows(
        session.clone(),
        "window".into(),
        Arc::clone(&window_store_lock),
        &mut set,
    )
    .await;

    let perimeter_store: HashMap<String, (PerimeterStatus, u64)> = HashMap::new();
    let perimeter_store_lock = Arc::new(RwLock::new(perimeter_store));
    hear_for_perimeters(
        session.clone(),
        "perimeter".into(),
        Arc::clone(&perimeter_store_lock),
        &mut set,
    )
    .await;

    answer_count_for(
        "window".into(),
        session.clone(),
        Arc::clone(&window_store_lock),
        &mut set,
    )
    .await;
    answer_count_for(
        "perimeter".into(),
        session.clone(),
        Arc::clone(&perimeter_store_lock),
        &mut set,
    )
    .await;

    while let Some(res) = set.join_next().await {
        res.unwrap(); // join error
    }
}

fn extract_key_name(key_expr: &KeyExpr<'static>, prefix: &String) -> String {
    key_expr.replace(prefix, "")
}

async fn make_key<'a>(session: &Session, base: &String) -> (String, KeyExpr<'a>) {
    let key_root = format!("{base}/");
    let key = session
        .declare_keyexpr(format!("{key_root}*"))
        .await
        .unwrap();
    (key_root, key)
}

async fn answer_count_for<_T: Send + Sync + 'static>(
    base: String,
    session: Session,
    window_store_lock: Arc<RwLock<HashMap<String, (_T, u64)>>>,
    set: &mut JoinSet<()>,
) -> AbortHandle {
    let key_root = format!("runtime_count/{base}/");
    let subscriber = session
        .declare_queryable(format!("{key_root}*"))
        .await
        .unwrap();
    set.spawn_blocking(move || {
        while let Ok(query) = subscriber.recv() {
            let name = extract_key_name(query.key_expr(), &key_root);
            let guarded_map = window_store_lock.blocking_read();
            let entry = guarded_map.get(&name);
            match entry {
                Some(value) => query.reply(query.key_expr(), &value.1),
                None => query.reply(query.key_expr(), 0),
            }
            .wait()
            .unwrap();
        }
    })
}

async fn hear_for_windows(
    session: Session,
    base: String,
    mem_store: Arc<RwLock<HashMap<String, (WindowStatus, u64)>>>,
    set: &mut JoinSet<()>,
) -> AbortHandle {
    let (key_root, key) = make_key(&session, &base).await;
    let subscriber = session.declare_subscriber(key).await.unwrap();
    set.spawn_blocking(move || {
        while let Ok(sample) = subscriber.recv() {
            let name = extract_key_name(sample.key_expr(), &key_root);
            if Delete == sample.kind() {
                let write_guard = Arc::clone(&mem_store);
                let previous_count = write_guard
                    .blocking_write()
                    .remove(&name)
                    .map(|val| val.1)
                    .unwrap_or(0);
                println!("stats for window {name} removed, previous count: {previous_count}");
                continue;
            }
            if let Ok(x) = sample.payload().deserialize::<WindowStatus>() {
                let write_guard = Arc::clone(&mem_store);
                let count = write_guard
                    .blocking_write()
                    .entry(name.clone())
                    .and_modify(|val| *val = (x.clone(), val.1 + 1))
                    .or_insert((x, 1))
                    .1;
                println!("stats for window {name}, count: {count}");
            } else {
                eprintln!("Received invalid value for {name} ({})", sample.key_expr());
                dbg!(sample.payload());
            }
        }
    })
}

async fn hear_for_perimeters(
    session: Session,
    base: String,
    mem_store: Arc<RwLock<HashMap<String, (PerimeterStatus, u64)>>>,
    set: &mut JoinSet<()>,
) -> AbortHandle {
    let (key_root, key) = make_key(&session, &base).await;
    let subscriber = session.declare_subscriber(key).await.unwrap();
    set.spawn_blocking(move || {
        while let Ok(sample) = subscriber.recv() {
            sample.kind();
            let name = extract_key_name(sample.key_expr(), &key_root);
            if Delete == sample.kind() {
                let write_guard = Arc::clone(&mem_store);
                let previous_count = write_guard
                    .blocking_write()
                    .remove(&name)
                    .map(|val| val.1)
                    .unwrap_or(0);
                println!("stats for perimeter {name} removed, previous count: {previous_count}");
                continue;
            }
            if let Ok(x) = sample.payload().deserialize::<PerimeterStatus>() {
                let write_guard = Arc::clone(&mem_store);
                let count = write_guard
                    .blocking_write()
                    .entry(name.clone())
                    .and_modify(|val| *val = (x.clone(), val.1 + 1))
                    .or_insert((x, 1))
                    .1;
                println!("stats for perimeter {name}, count: {count}");
            } else {
                eprintln!("Received invalid value for {name} ({})", sample.key_expr());
                dbg!(sample.payload());
            }
        }
    })
}
