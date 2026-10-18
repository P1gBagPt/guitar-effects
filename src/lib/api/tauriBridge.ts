import { invoke } from '@tauri-apps/api/core';

export interface AudioDeviceInfo {
  name: string;
  is_default: boolean;
  max_channels: number;
  sample_rates: number[];
}

export interface AudioHostInfo {
  name: string;
  host_id: string;
  input_devices: AudioDeviceInfo[];
  output_devices: AudioDeviceInfo[];
}

export interface ParameterMeta {
  id: number;
  name: string;
  min: number;
  max: number;
  default_value: number;
  unit: string;
}

export interface PedalMeta {
  id: string;
  name: string;
  category: string;
  enabled: boolean;
  parameters: ParameterMeta[];
  current_values?: Record<number, number>;
}

export interface AudioTelemetry {
  input_peak_db: number;
  input_rms_db: number;
  output_peak_db: number;
  output_rms_db: number;
  playhead_seconds: number;
  is_playing: boolean;
  is_recording: boolean;
}

export const tauriApi = {
  async getAudioHosts(): Promise<AudioHostInfo> {
    return await invoke<AudioHostInfo>('get_audio_hosts');
  },

  async startAudioEngine(
    inputDevice?: string,
    outputDevice?: string,
    sampleRate: number = 48000,
    bufferSize: number = 256
  ): Promise<void> {
    return await invoke('start_audio_engine', {
      inputDevice: inputDevice ?? null,
      outputDevice: outputDevice ?? null,
      sampleRate,
      bufferSize,
    });
  },

  async stopAudioEngine(): Promise<void> {
    return await invoke('stop_audio_engine');
  },

  async getPedalboard(): Promise<PedalMeta[]> {
    return await invoke<PedalMeta[]>('get_pedalboard');
  },

  async setPedalState(pedalId: string, enabled: boolean): Promise<boolean> {
    return await invoke<boolean>('set_pedal_state', { pedalId, enabled });
  },

  async setPedalParameter(pedalId: string, paramId: number, value: number): Promise<boolean> {
    return await invoke<boolean>('set_pedal_parameter', { pedalId, paramId, value });
  },

  async reorderPedalboard(newOrder: string[]): Promise<void> {
    return await invoke('reorder_pedalboard', { newOrder });
  },

  async setChannelVolumes(guitar: number, backing: number, master: number): Promise<void> {
    return await invoke('set_channel_volumes', { guitar, backing, master });
  },

  async getAudioTelemetry(): Promise<AudioTelemetry> {
    return await invoke<AudioTelemetry>('get_audio_telemetry');
  },

  async transportPlay(): Promise<void> {
    return await invoke('transport_play');
  },

  async transportPause(): Promise<void> {
    return await invoke('transport_pause');
  },

  async transportSeek(seconds: number): Promise<void> {
    return await invoke('transport_seek', { seconds });
  },

  async loadBackingTrackFile(filePath: string): Promise<number> {
    return await invoke<number>('load_backing_track_file', { filePath });
  },

  async loadBackingTrackBytes(bytes: number[] | Uint8Array, extension?: string): Promise<number> {
    const uint8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
    let binary = '';
    const len = uint8.byteLength;
    const chunkSize = 0x8000;
    for (let i = 0; i < len; i += chunkSize) {
      binary += String.fromCharCode.apply(null, uint8.subarray(i, Math.min(i + chunkSize, len)) as any);
    }
    const base64Data = btoa(binary);

    return await invoke<number>('load_backing_track_bytes', {
      base64Data,
      extension: extension ?? null,
    });
  },


  async startRecordingSession(filePath: string): Promise<void> {
    return await invoke('start_recording_session', { filePath });
  },

  async stopRecordingSession(): Promise<void> {
    return await invoke('stop_recording_session');
  },

  async importTabUrl(url: string): Promise<[string, string]> {
    return await invoke<[string, string]>('import_tab_url', { url });
  },
};
