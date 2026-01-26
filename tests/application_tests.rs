use nimble_web::app::builder::AppBuilder;
use nimble_web::app::application::AppError;

#[tokio::test]
async fn application_start_rejects_invalid_address() {
    let mut builder = AppBuilder::new();
    builder.use_address("not-an-addr");
    let app = builder.build();

    let result = app.start().await;
    match result {
        Err(AppError::InvalidAddress(value)) => assert_eq!(value, "not-an-addr"),
        other => panic!("unexpected result: {:?}", other),
    }
}

#[test]
fn app_error_display_includes_reason() {
    let error = AppError::InvalidAddress("bad".to_string());
    assert!(error.to_string().contains("invalid address"));
}
