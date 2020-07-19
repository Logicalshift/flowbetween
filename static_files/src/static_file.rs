use sha2::{Sha256, Digest};

///
/// Represents a static file
///
pub struct StaticFile {
    mime_type: String,
    path: String,
    etag: String,
    content: Vec<u8>
}

impl StaticFile {
    ///
    /// Returns the MIME type for a particular path
    ///
    fn infer_mime_type<'a>(path: &str) -> &'a str {
        if path.ends_with("html") || path.ends_with("htm") {
            "text/html; charset=utf-8"
        } else if path.ends_with("css") {
            "text/css; charset=utf-8"
        } else if path.ends_with("js") {
            "text/javascript; charset=utf-8"
        } else if path.ends_with("json") {
            "application/json; charset=utf-8"
        } else if path.ends_with("png") {
            "image/png"
        } else if path.ends_with("gif") {
            "image/gif"
        } else if path.ends_with("jpg") || path.ends_with("jpeg") {
            "image/jpeg"
        } else if path.ends_with("svg") {
            "image/svg+xml; charset=utf-8"
        } else if path.ends_with("txt") {
            "text/plain; charset=utf-8"
        } else if path.ends_with("woff2") {
            "font/woff2"
        } else {
            "application/octet-stream"
        }
    }

    ///
    /// Creates an etag from the content for an item
    ///
    fn etag_from_content(content: &[u8]) -> String {
        // Hash the content using SHA-256
        let mut hasher = Sha256::default();
        hasher.update(content);
        let output = hasher.finalize();

        // Use first few bytes to build a string
        output.iter()
            .take(8)
            .fold(String::new(), |so_far, next_byte| so_far + &format!("{:02x}", next_byte))
    }

    ///
    /// Creates a new static file with an inferred MIME type
    ///
    pub fn new_with_type(mime_type: &str, path: &str, content: &[u8]) -> StaticFile {
        StaticFile {
            mime_type:  String::from(mime_type),
            path:       String::from(path),
            etag:       StaticFile::etag_from_content(content),
            content:    Vec::from(content)
        }
    }

    ///
    /// Creates a new static file with an explicit MIME type
    ///
    pub fn new(path: &str, content: &[u8]) -> StaticFile {
        StaticFile::new_with_type(
            StaticFile::infer_mime_type(path),
            path,
            content
        )
    }

    ///
    /// Returns the list of paths where this string can be
    /// accessed
    ///
    pub fn valid_paths(&self) -> Vec<String> {
        let mut result = Vec::new();

        // Can always access at the default path
        result.push(self.path.clone());

        // index.html files are also allowed at the root
        if self.path.ends_with("/index.html") {
            let no_chars_without_index = self.path.len() - "index.html".len();
            result.push(String::from(&self.path[..no_chars_without_index]));
        }

        result
    }

    ///
    /// Retrieves the bytes that make up this static file
    ///
    pub fn content<'a>(&'a self) -> &'a [u8] {
        &self.content
    }

    ///
    /// Retrieves the MIME type of this static file
    ///
    pub fn mime_type<'a>(&'a self) -> &'a str {
        &self.mime_type
    }

    ///
    /// Computes the etag for this file
    ///
    pub fn etag(&self) -> String {
        self.etag.clone()
    }
}
