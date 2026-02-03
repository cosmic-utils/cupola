#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_no_network_dependencies() {
        // Verify that the image processing crates don't have network capabilities
        // This is a static analysis test to ensure privacy compliance

        // List of crates that should NOT be in dependencies (network-capable)
        let forbidden_crates = [
            "reqwest",
            "hyper",
            "tokio-tungstenite",
            "native-tls",
            "openssl",
        ];

        // Check Cargo.lock for forbidden crates
        let output = Command::new("cargo")
            .args(&["tree", "-e", "normal", "--prefix", "none"])
            .current_dir("/home/bryan/Projects/rust/cupola")
            .output()
            .expect("Failed to run cargo tree");

        let tree_output = String::from_utf8_lossy(&output.stdout);

        for crate_name in &forbidden_crates {
            assert!(
                !tree_output.contains(crate_name),
                "Forbidden network crate found: {}",
                crate_name
            );
        }
    }

    #[test]
    fn test_local_processing_only() {
        // Verify thumbnail generation doesn't make network requests
        // This test checks that the thumbnail service doesn't import any HTTP clients

        let viewer_thumbnail_content = std::fs::read_to_string(
            "/home/bryan/Projects/rust/cupola/crates/viewer/src/thumbnail.rs",
        )
        .expect("Failed to read thumbnail.rs");

        // Should not contain HTTP client imports
        assert!(!viewer_thumbnail_content.contains("reqwest"));
        assert!(!viewer_thumbnail_content.contains("hyper"));
        assert!(!viewer_thumbnail_content.contains("http"));

        // Should contain local file operations
        assert!(
            viewer_thumbnail_content.contains("std::fs")
                || viewer_thumbnail_content.contains("tokio::fs")
                || viewer_thumbnail_content.contains("File::open")
        );
    }

    #[test]
    fn test_image_processing_dependencies() {
        // Verify image processing is done locally
        let allowed_image_crates = ["image", "fast_image_resize", "turbojpeg", "zune-image"];

        let output = Command::new("cargo")
            .args(&["tree", "-e", "normal", "--prefix", "none"])
            .current_dir("/home/bryan/Projects/rust/cupola")
            .output()
            .expect("Failed to run cargo tree");

        let tree_output = String::from_utf8_lossy(&output.stdout);

        // Verify we have image processing crates
        let has_image_processing = allowed_image_crates
            .iter()
            .any(|crate_name| tree_output.contains(crate_name));

        assert!(has_image_processing, "No image processing crates found");
    }

    #[test]
    fn test_no_external_api_calls() {
        // Verify no API endpoints or URLs in the codebase
        let forbidden_patterns = ["http://", "https://", "api.", ".com/", ".org/"];

        // Check viewer crate source
        let viewer_content =
            std::fs::read_to_string("/home/bryan/Projects/rust/cupola/crates/viewer/src/app.rs")
                .unwrap_or_default();

        for pattern in &forbidden_patterns {
            // Allow patterns in comments or documentation
            let lines: Vec<&str> = viewer_content
                .lines()
                .filter(|line| !line.trim().starts_with("//") && !line.trim().starts_with("///"))
                .collect();

            let code_only = lines.join("\n");
            assert!(
                !code_only.contains(pattern),
                "External API pattern found: {}",
                pattern
            );
        }
    }
}
