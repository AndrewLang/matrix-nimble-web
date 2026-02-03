use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::http::context::HttpContext;
use crate::http::response_body::{ResponseBody, ResponseBodyStream};
use crate::result::into_response::IntoResponse;

pub struct FileResponse {
    source: FileSource,
    content_type: Option<String>,
    filename: Option<String>,
    headers: Vec<(String, String)>,
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
            headers: Vec::new(),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            source: FileSource::Bytes(bytes),
            content_type: None,
            filename: None,
            headers: Vec::new(),
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

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
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
            "webp" => "image/webp",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "pdf" => "application/pdf",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            _ => return None,
        };
        Some(content_type.to_string())
    }
}

impl IntoResponse for FileResponse {
    fn into_response(self, context: &mut HttpContext) {
        let response = context.response_mut();
        response.set_status(200);

        let inferred = match &self.source {
            FileSource::Path(path) => Self::infer_content_type(path),
            FileSource::Bytes(_) => None,
        };

        let mut content_length = None;

        let body = match self.source {
            FileSource::Path(path) => match File::open(&path) {
                Ok(file) => {
                    if let Ok(metadata) = file.metadata() {
                        content_length = Some(metadata.len());
                    }
                    ResponseBody::Stream(Box::new(FileStream::new(file)))
                }
                Err(err) => {
                    response.set_status(match err.kind() {
                        std::io::ErrorKind::NotFound => 404,
                        _ => 500,
                    });
                    ResponseBody::Empty
                }
            },
            FileSource::Bytes(bytes) => {
                content_length = Some(bytes.len() as u64);
                ResponseBody::Bytes(bytes)
            }
        };

        response.set_body(body);

        // Only set headers if we are returning success
        if response.status() == 200 {
            let content_type = self
                .content_type
                .or(inferred)
                .unwrap_or_else(|| "application/octet-stream".to_string());
            response.headers_mut().insert("content-type", &content_type);

            for (key, value) in self.headers {
                response.headers_mut().insert(&key, &value);
            }

            if let Some(len) = content_length {
                response
                    .headers_mut()
                    .insert("content-length", &len.to_string());
            }

            if let Some(filename) = self.filename {
                response.headers_mut().insert(
                    "content-disposition",
                    &format!("attachment; filename=\"{}\"", filename),
                );
            }
        }
    }
}

struct FileStream {
    file: File,
    buffer_size: usize,
}

impl FileStream {
    fn new(file: File) -> Self {
        Self {
            file,
            buffer_size: 8 * 1024,
        }
    }
}

impl ResponseBodyStream for FileStream {
    fn read_chunk(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        let mut buffer = vec![0u8; self.buffer_size];
        let bytes_read = self.file.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        buffer.truncate(bytes_read);
        Ok(Some(buffer))
    }
}
