# 🎸 Guitar Workstation Pro

A high-performance, ultra-low latency, cross-platform desktop workstation for guitarists. Built with **Tauri 2.0**, **Rust 2021**, **Svelte 5**, and **AlphaTab**.

Featuring a lock-free real-time DSP engine, modular stompbox pedalboard with acoustic IR cabinet convolution, full Guitar Pro tablature notation viewer, backing track player with dynamic time-stretching, and a 24-bit studio WAV recorder.

---

## ⚡ Key Highlights & Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Tauri 2.0 / Svelte 5 UI                         │
│  [Transport Bar]     [AlphaTab Score Viewer]     [Modular Pedalboard]  │
└────────────────────────────────────┬───────────────────────────────────┘
                                     │ Lock-Free IPC RingBuffers
┌────────────────────────────────────▼───────────────────────────────────┐
│                    Native Rust Real-Time Audio Engine                  │
│                                                                        │
│   CPAL Input Stream (Direct Audio Interface / ASIO / WASAPI)          │
│     │                                                                  │
│     ▼                                                                  │
│   [ DSP Pedalboard Chain (Zero Allocation, Lock-Free Processing) ]     │
│     ├─ Noise Gate (Downward Expander)                                  │
│     ├─ Polyphase 2x Oversampled Overdrive / Distortion                 │
│     ├─ Hermite Interpolated Analog Delay + Tape Flutter                │
│     ├─ Schroeder-Moorer Algorithmic Reverb                             │
│     ├─ FIR Acoustic Cabinet Impulse Response Convolution               │
│     └─ 0 dBFS Soft Brickwall Limiter                                   │
│     │                                                                  │
│     ├──► [ Lock-Free 24-bit WAV Disk Recorder (hound) ]                │
│     │                                                                  │
│     ▼                                                                  │
│   [ Real-Time 3-Bus Channel Mixer (Guitar DI + Backing + Master) ]     │
│     ▲                                                                  │
│     │ Symphonia Decoded Audio + Rubato Sinc Resampler                  │
│     └─ Backing Track Audio (MP3 / WAV / FLAC / OGG)                    │
│     │                                                                  │
│     ▼                                                                  │
│   CPAL Output Stream (Monitors / Headphones)                           │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🎛️ Features

### 1. Real-Time Lock-Free DSP Pedalboard
- **Zero-Allocation Audio Thread**: Real-time signal loop with atomic volume/telemetry pointers and lock-free thread coordination.
- **5 Modular Stompboxes**:
  - **Noise Gate**: Downward expander with adjustable threshold (-80 dB to 0 dB), attack, hold, and release.
  - **Vintage Overdrive**: Polyphase 2x oversampled non-linear waveshaping distortion with active tone stack filter.
  - **Analog Delay**: Fractional tape delay with 4-point Hermite cubic interpolation, high-frequency damping, and wow/flutter modulation.
  - **Ambient Reverb**: Schroeder-Moorer algorithmic reverberator with 4 parallel comb filters, 2 all-pass diffusers, room size, and high-frequency decay damping.
  - **Cabinet Simulator (IR)**: Real-time FIR acoustic convolution supporting loaded WAV impulse responses and factory vintage cabinets (Celestion Vintage 30, Fender Twin Reverb, Tweed 4x10).
  - **Output Protection**: 0 dBFS soft brickwall limiter preventing digital clipping.
- **Dynamic Reordering**: Move any pedal left or right on the signal chain on the fly.
- **Factory Presets**: Instant switching between *Clean Ambient*, *Blues Overdrive*, *Hard Rock Lead*, and *Heavy Metal Rhythm*.

### 2. AlphaTab Multi-Track Notation & Tab Viewer
- **File Format Support**: Native rendering for `.gp`, `.gpx`, `.gp5`, `.gp4`, `.gp3`, `.musicxml`, and `.cap`.
- **Multi-Track Switching**: Instant instrument track switching (Lead Guitar, Rhythm Guitar, Bass).
- **Web Importer**: Direct URL tab fetcher bypassing web CORS restrictions via native Rust backend.
- **Smooth Navigation**: Zoom controls, speed scaling (0.75x, 1.0x, 1.25x), and live demo riff loader.
- **Continuous Responsive Page Layout**: High-contrast engraving rendered on a clean paper surface without measure cutoffs.

### 3. Studio Mixer & Transport
- **3-Bus Hardware Mixer**: Dedicated volume faders and live peak / RMS decibel meters for **Guitar DI**, **Backing Track**, and **Master Out**.
- **Transport Bar**: Play/Stop controls, spacebar shortcut, clock timecode display, and metronome click generator.
- **24-bit WAV Recorder**: Record your jam sessions directly to disk without impacting real-time audio thread latency.

### 4. Resizable Workstation Layout
- **Vertical & Horizontal Splitters**: Drag the splitter bars to resize the pedalboard rack, tab viewer, and mixer strip to fit your screen.

---

## 🚀 Getting Started

### Prerequisites

1. **Node.js** (v18 or higher) & **npm**
2. **Rust Toolchain**:
   - `rustup` with target `stable-x86_64-pc-windows-gnu` or `stable-x86_64-pc-windows-msvc`.
   - On Windows with GNU toolchain: `MartinStorsjo.LLVM-MinGW.UCRT`.

---

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/P1gBagPt/guitar-effects.git
   cd guitar-effects
   ```

2. **Install frontend dependencies**:
   ```bash
   npm install
   ```

3. **Run in Development Mode**:
   ```bash
   npm run tauri dev
   ```
   *(Or run frontend only for UI preview: `npm run dev`)*

4. **Build for Production**:
   ```bash
   npm run tauri build
   ```
   The optimized native desktop executable will be built in `src-tauri/target/release/`.

---

## 🛠️ Project Structure

```
GuitarEffects/
├── src/                               # Frontend (Svelte 5 + TypeScript + Vite)
│   ├── lib/
│   │   ├── api/                       # Type-safe Tauri IPC bindings
│   │   │   └── tauriBridge.ts
│   │   ├── components/                # Modular UI Components
│   │   │   ├── AudioSettingsModal.svelte
│   │   │   ├── Knob.svelte            # Rotary potentiometer with arc tracks
│   │   │   ├── MixerBar.svelte        # Faders & Peak/RMS meters
│   │   │   ├── Pedal.svelte           # Die-cast stompbox chassis & LED
│   │   │   ├── Pedalboard.svelte      # Signal chain rack & presets
│   │   │   ├── TabViewer.svelte       # AlphaTab music sheet viewer
│   │   │   └── TransportBar.svelte    # Transport controls & clock
│   │   └── stores/                    # Svelte 5 Rune State Stores
│   │       ├── audioState.svelte.ts
│   │       └── pedalboardState.svelte.ts
│   ├── App.svelte                     # Root workstation container & splitters
│   └── main.ts
│
├── src-tauri/                         # Native Backend (Rust 2021)
│   ├── src/
│   │   ├── audio/                     # Hardware I/O & Recording
│   │   │   ├── device.rs              # cpal host & device enumerator
│   │   │   ├── mod.rs                 # Audio stream coordinator & telemetry
│   │   │   ├── recorder.rs            # Lock-free 24-bit WAV recorder (hound)
│   │   │   └── ringbuffer.rs          # Lock-free SPSC circular buffers
│   │   ├── decoder/                   # Audio File Decoder & Resampling
│   │   │   └── mod.rs                 # symphonia decoder + rubato sinc resampler
│   │   ├── dsp/                       # DSP Algorithms & FX
│   │   │   ├── biquad.rs              # Digital IIR biquad filters
│   │   │   ├── cab_ir.rs              # FIR speaker IR convolution
│   │   │   ├── chain.rs               # Pedalboard container & brickwall limiter
│   │   │   ├── delay.rs               # Hermite cubic tape delay & flutter
│   │   │   ├── drive.rs               # Polyphase 2x oversampled overdrive
│   │   │   ├── gate.rs                # Downward expander noise gate
│   │   │   ├── mod.rs
│   │   │   └── reverb.rs              # Schroeder-Moorer algorithmic reverb
│   │   ├── tabs/                      # Tablature Importer
│   │   │   └── scraper.rs             # Native URL tab scraper & Base64 encoder
│   │   ├── lib.rs                     # Tauri IPC command registration
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── public/                            # Bundled Assets (SMuFL Bravura WebFonts)
├── package.json
└── vite.config.ts
```

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
