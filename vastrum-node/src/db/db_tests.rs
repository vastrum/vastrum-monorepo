use super::*;

fn test_db(name: &str) -> Arc<Db> {
    let path = std::env::temp_dir().join(format!("vastrum_batch_test_{name}"));
    Arc::new(Db::open_fresh(path))
}

#[test]
fn read_own_write() {
    let db = test_db("read_own_write");
    let batch = BatchDb::new(db);
    batch.put(cf::SITE_KV, b"key1", b"value1".to_vec());
    assert_eq!(batch.get(cf::SITE_KV, b"key1").unwrap(), b"value1");
}

#[test]
fn read_own_write_overwrite() {
    let db = test_db("read_own_write_overwrite");
    let batch = BatchDb::new(db);
    batch.put(cf::SITE_KV, b"k", b"old".to_vec());
    batch.put(cf::SITE_KV, b"k", b"new".to_vec());
    assert_eq!(batch.get(cf::SITE_KV, b"k").unwrap(), b"new");
}

#[test]
fn uncommitted_not_visible_to_db() {
    let db = test_db("uncommitted_not_visible");
    let batch = BatchDb::new(Arc::clone(&db));
    batch.put(cf::SITE_KV, b"pending", b"data".to_vec());
    assert!(db.get(cf::SITE_KV, b"pending").is_none());
}

#[test]
fn commit_persists_to_db() {
    let db = test_db("commit_persists");
    let batch = BatchDb::new(Arc::clone(&db));
    batch.put(cf::SITE_KV, b"committed", b"yes".to_vec());
    batch.commit();
    assert_eq!(db.get(cf::SITE_KV, b"committed").unwrap(), b"yes");
}

#[test]
fn get_falls_through_to_db() {
    let db = test_db("falls_through");
    db.put(cf::SITE_KV, b"existing", b"fromdb".to_vec());
    let batch = BatchDb::new(Arc::clone(&db));
    assert_eq!(batch.get(cf::SITE_KV, b"existing").unwrap(), b"fromdb");
}

#[test]
fn rollback_removes_writes_after_begin() {
    let db = test_db("rollback_removes");
    let batch = BatchDb::new(db);
    batch.put(cf::SITE_KV, b"before", b"stays".to_vec());
    batch.begin_revertable();
    batch.put(cf::SITE_KV, b"after", b"gone".to_vec());
    batch.rollback_revertable();
    assert_eq!(batch.get(cf::SITE_KV, b"before").unwrap(), b"stays");
    assert!(batch.get(cf::SITE_KV, b"after").is_none());
}

#[test]
fn rollback_restores_overwritten_values() {
    let db = test_db("rollback_restores");
    let batch = BatchDb::new(db);
    batch.put(cf::SITE_KV, b"key", b"original".to_vec());
    batch.begin_revertable();
    batch.put(cf::SITE_KV, b"key", b"overwritten".to_vec());
    assert_eq!(batch.get(cf::SITE_KV, b"key").unwrap(), b"overwritten");
    batch.rollback_revertable();
    assert_eq!(batch.get(cf::SITE_KV, b"key").unwrap(), b"original");
}

#[test]
fn commit_revertable_keeps_all_writes() {
    let db = test_db("commit_revertable");
    let batch = BatchDb::new(db);
    batch.begin_revertable();
    batch.put(cf::SITE_KV, b"k1", b"v1".to_vec());
    batch.put(cf::SITE_KV, b"k2", b"v2".to_vec());
    batch.commit_revertable();
    assert_eq!(batch.get(cf::SITE_KV, b"k1").unwrap(), b"v1");
    assert_eq!(batch.get(cf::SITE_KV, b"k2").unwrap(), b"v2");
}

#[test]
fn rollback_preserves_writes_before_begin() {
    let db = test_db("rollback_preserves");
    let batch = BatchDb::new(db);
    batch.put(cf::SITE_KV, b"pre1", b"a".to_vec());
    batch.put(cf::SITE_KV, b"pre2", b"b".to_vec());
    batch.begin_revertable();
    batch.put(cf::SITE_KV, b"post", b"c".to_vec());
    batch.rollback_revertable();
    assert_eq!(batch.get(cf::SITE_KV, b"pre1").unwrap(), b"a");
    assert_eq!(batch.get(cf::SITE_KV, b"pre2").unwrap(), b"b");
    assert!(batch.get(cf::SITE_KV, b"post").is_none());
}
