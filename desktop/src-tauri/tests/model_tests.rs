use stenodrop_lib::model::{model_path, validate_download, MIN_VALID_SIZE};

#[test]
fn rejects_non_200_status_even_with_large_body() {
    let err = validate_download(404, 500_000_000).unwrap_err();
    assert!(err.contains("404"), "error should name the status: {err}");
}

#[test]
fn rejects_truncated_body_even_with_200() {
    // Captive portal / error page scenario from the Mac ModelDownloader fix.
    let err = validate_download(200, 10_000_000).unwrap_err();
    assert!(
        err.to_lowercase().contains("incomplete"),
        "error should say incomplete: {err}"
    );
}

#[test]
fn accepts_full_download() {
    assert!(validate_download(200, 465_000_000).is_ok());
    assert!(MIN_VALID_SIZE >= 400_000_000);
}

#[test]
fn model_path_is_under_stenodrop_app_data() {
    let p = model_path();
    let s = p.to_string_lossy();
    assert!(s.contains("StenoDrop"), "path was {s}");
    assert!(s.ends_with("ggml-small.bin"), "path was {s}");
}
