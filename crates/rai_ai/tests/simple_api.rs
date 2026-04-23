//! Tests for the simplified API

#[test]
fn test_simple_model_export() {
    // Verify SimpleModel is exported from rai_ai
    use rai_ai::SimpleModel;
    // SimpleModel is just a wrapper, so this verifies it exists
    assert_eq!(std::any::type_name::<SimpleModel>(), "rai_ai::simple::SimpleModel");
}
