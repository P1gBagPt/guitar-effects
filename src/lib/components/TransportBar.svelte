<script lang="ts">
  import { audioState } from '../stores/audioState.svelte';
  import { Play, Pause, Square, Circle, Settings2, Music, Volume2, Mic } from 'lucide-svelte';

  let { onOpenSettings = () => {} } = $props<{ onOpenSettings?: () => void }>();

  function formatTime(totalSeconds: number): string {
    const mins = Math.floor(totalSeconds / 60);
    const secs = Math.floor(totalSeconds % 60);
    const ms = Math.floor((totalSeconds % 1) * 100);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
  }

  async function handleTogglePlay() {
    await audioState.toggleTransport();
  }

  async function handleStop() {
    if (audioState.isPlaying) {
      await audioState.toggleTransport();
    }
    await audioState.seek(0);
  }

  async function handleRecord() {
    if (audioState.isRecording) {
      // Prompt user or stop
      try {
        await (window as any).__TAURI__?.invoke('stop_recording_session');
        audioState.isRecording = false;
      } catch (e) {
        console.warn(e);
      }
    } else {
      try {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const filename = `take_${timestamp}.wav`;
        await (window as any).__TAURI__?.invoke('start_recording_session', { filePath: filename });
        audioState.isRecording = true;
      } catch (e) {
        console.warn(e);
      }
    }
  }
</script>

<div class="transport-bar">
  <!-- Left: Audio Engine Status & Settings -->
  <div class="transport-section">
    <button
      class="settings-btn"
      onclick={onOpenSettings}
      title="Audio Settings & Drivers"
    >
      <Settings2 size={16} />
      <span class="engine-badge" class:online={audioState.isEngineRunning}>
        {audioState.isEngineRunning ? 'ONLINE' : 'OFFLINE'}
      </span>
      <span class="latency-info">
        {audioState.bufferSize} spls ({(audioState.bufferSize / (audioState.sampleRate / 1000)).toFixed(1)}ms)
      </span>
    </button>
  </div>

  <!-- Center: Main Transport Controls & Clock -->
  <div class="transport-section center-transport">
    <div class="transport-buttons">
      <!-- Play / Pause -->
      <button
        class="transport-btn play-btn"
        class:active={audioState.isPlaying}
        onclick={handleTogglePlay}
        title={audioState.isPlaying ? 'Pause Transport (Space)' : 'Play Transport (Space)'}
      >
        {#if audioState.isPlaying}
          <Pause size={18} />
        {:else}
          <Play size={18} />
        {/if}
      </button>

      <!-- Stop -->
      <button
        class="transport-btn"
        onclick={handleStop}
        title="Stop & Return to Zero"
      >
        <Square size={16} />
      </button>

      <!-- Record -->
      <button
        class="transport-btn record-btn"
        class:recording={audioState.isRecording}
        onclick={handleRecord}
        title="24-bit Lock-Free WAV Disk Record"
      >
        <Circle size={15} fill={audioState.isRecording ? '#ff1744' : 'transparent'} />
      </button>
    </div>

    <!-- Timecode Clock Display -->
    <div class="timecode-display">
      <span class="time-readout">{formatTime(audioState.playheadSeconds)}</span>
    </div>

    <!-- Tempo & Metronome -->
    <div class="tempo-group">
      <div class="bpm-box">
        <span class="bpm-label">BPM</span>
        <input
          type="number"
          min="40"
          max="260"
          bind:value={audioState.bpm}
          class="bpm-input"
        />
      </div>

      <button
        class="metro-btn"
        class:active={audioState.metronomeEnabled}
        onclick={() => (audioState.metronomeEnabled = !audioState.metronomeEnabled)}
        title="Toggle Metronome Click"
      >
        <Music size={14} />
      </button>
    </div>
  </div>

  <!-- Right: Input Monitoring Indicator -->
  <div class="transport-section right-section">
    <div class="monitor-tag">
      <Mic size={14} />
      <span>{audioState.selectedInput ? audioState.selectedInput.slice(0, 20) : 'Guitar DI'}</span>
    </div>
  </div>
</div>

<style>
  .transport-bar {
    height: 54px;
    background-color: var(--bg-surface, #1a1c22);
    border-bottom: 1px solid var(--border-color, #2e333d);
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 18px;
    user-select: none;
    z-index: 10;
  }

  .transport-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .center-transport {
    gap: 20px;
  }

  .settings-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #232731;
    border: 1px solid #323846;
    color: #e0e4eb;
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s ease;
  }

  .settings-btn:hover {
    background: #2c3240;
    border-color: #444c5e;
  }

  .engine-badge {
    font-weight: 700;
    font-size: 10px;
    padding: 2px 5px;
    border-radius: 3px;
    background: #3a3f4d;
    color: #9aa0a6;
  }

  .engine-badge.online {
    background: rgba(46, 204, 113, 0.2);
    color: #2ecc71;
    border: 1px solid rgba(46, 204, 113, 0.4);
  }

  .latency-info {
    font-family: monospace;
    font-size: 11px;
    color: #8c93a0;
  }

  .transport-buttons {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #14161b;
    padding: 4px 6px;
    border-radius: 8px;
    border: 1px solid #282c37;
  }

  .transport-btn {
    background: #20242e;
    border: 1px solid #303644;
    color: #f0f2f5;
    width: 36px;
    height: 32px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.1s ease;
  }

  .transport-btn:hover {
    background: #2a303e;
  }

  .transport-btn:active {
    transform: scale(0.96);
  }

  .play-btn.active {
    background: var(--accent-orange, #ff6b35);
    border-color: #ff8c5a;
    color: #fff;
    box-shadow: 0 0 10px rgba(255, 107, 53, 0.5);
  }

  .record-btn.recording {
    background: #d50000;
    border-color: #ff1744;
    color: #fff;
    animation: pulse 1s infinite alternate;
  }

  @keyframes pulse {
    from {
      box-shadow: 0 0 4px #ff1744;
    }
    to {
      box-shadow: 0 0 14px #ff1744;
    }
  }

  .timecode-display {
    background: #111317;
    border: 1px solid #282c37;
    border-radius: 6px;
    padding: 6px 14px;
    display: flex;
    align-items: center;
  }

  .time-readout {
    font-family: 'Courier New', Courier, monospace;
    font-size: 16px;
    font-weight: 700;
    color: #00d2ff;
    letter-spacing: 1px;
    text-shadow: 0 0 8px rgba(0, 210, 255, 0.4);
  }

  .tempo-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .bpm-box {
    display: flex;
    align-items: center;
    background: #1f232c;
    border: 1px solid #303746;
    border-radius: 6px;
    padding: 2px 8px;
    gap: 6px;
  }

  .bpm-label {
    font-size: 10px;
    font-weight: 700;
    color: #8c93a0;
  }

  .bpm-input {
    width: 48px;
    background: transparent;
    border: none;
    color: #fff;
    font-size: 13px;
    font-weight: 700;
    outline: none;
    font-family: monospace;
  }

  .metro-btn {
    background: #1f232c;
    border: 1px solid #303746;
    color: #8c93a0;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .metro-btn.active {
    background: rgba(247, 160, 40, 0.2);
    border-color: #f7a028;
    color: #f7a028;
    box-shadow: 0 0 8px rgba(247, 160, 40, 0.3);
  }

  .monitor-tag {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: #9aa0a6;
    background: #1f232c;
    padding: 5px 10px;
    border-radius: 6px;
    border: 1px solid #2d3340;
  }
</style>
