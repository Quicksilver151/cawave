# cawave


> [!CAUTION]
> **Vibe Coded Disclaimer**: This project was built with a high degree of agentic assistance. It was iterated through heavily but the overall project is still fully AI generated.

**cawave** is a lightweight, high-performance terminal audio visualizer built in Rust. It uses [cava](https://github.com/karlstav/cava) as its audio processing backend and implements a real-time wave physics simulation to create smooth, organic motion that goes beyond traditional bar visualizers.

![demo](https://raw.githubusercontent.com/Quicksilver151/cawave/refs/heads/assets/demo-small.gif)

## Features

- **Physics-Based Animation**: Unlike standard visualizers, `cawave` uses a 1D wave equation simulation. Audio transients act as "impulses" that trigger ripples traveling across the display.
- **Fluid Motion**: High-framerate simulation (default 60fps) ensures buttery-smooth transitions.
- **Configurable Aesthetics**: Support for solid colors, gradients, and mirrored modes.
- **Cava Backend**: Leverages the robust and widely supported `cava` library for accurate FFT and audio capture.
- **Minimalist TUI**: Built with `ratatui` for a clean, efficient terminal interface.

## Prerequisites

- **cava**: Must be installed and available in your PATH. 
- **Rust**: Version 1.70 or newer is required to build the project.
- **Linux**: Currently optimized for Linux (requires `mkfifo` and standard unix pipes).

## Installation

1. **Install cava**:
   ```bash
   # Debian/Ubuntu
   sudo apt install cava
   # Arch Linux
   sudo pacman -S cava
   ```

2. **Build cawave**:
   ```bash
   git clone https://github.com/yourusername/cawave.git
   cd cawave
   cargo build --release
   ```

3. **Run**:
   ```bash
   ./target/release/cawave
   ```

## Usage

Launch the visualizer with default settings:
```bash
cawave
```

### Controls
- `q`: Quit the application.
- `--debug` / `-d`: Show raw cava input buckets for troubleshooting.
- `--config` / `-c`: Specify a custom path to a TOML config file.

## Configuration

`cawave` automatically creates a default configuration file at `~/.config/cawave/config.toml` (following XDG standards).

### Key Parameters

| Section   | Parameter           | Default    | Description |
|-----------|---------------------|------------|-------------|
| `general` | `framerate`         | `60`       | Target FPS for both physics and rendering. |
| `input`   | `driver_bars`       | `16`       | Number of frequency buckets requested from cava. |
| `physics` | `wave_speed`        | `0.3`      | How fast waves propagate across the screen. |
| `physics` | `damping`           | `0.97`     | Energy retention per frame (closer to 1.0 = longer decay). |
| `physics` | `points`            | `64`       | Simulation resolution. Higher = smoother wave. |
| `output`  | `mirror`            | `true`     | Mirror the wave about the center. |
| `output`  | `visual_gain`       | `1.0`      | Global amplitude multiplier. |
| `output`  | `color_mode`        | `gradient` | `solid` or `gradient`. |

## The Physics Model

Unlike traditional visualizers that map frequency amplitude directly to bar height, `cawave` treats audio input as a series of physical impulses. 

1. **Impulse Detection**: The difference (delta) between audio frames is calculated.
2. **Dynamic Force**: If the delta exceeds a threshold, a downward velocity is applied to the wave at the corresponding frequency position.
3. **Wave Equation**: The simulation solves a discrete wave equation every frame:
   `velocity[i] += wave_speed * (left + right - 2 * center)`
4. **Volume Conservation**: The `compressibility` parameter ensures displaced "liquid" eventually settles back to the mean.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
