# cudup check

Verify cudup configuration and CUDA installation.

## Usage

```bash
cudup check
```

## Output

```
cudup check

[✓] cudup directory: /home/you/.cudup
[✓] shell integration: env file exists
[✓] installed versions: 2 (12.4.1, 11.8.0)
[✓] active version: 12.4.1
[✓] nvcc: 12.4
[✓] nvidia driver: v550.54
[✓] gpu: NVIDIA GeForce RTX 4090

All checks passed!
```

## Checks Performed

| Check | Description |
|-------|-------------|
| cudup directory | `~/.cudup` exists |
| shell integration | `~/.cudup/env` file exists |
| installed versions | Lists versions in `~/.cudup/versions/` |
| active version | `CUDA_HOME` is set and valid |
| nvcc | CUDA compiler is accessible |
| nvidia driver | Driver version via `nvidia-smi` |
| gpu | GPU detection via `nvidia-smi` |
