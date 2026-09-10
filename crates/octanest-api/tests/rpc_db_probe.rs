use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use octanest_api::{build_cors, router};
use octanest_db::Database;
use tower::ServiceExt;

fn app_with(db: Database) -> axum::Router {
    let cors = build_cors("development", None).expect("cors");
    router(db, cors)
}

#[tokio::test]
async fn db_probe_without_database_returns_not_configured() {
    let app = app_with(Database::skipped());
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rpc")
                .header("content-type", "application/json")
                .header("Octanest-RPC-Version", "1")
                .body(Body::from(r#"{"procedure":"system.db_probe","input":{}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("db.not_configured"), "body: {body}");
    assert!(body.contains("\"ok\":false"), "body: {body}");
}

#[tokio::test]
async fn db_probe_requires_version_header() {
    let app = app_with(Database::skipped());
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rpc")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"procedure":"system.db_probe","input":{}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("rpc.version_mismatch"), "body: {body}");
}

#[tokio::test]
async fn db_probe_round_trip_sqlite() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("octanest.db");
    let url = format!("sqlite:{}", db_path.display());

    let db = Database::connect(&url).await.expect("connect sqlite");
    db.migrate().await.expect("migrate sqlite");
    let app = app_with(db);

    let req = |body: &'static str| {
        Request::builder()
            .method("POST")
            .uri("/api/rpc")
            .header("content-type", "application/json")
            .header("Octanest-RPC-Version", "1")
            .body(Body::from(body))
            .unwrap()
    };

    let res1 = app
        .clone()
        .oneshot(req(r#"{"procedure":"system.db_probe","input":{}}"#))
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    let bytes1 = res1.into_body().collect().await.unwrap().to_bytes();
    let v1: serde_json::Value = serde_json::from_slice(&bytes1).unwrap();
    assert_eq!(v1["data"]["dialect"], "sqlite");
    let count1 = v1["data"]["probe_count"].as_i64().unwrap();

    let res2 = app
        .oneshot(req(r#"{"procedure":"system.db_probe","input":{}}"#))
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    let bytes2 = res2.into_body().collect().await.unwrap().to_bytes();
    let v2: serde_json::Value = serde_json::from_slice(&bytes2).unwrap();
    assert_eq!(v2["data"]["dialect"], "sqlite");
    let count2 = v2["data"]["probe_count"].as_i64().unwrap();

    assert!(
        count2 > count1,
        "expected probe_count to increase: {count1} -> {count2}"
    );
}
