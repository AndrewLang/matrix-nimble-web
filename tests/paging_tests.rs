use nimble_web::data::paging::{Page, PageRequest};

#[test]
fn page_request_validation_rejects_zero_values() {
    let request = PageRequest::new(0, 10);
    assert!(request.validate().is_err());

    let request = PageRequest::new(1, 0);
    assert!(request.validate().is_err());
}

#[test]
fn page_request_validation_accepts_positive_values() {
    let request = PageRequest::new(1, 25);
    assert!(request.validate().is_ok());
}

#[test]
fn page_construction_keeps_fields() {
    let page = Page::new(vec![1, 2, 3], 10, 2, 3);
    assert_eq!(page.items, vec![1, 2, 3]);
    assert_eq!(page.total, 10);
    assert_eq!(page.page, 2);
    assert_eq!(page.page_size, 3);
}
