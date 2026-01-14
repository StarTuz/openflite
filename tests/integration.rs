use openflite_core::Core;

#[tokio::test]
async fn test_core_initialization() {
    let (core, _) = Core::new();
    // Verify we can call scan (it will likely return empty on CI without real hardware)
    let result = core.scan_devices();
    assert!(result.is_ok());
}
