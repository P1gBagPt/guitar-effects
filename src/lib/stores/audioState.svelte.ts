import { tauriApi, type AudioHostInfo, type AudioTelemetry } from '../api/tauriBridge';

class AudioState {
  hostInfo = $state<AudioHostInfo | null>(null);
  selectedInput = $state<string>('');
  selectedOutput = $state<string>('');
  sampleRate = $state<number>(48000);
  bufferSize = $state<number>(256);
  isEngineRunning = $state<boolean>(false);
  errorMessage = $state<string | null>(null);

  // Volumes
  guitarVolume = $state<number>(1.0);
  backingVolume = $state<number>(1.0);
  masterVolume = $state<number>(1.0);

  // Transport & Backing Track
  isPlaying = $state<boolean>(false);
  isRecording = $state<boolean>(false);
  playheadSeconds = $state<number>(0);
  durationSeconds = $state<number>(0);
  backingTrackTitle = $state<string>('');
  bpm = $state<number>(120);
  metronomeEnabled = $state<boolean>(false);

  // Decibel Meters
  inputPeakDb = $state<number>(-96);
  inputRmsDb = $state<number>(-96);
  outputPeakDb = $state<number>(-96);
  outputRmsDb = $state<number>(-96);

  private telemetryTimer: number | null = null;

  async init() {
    try {
      this.hostInfo = await tauriApi.getAudioHosts();
      if (this.hostInfo) {
        const defIn = this.hostInfo.input_devices.find((d) => d.is_default);
        if (defIn) this.selectedInput = defIn.name;
        const defOut = this.hostInfo.output_devices.find((d) => d.is_default);
        if (defOut) this.selectedOutput = defOut.name;
      }
      try {
        await this.startEngine();
      } catch (err) {
        console.warn('Initial engine start will retry on transport play:', err);
      }
      this.startTelemetryLoop();
    } catch (err) {
      console.warn('Backend not yet ready or running in browser preview:', err);
    }
  }

  async startEngine() {
    try {
      this.errorMessage = null;
      await tauriApi.startAudioEngine(
        this.selectedInput || undefined,
        this.selectedOutput || undefined,
        this.sampleRate,
        this.bufferSize
      );
      this.isEngineRunning = true;
    } catch (err: any) {
      this.errorMessage = String(err);
      this.isEngineRunning = false;
      throw err;
    }
  }

  async stopEngine() {
    try {
      await tauriApi.stopAudioEngine();
      this.isEngineRunning = false;
    } catch (err) {
      console.error('Failed to stop engine:', err);
    }
  }

  async toggleTransport() {
    if (this.isPlaying) {
      await tauriApi.transportPause();
      this.isPlaying = false;
    } else {
      if (!this.isEngineRunning) {
        try {
          await this.startEngine();
        } catch (e) {
          console.error('Auto-start engine error:', e);
        }
      }
      if (this.durationSeconds > 0 && this.playheadSeconds >= this.durationSeconds - 0.1) {
        await this.seek(0);
      }
      await tauriApi.transportPlay();
      this.isPlaying = true;
    }
  }


  async seek(seconds: number) {
    this.playheadSeconds = Math.max(0, seconds);
    await tauriApi.transportSeek(seconds);
  }

  async loadBackingTrack(name: string, source: string | ArrayBuffer) {
    try {
      let dur = 0;
      if (typeof source === 'string' && source.includes('/') || (typeof source === 'string' && source.includes('\\'))) {
        dur = await tauriApi.loadBackingTrackFile(source);
      } else {
        // In-memory ArrayBuffer from webview file picker
        const buffer = source instanceof ArrayBuffer ? source : new ArrayBuffer(0);
        const bytes = new Uint8Array(buffer);
        const ext = name.split('.').pop()?.toLowerCase();
        dur = await tauriApi.loadBackingTrackBytes(bytes, ext);
      }
      this.durationSeconds = dur;
      this.backingTrackTitle = name;
      return dur;
    } catch (err) {
      console.error('Failed to load backing track:', err);
      throw err;
    }
  }

  async setVolumes(guitar: number, backing: number, master: number) {
    this.guitarVolume = guitar;
    this.backingVolume = backing;
    this.masterVolume = master;
    await tauriApi.setChannelVolumes(guitar, backing, master);
  }

  private startTelemetryLoop() {
    if (this.telemetryTimer) return;
    this.telemetryTimer = window.setInterval(async () => {
      try {
        const tel: AudioTelemetry = await tauriApi.getAudioTelemetry();
        this.inputPeakDb = tel.input_peak_db;
        this.inputRmsDb = tel.input_rms_db;
        this.outputPeakDb = tel.output_peak_db;
        this.outputRmsDb = tel.output_rms_db;
        this.playheadSeconds = tel.playhead_seconds;
        this.isPlaying = tel.is_playing;
        this.isRecording = tel.is_recording;
      } catch {
        // Fallback or simulated meters in preview mode
      }
    }, 33); // ~30-60 FPS UI meter refresh
  }
}

export const audioState = new AudioState();
