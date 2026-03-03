use axum_api::auth::jwt::{decode_token, encode_token, Claims};

#[test]
fn test_jwt_roundtrip() {
    let claims = Claims {
        sub: "user_1".into(),
        role: "USER".into(),
        exp: 2_000_000_000,
    };
    let token = encode_token(&claims, "secret").unwrap();
    let decoded = decode_token(&token, "secret").unwrap();
    assert_eq!(decoded.sub, "user_1");
    assert_eq!(decoded.role, "USER");
}
