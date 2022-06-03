use common::db::{DbRequest, DbResponse};
use sled::Db;

fn table_tree_name(owner: &str, table: &str) -> String {
    format!("{}:{}", owner, table)
}
fn table_tree(owner: &str, table: &str, db: &Db) -> anyhow::Result<sled::Tree> {
    let tree = db.open_tree(table_tree_name(owner, table))?;
    Ok(tree)
}
pub fn process_request(req: DbRequest, db: &Db) -> anyhow::Result<DbResponse> {
    match req {
        DbRequest::Del { owner, table, key } => {
            let table_tree = table_tree(&owner, &table, db)?;
            table_tree.remove(key)?;
            Ok(DbResponse::DeletedOk)
        }
        DbRequest::DelPrefixed {
            owner,
            table,
            prefix,
        } => {
            let table_tree = table_tree(&owner, &table, db)?;
            let prefix_iter = table_tree
                .scan_prefix(&prefix)
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
            let table_tree = table_tree(&owner, &table, db)?;
            let value = table_tree.get(key).unwrap().map(|value| value.to_vec());
            Ok(DbResponse::Value(value))
        }
        DbRequest::GetPrefixed {
            owner,
            table,
            prefix,
        } => {
            let table_tree = table_tree(&owner, &table, db)?;
            let values = table_tree
                .scan_prefix(prefix)
                .filter_map(|r| r.ok())
                .map(|(_key, val)| val.to_vec())
                .collect::<Vec<_>>();
            Ok(DbResponse::ManyValues(values))
        }
        DbRequest::Set {
            owner,
            table,
            key,
            value,
        } => {
            let table_tree = table_tree(&owner, &table, db)?;
            table_tree.insert(key, value).unwrap();
            Ok(DbResponse::SetOk)
        }
        DbRequest::SetMany {
            owner,
            table,
            pairs,
        } => {
            let table_tree = table_tree(&owner, &table, db)?;
            let mut batch = sled::Batch::default();
            for (key, value) in pairs.into_iter() {
                batch.insert(key.as_str(), value);
            }
            table_tree.apply_batch(batch).unwrap();
            Ok(DbResponse::SetManyOk)
        }
    }
}
