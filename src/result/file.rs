use std::path::{Path, PathBuf};

use crate::http::context::HttpContext;
use crate::http::response_body::ResponseBody;
use crate::result::into_response::IntoResponse;

pub struct FileResponse {
    source: FileSource,
    content_type: Option<String>,
    filename: Option<String>,
}

enum FileSource {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

impl FileResponse {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            source: FileSource::Path(path.into()),
            content_type: None,
            filename: None,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            source: FileSource::Bytes(bytes),
            content_type: None,
            filename: None,
        }
    }

    pub fn with_content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }

    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = Some(filename.to_string());
        self
    }
}

impl IntoResponse for FileResponse {
    fn into_response(self, context: &mut HttpContext) {
        let response = context.response_mut();
        response.set_status(200);

        let (bytes, inferred) = match &self.source {
            FileSource::Path(path) => match std::fs::read(path) {
                Ok(bytes) => (bytes, infer_content_type(path)),
                Err(err) => {
                    response.set_status(match err.kind() {
                        std::io::ErrorKind::NotFound => 404,
                        _ => 500,
                    });
                    response.set_body(ResponseBody::Empty);
                    return;
                }
            },
            FileSource::Bytes(bytes) => (bytes.clone(), None),
        };

        response.set_body(ResponseBody::Bytes(bytes));

        let content_type = self
            .content_type
            .or(inferred)
            .unwrap_or_else(|| "application/octet-stream".to_string());
        response.headers_mut().insert("content-type", &content_type);

        if let Some(filename) = self.filename {
            response.headers_mut().insert(
                "content-disposition",
                &format!("attachment; filename=\"{}\"", filename),
            );
        }
    }
}

fn infer_content_type(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let content_type = match ext.as_str() {
        "txt" => "text/plain; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "htm" => "text/html; charset=utf-8",
        "json" => "application/json",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    Some(content_type.to_string())
}
