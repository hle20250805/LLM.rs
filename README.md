# LLM.rs

<p align="center">
  <img src="./assets/logo.png" alt="LLM.rs Logo" width="300" height="200" />
</p>

<p align="center">
  💬High-Performance Embedding Service Built with Rust
</p>

<p align="center">
  🔧Rust-based embedding service with OpenAI-compatible API and pluggable backends
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-1.0.0-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange.svg" alt="Rust Version">
  <img src="https://img.shields.io/badge/license-MIT-red.svg" alt="License">
</p>

<p align="center">
  <a href="./README-ZH.md">中文 / README-ZH.md</a>
</p>

<p align="center">
  🔥 Faster Speed | 📈 Higher Throughput | 📊 Lower Resource Usage
</p>

---

## 🔥 Performance Benchmark

LLM.rs has been benchmarked against FastAPI-based embedding services to demonstrate its performance advantages:

<p align="center">
  <img src="assets/benchmark-chart.svg" alt="LLM.rs vs FastAPI - comparison chart" width="1300" height="600" />
</p>

**Key Performance Metrics:**
- **Average latency (s):** FastAPI 1.33 → Rust 0.21 (↓84%)
- **P95 latency (s):** FastAPI 1.55 → Rust 0.24 (↓84%)
- **P99 latency (s):** FastAPI 1.91 → Rust 0.25 (↓87%)
- **Min latency (s):** FastAPI 0.52 → Rust 0.16 (↓69%)
- **Max latency (s):** FastAPI 2.59 → Rust 0.29 (↓89%)
- **Requests per Second (RPS):** FastAPI 37.19 → Rust 56.44 (↑52%)
- **Total Requests Processed:** FastAPI 2270 → Rust 3444 (↑52%)
- **Memory Usage (MB):** FastAPI 2858.12 → Rust 2420 (≈↓15%)

---

## Project Introduction

LLM.rs is a high-performance embedding service built with Rust, providing OpenAI-compatible `/v1/embeddings` endpoint, built-in queue and concurrency control, and support for pluggable inference backends (local candle or proxy to Python services).

### Main features
- OpenAI-compatible `/v1/embeddings` endpoint
- Pluggable backends (candle / Python proxy / future: vLLM)
- Built-in queue, batching, and concurrency control
- Production-oriented performance optimization with low resource usage
- Support for multiple models through configuration

---

## How to Use

### 1. Docker Usage

#### Pull Docker Image
```bash
# Pull the latest image from Docker Hub
docker pull h20260224/llmrs:latest
```

#### Run Docker Container
```bash
# Run container with default configuration
docker run -d -p 3000:3000 --name llmrs h20260224/llmrs:latest

# Run with custom environment variables
docker run -d -p 3000:3000 --name llmrs \
  -e HOST=0.0.0.0 \
  -e PORT=3000 \
  -e BACKEND_URL=http://127.0.0.1:8000 \
  -e MODEL_NAME=your-model-name \
  h20260224/llmrs:latest

# Run with config.toml file
docker run -d -p 3000:3000 --name llmrs \
  -v ./config.toml:/app/config.toml \
  h20260224/llmrs:latest
```

### 2. Startup and Model Configuration

#### Prerequisites
- Rust installed (see [Linux Build Environment](#linux-build-environment) for installation instructions)
- Backend model service running (e.g., Python service with embedding model)

#### Basic Startup
```bash
# Build and run the service
cargo build
cargo run
```

#### Custom Startup Parameters
You can configure the service using environment variables:

```bash
# Example: Start with custom port, backend URL, and model name
HOST=0.0.0.0 PORT=8080 BACKEND_URL=http://127.0.0.1:8000 MODEL_NAME=your-model-name cargo run
```

#### Configuration Options

**Environment Variables:**
- `HOST` - Server host (default: 127.0.0.1)
- `PORT` - Server port (default: 3000)
- `BACKEND_URL` - Backend model service URL (default: http://127.0.0.1:8000)
- `MODEL_NAME` - Model name to use (default: your-model-name)
- `NORMALIZE_EMBEDDINGS` - Whether to normalize embeddings (default: true)
- `BATCH_SIZE` - Batch size for processing requests (default: 32)
- `WORKERS` - Number of worker threads (default: 1)
- `QUEUE_CAPACITY` - Maximum queue capacity (default: 100)

**Using config.toml file:**
Create a `config.toml` file in the project root:
```toml
# config.toml
host = "127.0.0.1"
port = 3000
backend_url = "http://127.0.0.1:8000"
model_name = "your-model-name"
normalize_embeddings = true
batch_size = 32
workers = 1
queue_capacity = 100
```

#### Model Switching
To switch models, simply update the `MODEL_NAME` environment variable or config.toml setting:

```bash
# Example: Switch to a different model
MODEL_NAME=new-model-name cargo run
```

### 3. RESTful API Usage

#### Health Check
```bash
# Check service status
curl http://127.0.0.1:3000/health
```

Response example:
```json
{
  "status": "ok",
  "backend_url": "http://127.0.0.1:8000",
  "model_name": "your-model-name"
}
```

#### Embeddings API (OpenAI Compatible)
```bash
# Get embeddings for text
curl -X POST http://127.0.0.1:3000/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{"input":["hello", "world"], "model":"your-model-name", "encoding_format":"float"}'
```

Request format:
```json
{
  "input": ["text1", "text2"],  // Array of texts to embed
  "model": "your-model-name",  // Model name
  "encoding_format": "float"  // Output format: float or base64
}
```

#### Legacy Embed API
```bash
# Get embeddings using legacy endpoint
curl -X POST http://127.0.0.1:3000/embed \
  -H "Content-Type: application/json" \
  -d '{"texts":["hello", "world"], "normalize_embeddings":true, "batch_size":32}'
```

---

## Model Support

LLM.rs is designed to support multiple embedding models through its pluggable backend architecture:

### Current Support
- **Versatile model compatibility** - Supports a wide range of embedding models through configuration

### Future Plans
- Expanded support for additional models through configuration
- Local inference backends for more models
- Model-specific configuration options

---

## Linux Build Environment

### Recommended Usage
1. Install necessary dependencies and Rust:
   
   ```bash
   sudo apt update && sudo apt install -y build-essential curl
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   
2. Clone the repository and build the project:
   
   ```bash
   git clone <repository-url>
   cd LLM.rs
   cargo build
   ```

> Note: Linux is the preferred development and deployment environment for this project, as most server operating systems are Linux-based.

---

## Roadmap
- Local inference backend (candle/tch-rs), pluggable interface already reserved
- More complete model configuration and switching
- Performance metrics and monitoring

## Contribution
Welcome to submit issues/PRs. Please include reproduction steps and performance data (screenshots or CSV) in PRs.