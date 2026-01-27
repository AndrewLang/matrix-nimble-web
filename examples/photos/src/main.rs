mod app;

use app::PhotosApp;

#[tokio::main]
async fn main() {
    let app = PhotosApp::new();
    app.run().await;
}
