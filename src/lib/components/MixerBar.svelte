<script lang="ts">
  import { audioState } from '../stores/audioState.svelte';
  import { Volume2, Mic, Music, Disc } from 'lucide-svelte';

  function dbToHeight(db: number): number {
    // Map -60 dB to 0 dB into 0% to 100%
    const clamped = Math.max(-60, Math.min(0, db));
    return ((clamped + 60) / 60) * 100;
  }

  function handleGuitarVol(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    audioState.setVolumes(val, audioState.backingVolume, audioState.masterVolume);
  }

  function handleBackingVol(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    audioState.setVolumes(audioState.guitarVolume, val, audioState.masterVolume);
  }

  function handleMasterVol(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    audioState.setVolumes(audioState.guitarVolume, audioState.backingVolume, val);
  }
</script>

<div class="mixer-bar">
  <div class="mixer-header">
    <span class="mixer-title">MIXER BUS</span>
  </div>

  <div class="channels-strip">
    <!-- Channel 1: Guitar DI -->
    <div class="channel-card">
      <div class="channel-info">
        <Mic size={14} class="chan-icon" />
        <span class="channel-name">GUITAR</span>
        <div class="mini-chan-selector">
          <button
            class="mini-btn"
            class:active={audioState.inputChannel === 0}
            onclick={() => audioState.setInputChannel(0)}
            title="Focusrite Input 1 (Left / Mic 1 / Inst 1)"
          >
            IN 1
          </button>
          <button
            class="mini-btn"
            class:active={audioState.inputChannel === 1}
            onclick={() => audioState.setInputChannel(1)}
            title="Focusrite Input 2 (Right / Guitar / Inst 2)"
          >
            IN 2
          </button>
          <button
            class="mini-btn"
            class:active={audioState.inputChannel === 2}
            onclick={() => audioState.setInputChannel(2)}
            title="Both Inputs (Auto / Sum)"
          >
            ALL
          </button>
        </div>
      </div>

      <!-- Meter & Fader Pair -->
      <div class="fader-meter-row">
        <!-- Vertical Peak/RMS Meter -->
        <div class="meter-column">
          <div class="meter-track">
            <div
              class="meter-fill rms-fill"
              style="height: {dbToHeight(audioState.inputRmsDb)}%;"
            ></div>
            <div
              class="meter-fill peak-fill"
              style="height: {dbToHeight(audioState.inputPeakDb)}%;"
            ></div>
            {#if audioState.inputPeakDb >= -0.5}
              <div class="clip-light active"></div>
            {:else}
              <div class="clip-light"></div>
            {/if}
          </div>
          <span class="meter-db">{audioState.inputPeakDb.toFixed(0)}</span>
        </div>

        <!-- Vertical Slider -->
        <div class="slider-container">
          <input
            type="range"
            min="0"
            max="1.5"
            step="0.01"
            value={audioState.guitarVolume}
            oninput={handleGuitarVol}
            class="vertical-slider"
          />
        </div>
      </div>
      <span class="vol-label">{(audioState.guitarVolume * 100).toFixed(0)}%</span>
    </div>

    <!-- Channel 2: Backing Track -->
    <div class="channel-card">
      <div class="channel-info">
        <Music size={14} class="chan-icon" />
        <span class="channel-name">BACKING</span>
      </div>

      <div class="fader-meter-row">
        <div class="meter-column">
          <div class="meter-track">
            <div
              class="meter-fill rms-fill backing"
              style="height: {audioState.isPlaying ? dbToHeight(audioState.outputRmsDb - 3) : 0}%;"
            ></div>
            <div
              class="meter-fill peak-fill backing"
              style="height: {audioState.isPlaying ? dbToHeight(audioState.outputPeakDb - 2) : 0}%;"
            ></div>
          </div>
          <span class="meter-db">{audioState.isPlaying ? audioState.outputPeakDb.toFixed(0) : '-inf'}</span>
        </div>

        <div class="slider-container">
          <input
            type="range"
            min="0"
            max="1.5"
            step="0.01"
            value={audioState.backingVolume}
            oninput={handleBackingVol}
            class="vertical-slider"
          />
        </div>
      </div>
      <span class="vol-label">{(audioState.backingVolume * 100).toFixed(0)}%</span>
    </div>

    <!-- Channel 3: Master Out -->
    <div class="channel-card master-channel">
      <div class="channel-info">
        <Disc size={14} class="chan-icon" />
        <span class="channel-name">MASTER</span>
      </div>

      <div class="fader-meter-row">
        <div class="meter-column">
          <div class="meter-track">
            <div
              class="meter-fill rms-fill master"
              style="height: {dbToHeight(audioState.outputRmsDb)}%;"
            ></div>
            <div
              class="meter-fill peak-fill master"
              style="height: {dbToHeight(audioState.outputPeakDb)}%;"
            ></div>
            {#if audioState.outputPeakDb >= -0.1}
              <div class="clip-light active"></div>
            {:else}
              <div class="clip-light"></div>
            {/if}
          </div>
          <span class="meter-db">{audioState.outputPeakDb.toFixed(0)}</span>
        </div>

        <div class="slider-container">
          <input
            type="range"
            min="0"
            max="1.5"
            step="0.01"
            value={audioState.masterVolume}
            oninput={handleMasterVol}
            class="vertical-slider"
          />
        </div>
      </div>
      <span class="vol-label">{(audioState.masterVolume * 100).toFixed(0)}%</span>
    </div>
  </div>
</div>

<style>
  .mixer-bar {
    width: 100%;
    height: 100%;
    background-color: var(--bg-panel, #22252c);
    border-left: 1px solid var(--border-color, #2e333d);
    display: flex;
    flex-direction: column;
    padding: 14px;
    gap: 12px;
    user-select: none;
    box-sizing: border-box;
  }

  .mixer-header {
    border-bottom: 1px solid #2e333d;
    padding-bottom: 6px;
  }

  .mixer-title {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 1px;
    color: #8c93a0;
  }

  .channels-strip {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    height: 100%;
  }

  .channel-card {
    flex: 1;
    background: #191b20;
    border: 1px solid #282c35;
    border-radius: 8px;
    padding: 10px 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
  }

  .channel-card.master-channel {
    background: #1d1e25;
    border-color: #3b3f4d;
  }

  .channel-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .channel-name {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: #abb2bf;
  }

  .mini-chan-selector {
    display: flex;
    gap: 2px;
    background: #111317;
    padding: 2px;
    border-radius: 4px;
    border: 1px solid #282c35;
  }

  .mini-btn {
    background: transparent;
    border: none;
    color: #6a7180;
    font-size: 8px;
    font-weight: 700;
    padding: 2px 4px;
    border-radius: 2px;
    cursor: pointer;
    line-height: 1;
    transition: all 0.12s ease;
  }

  .mini-btn:hover {
    color: #fff;
  }

  .mini-btn.active {
    background: var(--accent-orange, #ff6b35);
    color: #fff;
  }

  .fader-meter-row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 180px;
    margin: 10px 0;
  }

  .meter-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    gap: 4px;
  }

  .meter-track {
    width: 8px;
    flex: 1;
    background: #111317;
    border: 1px solid #282c35;
    border-radius: 4px;
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  .meter-fill {
    width: 100%;
    position: absolute;
    bottom: 0;
    transition: height 0.04s ease-out;
  }

  .peak-fill {
    background: linear-gradient(0deg, #2ecc71 70%, #f1c40f 90%, #e74c3c 100%);
    opacity: 0.85;
  }

  .rms-fill {
    background: #00d2ff;
    opacity: 0.5;
  }

  .clip-light {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 4px;
    background: #333;
  }

  .clip-light.active {
    background: #ff1744;
    box-shadow: 0 0 6px #ff1744;
  }

  .meter-db {
    font-size: 9px;
    font-family: monospace;
    color: #6a7180;
  }

  .slider-container {
    height: 100%;
    display: flex;
    align-items: center;
  }

  .vertical-slider {
    writing-mode: vertical-lr;
    direction: rtl;
    width: 18px;
    height: 100%;
    accent-color: var(--accent-orange, #ff6b35);
    cursor: pointer;
  }

  .vol-label {
    font-size: 10px;
    font-family: monospace;
    color: #00d2ff;
  }
</style>
