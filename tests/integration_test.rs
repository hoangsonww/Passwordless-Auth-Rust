use reqwest::Client;
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tokio::time::sleep;
use uuid::Uuid;
use rusqlite::{Connection, params};

fn build_config_override(base: &str, db_path: &str, tempdir: &PathBuf) -> String {
    let mut config = fs::read_to_string(base).expect("read base config.toml");
    config = config.replace("database_path = \"auth.db\"", &format!("database_path = \"{}\"", db_path));
    // ensure magic_link_base_url points to localhost
    // optional: already in base
    let dest = tempdir.join("config.toml");
    fs::write(&dest, config).expect("write overridden config");
    dest.to_string_lossy().to_string()
}

async fn wait_for_server_ready() {
    let client = Client::new();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(15) {
        if let Ok(resp) = client.get("http://localhost:3000/").send().await {
            if resp.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_millis(200)).await;
    }
    panic!("server did not become ready in time");
}

fn start_server_in_dir(dir: &PathBuf) -> Child {
    // Build release binary before test if not exists
    // Assume binary output at ../target/debug/passwordless-auth or release if built
    let bin_path = {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("target");
        p.push("debug");
        p.push("passwordless-auth");
        if !p.exists() {
            // try release
            p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.push("target");
            p.push("release");
            p.push("passwordless-auth");
        }
        if !p.exists() {
            panic!("server binary not built; run `cargo build` first");
        }
        p
    };

    let mut child = Command::new(bin_path)
        .current_dir(dir)
        .env("RUST_LOG", "info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn auth server binary");
    // give a little time: actual readiness wait done externally
    thread::sleep(Duration::from_millis(200));
    child
}

#[tokio::test]
async fn magic_link_flow() {
    // Setup temp workspace
    let temp = TempDir::new().unwrap();
    let tmp_path = temp.path().to_path_buf();

    // Copy migration file
    fs::create_dir_all(tmp_path.join("migrations")).unwrap();
    fs::copy(
        PathBuf::from("migrations/init.sql"),
        tmp_path.join("migrations/init.sql"),
    )
    .unwrap();

    // Override config to point at temp db
    let db_file = tmp_path.join("auth.db");
    let config_path = build_config_override("config.toml", db_file.to_str().unwrap(), &tmp_path);

    // Start server
    let mut child = start_server_in_dir(&tmp_path);

    // Wait for server to be up
    wait_for_server_ready().await;

    let client = Client::new();
    let email = format!("test+{}@example.com", Uuid::new_v4());

    // Request magic link
    let resp = client
        .post("http://localhost:3000/request/magic")
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await
        .expect("request magic link");
    assert!(resp.status().is_success());

    // Open DB and fetch token
    let conn = Connection::open(db_file).unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT token FROM magic_links JOIN users ON users.id=magic_links.user_id WHERE users.email = ?1",
        )
        .unwrap();
    let token_row: String = stmt
        .query_row(params![email], |r| r.get(0))
        .unwrap();

    // Verify magic link
    let verify = client
        .get("http://localhost:3000/verify/magic")
        .query(&[("token", token_row.clone())])
        .send()
        .await
        .expect("verify magic");
    assert!(verify.status().is_success(), "magic verify failed: {:?}", verify.text().await);
    let body: Value = verify.json().await.unwrap();
    assert!(body.get("access_token").is_some(), "missing access_token");
    assert!(body.get("refresh_token").is_some(), "missing refresh_token");

    // Cleanup
    let _ = child.kill();
}

#[tokio::test]
async fn refresh_token_flow() {
    let temp = TempDir::new().unwrap();
    let tmp_path = temp.path().to_path_buf();

    fs::create_dir_all(tmp_path.join("migrations")).unwrap();
    fs::copy(
        PathBuf::from("migrations/init.sql"),
        tmp_path.join("migrations/init.sql"),
    )
    .unwrap();

    let db_file = tmp_path.join("auth.db");
    let config_path = build_config_override("config.toml", db_file.to_str().unwrap(), &tmp_path);

    let mut child = start_server_in_dir(&tmp_path);
    wait_for_server_ready().await;

    let client = Client::new();
    let email = format!("refresh+{}@example.com", Uuid::new_v4());

    // Request magic link and verify to get refresh token
    client
        .post("http://localhost:3000/request/magic")
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await
        .unwrap();

    // Pull magic link token from DB
    let conn = Connection::open(db_file).unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT token FROM magic_links JOIN users ON users.id=magic_links.user_id WHERE users.email = ?1",
        )
        .unwrap();
    let magic_token: String = stmt
        .query_row(params![email], |r| r.get(0))
        .unwrap();

    let verify = client
        .get("http://localhost:3000/verify/magic")
        .query(&[("token", magic_token.clone())])
        .send()
        .await
        .unwrap();
    assert!(verify.status().is_success());
    let tokens: Value = verify.json().await.unwrap();
    let refresh_token = tokens
        .get("refresh_token")
        .expect("refresh_token")
        .as_str()
        .unwrap()
        .to_string();

    // Use refresh endpoint
    let refresh_resp = client
        .post("http://localhost:3000/token/refresh")
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .unwrap();
    assert!(refresh_resp.status().is_success(), "refresh failed: {:?}", refresh_resp.text().await);
    let new_tokens: Value = refresh_resp.json().await.unwrap();
    assert!(new_tokens.get("access_token").is_some());
    assert!(new_tokens.get("refresh_token").is_some());

    let _ = child.kill();
}

#[tokio::test]
async fn totp_flow() {
    let temp = TempDir::new().unwrap();
    let tmp_path = temp.path().to_path_buf();

    fs::create_dir_all(tmp_path.join("migrations")).unwrap();
    fs::copy(
        PathBuf::from("migrations/init.sql"),
        tmp_path.join("migrations/init.sql"),
    )
    .unwrap();

    let db_file = tmp_path.join("auth.db");
    let config_path = build_config_override("config.toml", db_file.to_str().unwrap(), &tmp_path);

    let mut child = start_server_in_dir(&tmp_path);
    wait_for_server_ready().await;

    let client = Client::new();
    let email = format!("totp+{}@example.com", Uuid::new_v4());

    // Enroll TOTP
    let enroll = client
        .post("http://localhost:3000/totp/enroll")
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await
        .unwrap();
    assert!(enroll.status().is_success());
    let enroll_resp: Value = enroll.json().await.unwrap();
    let secret = enroll_resp
        .get("secret")
        .expect("secret")
        .as_str()
        .unwrap()
        .to_string();

    // Compute current TOTP code using same algorithm (allow slight skew)
    let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
        .expect("decode base32 secret");
    let code_num = oath::totp_raw_now(&secret_bytes, 6, 0, 30);
    let code = format!("{:06}", code_num);

    // Verify TOTP
    let verify = client
        .post("http://localhost:3000/totp/verify")
        .json(&serde_json::json!({ "email": email, "code": code }))
        .send()
        .await
        .unwrap();
    assert!(verify.status().is_success());
    let tokens: Value = verify.json().await.unwrap();
    assert!(tokens.get("access_token").is_some());
    assert!(tokens.get("refresh_token").is_some());

    // Invalid TOTP code should fail
    let bad = client
        .post("http://localhost:3000/totp/verify")
        .json(&serde_json::json!({ "email": email, "code": "000000" }))
        .send()
        .await
        .unwrap();
    assert!(bad.status().is_client_error());

    let _ = child.kill();
}

#[tokio::test]
async fn invalid_magic_link() {
    let temp = TempDir::new().unwrap();
    let tmp_path = temp.path().to_path_buf();
    fs::create_dir_all(tmp_path.join("migrations")).unwrap();
    fs::copy(
        PathBuf::from("migrations/init.sql"),
        tmp_path.join("migrations/init.sql"),
    )
    .unwrap();
    let db_file = tmp_path.join("auth.db");
    let config_path = build_config_override("config.toml", db_file.to_str().unwrap(), &tmp_path);
    let mut child = start_server_in_dir(&tmp_path);
    wait_for_server_ready().await;

    let client = Client::new();
    let resp = client
        .get("http://localhost:3000/verify/magic")
        .query(&[("token", "nonexistent-token")])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_client_error());

    let _ = child.kill();
}

#[tokio::test]
async fn webauthn_options_and_invalid_complete() {
    let temp = TempDir::new().unwrap();
    let tmp_path = temp.path().to_path_buf();
    fs::create_dir_all(tmp_path.join("migrations")).unwrap();
    fs::copy(
        PathBuf::from("migrations/init.sql"),
        tmp_path.join("migrations/init.sql"),
    )
    .unwrap();
    let db_file = tmp_path.join("auth.db");
    let config_path = build_config_override("config.toml", db_file.to_str().unwrap(), &tmp_path);
    let mut child = start_server_in_dir(&tmp_path);
    wait_for_server_ready().await;

    let client = Client::new();
    let email = format!("webauthn+{}@example.com", Uuid::new_v4());

    // Request registration options
    let reg_opts = client
        .post("http://localhost:3000/webauthn/register/options")
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await
        .unwrap();
    assert!(reg_opts.status().is_success());

    // Complete with bogus data
    let bad_reg = client
        .post("http://localhost:3000/webauthn/register/complete")
        .json(&serde_json::json!({
            "pending_id": "fake",
            "response": { "foo": "bar" }
        }))
        .send()
        .await
        .unwrap();
    assert!(bad_reg.status().is_client_error());

    // Request login options (user may not exist yet)
    let login_opts = client
        .post("http://localhost:3000/webauthn/login/options")
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await
        .unwrap();
    // Could be bad if user not found; just ensure service doesn't crash
    assert!(login_opts.status().is_success() || login_opts.status().is_client_error());

    let bad_login = client
        .post("http://localhost:3000/webauthn/login/complete")
        .json(&serde_json::json!({
            "pending_id": "fake",
            "response": { "foo": "bar" }
        }))
        .send()
        .await
        .unwrap();
    assert!(bad_login.status().is_client_error());

    let _ = child.kill();
}
