use common::db::*;
use common::prelude::*;
use sled::Db;
use tokio::net::UnixListener;
const DB_SOCKET_PATH: &str = "/tmp/zhur-db.sck";
const DB_FILE_PATH: &str = "/tmp/zhur-db.sled";
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init().unwrap();
    let database = sled::open(DB_FILE_PATH)?;
    let listener = UnixListener::bind(DB_SOCKET_PATH)?;
    while let Ok((conn, _)) = listener.accept().await {
        tokio::spawn(async move {
            let mut server = ipc::UnixServer::new(1024 * 16, conn);
            let request = server.get_request::<DbRequest>().await.unwrap();
        });
    }
    Ok(())
}
fn table_tree_name(owner: &str, table: &str) -> String {
    format!("{}:{}", owner, table)
}
fn table_tree(owner: &str, table: &str, db: &Db) -> anyhow::Result<sled::Tree> {
    let tree = db.open_tree(table_tree_name(owner, table))?;
    Ok(tree)
}
fn process_request(req: &DbRequest, db: &Db) -> anyhow::Result<DbResponse> {
    match req {
        DbRequest::Del { owner, table, key } => {
            let table_tree = table_tree(owner, table, db)?;
            table_tree.remove(key)?;
            Ok(DbResponse::DeletedOk)
        }
        DbRequest::DelPrefixed {
            owner,
            table,
            prefix,
        } => {
            let table_tree = table_tree(owner, table, db)?;
            let prefix_iter = table_tree
                .scan_prefix(prefix)
                .filter_map(|r| r.ok())
                .map(|tuple| tuple.0)
                .filter(|key| key.as_ref().starts_with(prefix.as_bytes()))
                .collect::<Vec<_>>();
            let counter = table_tree
                .transaction::<_, _, ()>(|db| {
                    let mut counter = 0u64;
                    for key in &prefix_iter {
                        db.remove(key).unwrap();
                        counter += 1;
                    }
                    Ok(counter)
                })
                .unwrap();
            Ok(DbResponse::DeletedManyOk(counter))
        }
        DbRequest::Get { owner, table, key } => {
            let table_tree = table_tree(owner, table, db)?;
            let value = table_tree.get(key).unwrap().map(|value| value.to_vec());
            Ok(DbResponse::Value(value))
        }
        DbRequest::GetPrefixed {
            owner,
            table,
            prefix,
        } => {
            let table_tree = table_tree(owner, table, db)?;
            let values = table_tree
                .scan_prefix(prefix)
                .filter_map(|r| r.ok())
                .map(|(_key, val)| val.to_vec())
                .collect::<Vec<_>>();
            Ok(DbResponse::ManyValues(values))
        }
        _ => todo!(),
    }
}
