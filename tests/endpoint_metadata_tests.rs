use nimble_web::endpoint::metadata::EndpointMetadata;
use nimble_web::security::policy::Policy;
use nimble_web::validation::{AnyValidator, ValidationError};

struct AlwaysOkValidator;

impl AnyValidator for AlwaysOkValidator {
    fn validate(
        &self,
        _context: &nimble_web::http::context::HttpContext,
    ) -> Result<(), ValidationError> {
        Ok(())
    }
}

#[test]
fn endpoint_metadata_tracks_name_tags_policy_and_validators() {
    let metadata = EndpointMetadata::new("GET", "/test")
        .with_name("test")
        .with_tags(vec!["a", "b"])
        .require_policy(Policy::Authenticated)
        .add_validator(AlwaysOkValidator);

    assert_eq!(metadata.method(), "GET");
    assert_eq!(metadata.route_pattern(), "/test");
    assert_eq!(metadata.name(), Some("test"));
    assert_eq!(metadata.tags(), &["a".to_string(), "b".to_string()]);
    assert_eq!(metadata.policy(), Some(&Policy::Authenticated));
    assert_eq!(metadata.validators().len(), 1);
}
