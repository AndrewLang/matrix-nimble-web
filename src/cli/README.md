# Nimble CLI 🚀

The Nimble CLI is a powerful project generator for the Nimble Web Framework. It helps you scaffold modern web applications with a Rust backend and choice of frontend frameworks.

## Installation

To build the CLI for distribution:

```bash
cargo build --release --features cli --bin nimble
```
The compiled binary will be available at `target/release/nimble` (or `target/release/nimble.exe` on Windows).

To install the Nimble CLI locally:

```bash
cargo install --path . --bin nimble
```

Alternatively, you can run it directly during development:

```bash
cargo run --bin nimble -- [COMMAND_ARGS]
```

## Usage

The basic syntax for creating a new project is:

```bash
nimble <PROJECT_NAME> [OPTIONS]
```

### Options

| Flag | Description | Values | Default |
|------|-------------|--------|---------|
| `-f`, `--frontend` | Frontend framework to scaffold | `angular`, `react` | None (Backend only) |
| `-c`, `--css` | CSS library for the frontend | `tailwind`, `bootstrap` | `tailwind` |
| `-d`, `--database` | Database provider to use | `postgres`, `mongodb`, `in-memory` | `in-memory` |
| `-o`, `--output` | Destination directory | Any valid path | `<PROJECT_NAME>` |

## Examples

### 1. Simple Backend API (In-Memory)
Creates a clean Rust backend using `nimble-web` with in-memory storage.
```bash
nimble my-api
```

### 2. Backend with Postgres
Creates a backend with Postgres support enabled.
```bash
nimble my-service --database postgres
```

### 3. Fullstack React + Tailwind
Creates a backend and a React frontend pre-configured with Tailwind CSS.
```bash
nimble my-app --frontend react --css tailwind
```

### 3. Fullstack Angular + Bootstrap
Creates a backend and an Angular frontend using Bootstrap.
```bash
nimble my-portal --frontend angular --css bootstrap
```

### 4. Custom Output Directory
```bash
nimble my-app --output ./projects/my-new-app
```

## Project Structure

A generated project includes:
- **`src/main.rs`**: A pre-configured Nimble application entry point.
- **`Cargo.toml`**: Project manifest with necessary dependencies.
- **`frontend/`**: (Optional) Scaffolded frontend project.
- **`Dockerfile`**: multi-stage build for production deployment.
- **`Dockerfile.dev`**: Fast-rebuild environment for development.
- **`.gitignore`**: standard rules for Rust and Node.js.

## Getting Started

Once generated, you can start your development environment:

```bash
cd <PROJECT_NAME>
# Run the backend
cargo run
```
