// use fds_toolbox_core::file::ParsedFile;

use moka::future::Cache;

#[tokio::main]
async fn main() {
    // let hello = ParsedFile::new(parsed, path);
    let cache = Cache::new(100);

    cache.insert(5, "Past moment").await;

    dbg!(cache.get_with(5, async { "Future moment" }).await);

    dbg!(cache.get_with(6, async { "Future moment" }).await);

    let cc = cache.clone();
    let f = async move { cc.get_with(7, async { "Future moment" }).await };
    tokio::spawn(f);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    dbg!(cache.get(&7));
}
