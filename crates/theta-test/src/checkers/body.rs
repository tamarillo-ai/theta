//! Markdown body semantic equality.
//!
//! Compares content after frontmatter, tolerating trailing whitespace
//! and CRLF normalization.

use std::path::Path;

use super::frontmatter::parse_document;

/// Normalize a body string for comparison: trim, normalize CRLF --> LF.
fn normalize_body(s: &str) -> String {
    s.replace("\r\n", "\n").trim().to_string()
}

/// Assert that two markdown files have semantically equal body content.
///
/// Strips frontmatter, normalizes CRLF --> LF, trims whitespace.
pub fn assert_body_equal(original_path: &Path, cast_path: &Path) {
    let orig_content = fs_err::read_to_string(original_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original_path.display()));
    let cast_content = fs_err::read_to_string(cast_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast_path.display()));

    let orig = parse_document(&orig_content);
    let cast = parse_document(&cast_content);

    let orig_body = normalize_body(&orig.body);
    let cast_body = normalize_body(&cast.body);

    if orig_body != cast_body {
        // find first differing line for a useful error
        let orig_lines: Vec<&str> = orig_body.lines().collect();
        let cast_lines: Vec<&str> = cast_body.lines().collect();
        let first_diff = orig_lines
            .iter()
            .zip(cast_lines.iter())
            .enumerate()
            .find(|(_, (a, b))| a != b);

        let detail = match first_diff {
            Some((i, (a, b))) => format!("first diff at line {}: orig={a:?} cast={b:?}", i + 1),
            None => format!(
                "line count differs: orig={} cast={}",
                orig_lines.len(),
                cast_lines.len()
            ),
        };

        panic!(
            "body content differs between {} and {}:\n{detail}",
            original_path.display(),
            cast_path.display(),
        );
    }
}

/// Assert that two files are byte-identical (for skills, scripts, etc.).
pub fn assert_file_identical(original_path: &Path, cast_path: &Path) {
    let orig = fs_err::read(original_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original_path.display()));
    let cast = fs_err::read(cast_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast_path.display()));

    // tolerate trailing newline difference (POSIX fix)
    let orig_trimmed = orig.strip_suffix(b"\n").unwrap_or(&orig);
    let cast_trimmed = cast.strip_suffix(b"\n").unwrap_or(&cast);

    assert!(
        orig_trimmed == cast_trimmed,
        "files differ (not byte-identical): {} vs {}",
        original_path.display(),
        cast_path.display(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_normalization() {
        let a = "line1\r\nline2\r\n";
        let b = "line1\nline2\n";
        assert_eq!(normalize_body(a), normalize_body(b));
    }

    #[test]
    fn trailing_whitespace_tolerated() {
        let a = "content\n\n";
        let b = "content";
        assert_eq!(normalize_body(a), normalize_body(b));
    }
}
