# 📻 Caribbean Radio Player

A web-hosted radio streaming player built with **Rust**, **Leptos** (WASM frontend), and **Axum** (backend).

## Architecture

```
radio-player/
├── Cargo.toml          # Workspace root
├── shared/             # Shared data models (RadioStation)
├── backend/            # Axum API server
│   └── src/main.rs     # /api/stations endpoint + static file serving
└── frontend/           # Leptos WASM UI
    ├── src/
    │   ├── main.rs           # Entry point
    │   ├── app.rs            # App component + layout
    │   └── components/
    │       ├── station_list.rs   # Station browser grid
    │       └── audio_player.rs   # Playback controls
    └── dist/
        ├── index.html        # HTML template
        └── style.css         # Dark theme styling
```

## Features

- **Station browser**: Grid layout with genre tags and play indicators
- **Audio playback**: Uses browser's native `<audio>` element for streaming
- **Play/Pause controls**: Toggle playback with visual feedback
- **Volume control**: Adjustable volume slider
- **Responsive design**: Works on desktop and mobile
- **Dark theme**: Modern dark UI with accent colors

## Prerequisites

- Rust 1.70+ (`rustup install stable`)
- WASM target: `rustup target add wasm32-unknown-unknown`
- wasm-pack: `cargo install wasm-pack`

## Development

### Build the frontend (WASM)

```bash
cd frontend
wasm-pack build --target web --out-dir dist
```

### Run the backend server

```bash
cargo run -p backend
```

The server starts at `http://localhost:3001` and serves:
- API: `http://localhost:3001/api/stations`
- Frontend: `http://localhost:3001/` (static files from `frontend/dist/`)

## Downloading a Release

Pre-built binaries are available on the [Releases page](https://github.com/steveroach/radio-player/releases).

### Linux (x86_64)

```bash
# Download and extract
curl -LO https://github.com/steveroach/radio-player/releases/latest/download/radio-server-x86_64-unknown-linux-gnu.tar.gz
tar xzf radio-server-x86_64-unknown-linux-gnu.tar.gz

# Make executable and run
chmod +x radio-server
./radio-server
```

### Linux (ARM64 / aarch64)

```bash
# Download and extract
curl -LO https://github.com/steveroach/radio-player/releases/latest/download/radio-server-aarch64-unknown-linux-gnu.tar.gz
tar xzf radio-server-aarch64-unknown-linux-gnu.tar.gz

# Make executable and run
chmod +x radio-server
./radio-server
```

### macOS (Apple Silicon)

```bash
# Download and extract
curl -LO https://github.com/steveroach/radio-player/releases/latest/download/radio-server-aarch64-apple-darwin.tar.gz
tar xzf radio-server-aarch64-apple-darwin.tar.gz

# Make executable and run
chmod +x radio-server
./radio-server
```

### macOS (Intel)

```bash
# Download and extract
curl -LO https://github.com/steveroach/radio-player/releases/latest/download/radio-server-x86_64-apple-darwin.tar.gz
tar xzf radio-server-x86_64-apple-darwin.tar.gz

# Make executable and run
chmod +x radio-server
./radio-server
```

### Windows

```powershell
# Download
Invoke-WebRequest -Uri "https://github.com/steveroach/radio-player/releases/latest/download/radio-server-x86_64-pc-windows-msvc.zip" -OutFile "radio-server.zip"

# Extract and run
Expand-Archive -Path "radio-server.zip" -DestinationPath "."
.\radio-server\radio-server.exe
```

### Running the Server

After downloading, place the `radio-server` binary in the project root (next to the `frontend/dist/` directory), then run it:

```bash
./radio-server
```

The server starts at `http://localhost:3001` and serves:
- **Web UI**: `http://localhost:3001/`
- **API**: `http://localhost:3001/api/stations`

> **Note:** The server serves the frontend from `frontend/dist/` relative to the current working directory. Make sure the `frontend/dist/` directory exists, or build it first using the development instructions below.

## Deployment

### Option 1: Single binary deployment

Build the frontend, then run the backend binary which serves both API and static files:

```bash
# Build frontend
cd frontend && wasm-pack build --release --target web --out-dir dist

# Build and run backend
cd ../backend
cargo build --release
../target/release/radio-server
```

### Option 2: Reverse proxy (nginx/Caddy)

Point your web server to serve `frontend/dist/` statically and proxy `/api/*` to the backend:

```nginx
server {
    listen 80;
    
    location /api/ {
        proxy_pass http://127.0.0.1:3001;
    }
    
    location / {
        root /path/to/frontend/dist;
        try_files $uri /index.html;
    }
}
```

### Option 3: Static-only (no backend needed)

If you don't need a dynamic API, you can embed the station list directly in the frontend and deploy `frontend/dist/` to any static host (GitHub Pages, Netlify, S3):

1. Modify `station_list.rs` to use `shared::get_stations()` directly instead of fetching from API
2. Build with `wasm-pack` and deploy the `dist/` folder

## Adding Stations

Edit `shared/src/lib.rs` and add entries to the `get_stations()` function:

```rust
RadioStation {
    index: 8,
    url: "https://your-stream-url.com/stream".to_string(),
    label: "Your Station Name".to_string(),
    type_: vec!["Genre1".to_string(), "Genre2".to_string()],
    image_resource: Some("radio_icon".to_string()),
},
```

## Notes

- Some radio streams use HTTP (not HTTPS). If deploying with HTTPS, you may need a stream proxy for mixed-content streams.
- The browser's `<audio>` element handles most stream formats (MP3, AAC, Ogg). For unsupported formats, consider a backend stream transcoder.
