use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::ManifestHashError;
use crate::types::ContentHash;

/// SHA-256 hash of raw bytes, returned as `"sha256:<64 lowercase hex>"`.
pub fn content_hash(data: &[u8]) -> ContentHash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    ContentHash::from_bytes(hasher.finalize().into())
}

/// Canonical hash of `theta.toml` — parses, re-serializes to a stable
/// form, then SHA-256s that. Immune to comments, whitespace, and key
/// reordering. Format: `"sha256:<64 lowercase hex>"`.
pub fn manifest_hash(manifest_bytes: &[u8]) -> Result<ContentHash, ManifestHashError> {
    let manifest_str = std::str::from_utf8(manifest_bytes)?;
    let manifest = toml::from_str::<theta_schema::ThetaManifest>(manifest_str)?;
    let canonical = toml::to_string(&manifest)?;
    Ok(content_hash(canonical.as_bytes()))
}

/// Recursive hash of a skill directory — walks sorted entries, hashing
/// each relative path + file content. Format: `"sha256:<64 lowercase hex>"`.
pub fn skill_content_hash(skill_dir: &Path) -> Result<ContentHash, std::io::Error> {
    let mut hasher = Sha256::new();
    hash_dir_recursive(skill_dir, skill_dir, &mut hasher)?;
    Ok(ContentHash::from_bytes(hasher.finalize().into()))
}

fn hash_dir_recursive(base: &Path, dir: &Path, hasher: &mut Sha256) -> Result<(), std::io::Error> {
    let mut entries: Vec<_> = fs_err::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(fs_err::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_str()
            .expect("non-UTF-8 path in content hash");

        if path.is_dir() {
            hash_dir_recursive(base, &path, hasher)?;
        } else {
            hasher.update(rel.as_bytes());
            hasher.update(b"\0");
            let data = fs_err::read(&path)?;
            hasher.update(&data);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_is_deterministic() {
        let h1 = content_hash(b"hello world");
        let h2 = content_hash(b"hello world");
        assert_eq!(h1, h2);
        assert!(h1.to_string().starts_with("sha256:"));
    }

    #[test]
    fn different_content_different_hash() {
        let h1 = content_hash(b"hello");
        let h2 = content_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn manifest_hash_ignores_comments_and_whitespace() {
        let with_comments = b"# this is a comment\n[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"desc\"\nversion = \"0.1.0\"\n";
        let without_comments = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"desc\"\nversion = \"0.1.0\"\n";
        let with_extra_newlines = b"\n\n[theta]\nschema = \"2026-04\"\n\n\n\n[agent]\nname = \"test\"\ndescription = \"desc\"\nversion = \"0.1.0\"\n\n\n";

        let h1 = manifest_hash(with_comments).unwrap();
        let h2 = manifest_hash(without_comments).unwrap();
        let h3 = manifest_hash(with_extra_newlines).unwrap();

        assert_eq!(h1, h2, "comments should not affect manifest hash");
        assert_eq!(h2, h3, "extra whitespace should not affect manifest hash");
    }

    #[test]
    fn manifest_hash_ignores_section_reordering() {
        let canonical = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"desc\"\nversion = \"0.1.0\"\n";
        let agent_first = b"[agent]\nname = \"test\"\ndescription = \"desc\"\nversion = \"0.1.0\"\n\n[theta]\nschema = \"2026-04\"\n";

        let h1 = manifest_hash(canonical).unwrap();
        let h2 = manifest_hash(agent_first).unwrap();

        assert_eq!(h1, h2, "section order should not affect manifest hash");
    }

    #[test]
    fn manifest_hash_ignores_key_reordering_within_section() {
        let canonical = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"desc\"\nversion = \"0.1.0\"\n";
        let keys_reordered = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nversion = \"0.1.0\"\nname = \"test\"\ndescription = \"desc\"\n";

        let h1 = manifest_hash(canonical).unwrap();
        let h2 = manifest_hash(keys_reordered).unwrap();

        assert_eq!(
            h1, h2,
            "key order within a section should not affect manifest hash"
        );
    }

    #[test]
    fn manifest_hash_differs_when_skill_added() {
        let one_skill = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"test\"\nversion = \"0.1.0\"\n\n[skills.react-native-skills]\nsource = { git = \"https://github.com/vercel-labs/agent-skills\", subdirectory = \"skills/react-native-skills\" }\n";
        let two_skills = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"test\"\nversion = \"0.1.0\"\n\n[skills.react-native-skills]\nsource = { git = \"https://github.com/vercel-labs/agent-skills\", subdirectory = \"skills/react-native-skills\" }\n\n[skills.vercel-cli-with-tokens]\nsource = { git = \"https://github.com/vercel-labs/agent-skills\", subdirectory = \"skills/vercel-cli-with-tokens\" }\n";

        let h1 = manifest_hash(one_skill).unwrap();
        let h2 = manifest_hash(two_skills).unwrap();
        assert_ne!(h1, h2, "adding a skill must change the manifest hash");
    }

    #[test]
    fn manifest_hash_rejects_invalid_utf8() {
        let garbage: &[u8] = &[0xff, 0xfe, 0x00, 0x01];
        let err = manifest_hash(garbage).unwrap_err();
        assert!(
            matches!(err, ManifestHashError::InvalidUtf8(_)),
            "expected InvalidUtf8, got: {err:?}"
        );
    }

    #[test]
    fn manifest_hash_rejects_invalid_toml() {
        let bad_toml = b"this is not [valid toml";
        let err = manifest_hash(bad_toml).unwrap_err();
        assert!(
            matches!(err, ManifestHashError::InvalidToml(_)),
            "expected InvalidToml, got: {err:?}"
        );
    }

    #[test]
    fn manifest_hash_rejects_valid_toml_but_wrong_schema() {
        let wrong_shape = b"[something]\nkey = \"value\"\n";
        let err = manifest_hash(wrong_shape).unwrap_err();
        assert!(
            matches!(err, ManifestHashError::InvalidToml(_)),
            "expected InvalidToml for wrong schema, got: {err:?}"
        );
    }
}
