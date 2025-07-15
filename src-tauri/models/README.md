# Whisper Models

To use the MLX Whisper functionality, you need to download a Whisper model.

## Recommended Models

Download one of these models and place it in this directory:

### Base Models (English)
- **ggml-base.en.bin** (74MB) - Good balance of speed and accuracy
  ```bash
  curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin -o models/ggml-base.en.bin
  ```

### Small Models (English)
- **ggml-small.en.bin** (244MB) - Better accuracy, slightly slower
  ```bash
  curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin -o models/ggml-small.en.bin
  ```

### Multilingual Models
- **ggml-base.bin** (74MB) - Supports multiple languages
  ```bash
  curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -o models/ggml-base.bin
  ```

## Usage

After downloading a model, update the `model_path` variable in `src/lib.rs` to match your chosen model.

For example:
```rust
let model_path = "models/ggml-base.en.bin";
```