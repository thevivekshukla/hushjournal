use db::Session;

#[tokio::test]
async fn session_round_trip() {
    let pool = db::connect_pool("sqlite::memory:").await.expect("connect");
    let mut session = Session::new();
    let user_id = uuid::Uuid::now_v7();
    session
        .attach(&pool, "user_id", &user_id)
        .await
        .expect("attach");

    let mut loaded = Session::load(&pool, session.raw_id())
        .await
        .expect("load")
        .expect("session");
    assert_eq!(loaded.user_id(), Some(user_id));

    loaded.destroy(&pool).await.expect("destroy");
    assert!(
        Session::load(&pool, session.raw_id())
            .await
            .expect("load after destroy")
            .is_none()
    );
}

#[tokio::test]
async fn backup_to_path() {
    let dir = std::env::temp_dir().join(format!("e2ejournal-backup-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let src = dir.join("e2ejournal.db");
    let dest = dir.join("backup.db");
    let url = format!("sqlite:{}", src.display());
    db::connect_pool(&url).await.expect("create db");

    let written = db::backup_to(&url, &dest).await.expect("backup");
    assert_eq!(written, dest);
    assert!(dest.is_file());
    assert!(
        dest.metadata().expect("backup metadata").len() > 0,
        "backup should not be empty"
    );

    let err = db::backup_to(&url, &dest)
        .await
        .expect_err("refuse overwrite");
    assert!(err.to_string().contains("already exists"));

    std::fs::remove_dir_all(&dir).ok();
}
