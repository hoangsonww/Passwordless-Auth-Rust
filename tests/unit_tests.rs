use passwordless_auth::{
    config::Config,
    db::Database,
    jwt,
    magic_link::{MagicLink, MagicLinkError},
    session::Session,
    totp,
};
use rusqlite::params;
use std::fs;
use uuid::Uuid;

#[test]
fn test_jwt_create_verify() {
    let secret = "supersecret1234567890";
    let user_id = "user-abc";
    let access = jwt::create_token(user_id, secret, 60, "access").unwrap();
    let refresh = jwt::create_token(user_id, secret, 120, "refresh").unwrap();

    let claims_access = jwt::verify_token(&access, secret).unwrap();
    assert_eq!(claims_access.sub, user_id);
    assert_eq!(claims_access.kind, "access");

    let claims_refresh = jwt::verify_token(&refresh, secret).unwrap();
    assert_eq!(claims_refresh.sub, user_id);
    assert_eq!(claims_refresh.kind, "refresh");

    // invalid token
    let bad = jwt::verify_token("invalidtoken", secret);
    assert!(bad.is_err());
}

#[test]
fn test_totp_generation_and_verification() {
    let secret = totp::generate_secret();
    assert!(!secret.is_empty());

    // generate current code using the same logic
    // decode base32
    let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
        .expect("decode secret");
    let code_num = oath::totp_raw_now(&secret_bytes, 6, 0, 30);
    let code = format!("{:06}", code_num);
    // verify with module
    assert!(totp::verify_code(&secret, &code).is_ok());

    // wrong code
    assert!(totp::verify_code(&secret, "000000").is_err());
}

#[test]
fn test_magic_link_lifecycle() {
    // in-memory DB
    let db = Database::open(":memory:").expect("open db");
    let migration_sql = fs::read_to_string("migrations/init.sql").expect("read migration");
    db.migrate(&migration_sql).expect("migrate");

    // create user
    let email = format!("unit+{}@example.com", Uuid::new_v4());
    let user_id = db.get_or_create_user(&email).unwrap();
    assert!(!user_id.is_empty());

    // generate magic link
    let token = MagicLink::generate(&db, &user_id, 5).unwrap(); // 5 seconds lifetime
    assert!(!token.is_empty());

    // consume successfully
    let uid = MagicLink::consume(&db, &token).unwrap();
    assert_eq!(uid, user_id);

    // second consume should fail (used)
    let second = MagicLink::consume(&db, &token);
    assert!(matches!(second, Err(MagicLinkError::Used)));

    // expired link: insert new one with artificially old expires_at
    let token2 = MagicLink::generate(&db, &user_id, 1).unwrap();
    // manually set expires_at in past
    let past = Database::now_ts() - 100;
    db.conn
        .execute(
            "UPDATE magic_links SET expires_at = ?1 WHERE token = ?2",
            params![past, token2],
        )
        .unwrap();
    let expired = MagicLink::consume(&db, &token2);
    assert!(matches!(expired, Err(MagicLinkError::Invalid)));
}

#[test]
fn test_session_refresh_token_and_revocation() {
    let db = Database::open(":memory:").expect("open db");
    let migration_sql = fs::read_to_string("migrations/init.sql").expect("read migration");
    db.migrate(&migration_sql).expect("migrate");

    // create user
    let email = format!("unit+{}@example.com", Uuid::new_v4());
    let user_id = db.get_or_create_user(&email).unwrap();

    // create refresh token
    let token = Session::create_refresh_token(&db, &user_id, 60).unwrap();
    assert!(!token.is_empty());

    // validate
    let validated = Session::validate_refresh_token(&db, &token).unwrap();
    assert_eq!(validated, user_id);

    // revoke
    Session::revoke_refresh_token(&db, &token).unwrap();
    let invalid = Session::validate_refresh_token(&db, &token);
    assert!(invalid.is_err());
}
