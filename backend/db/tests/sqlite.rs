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
