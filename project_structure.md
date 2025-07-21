# Project Structure - MLX Transcription Tool

## Core Components
```
ava/
├── mlx_cli.py              # Core CLI transcription engine
├── streamlit_app.py        # Web UI (to be created)
├── requirements.txt        # Web dependencies (to be created)
├── output/                 # Default output directory
│   └── [video_name]/       # Per-video results
└── README.md              # Usage guide
```

## Architecture
- **Core Engine**: `mlx_cli.py` - Handles all transcription logic
- **Web UI**: `streamlit_app.py` - User interface layer only
- **Shared**: JSON checkpoint system for both CLI and web

## Usage Paths
1. **CLI**: `python mlx_cli.py video.mp4`
2. **Web**: `streamlit run streamlit_app.py`

## Key Design Principles
- **Separation of Concerns**: Core logic in CLI, UI in web
- **Shared Data Format**: JSON checkpoints work with both
- **No Duplication**: Web UI calls CLI functions directly