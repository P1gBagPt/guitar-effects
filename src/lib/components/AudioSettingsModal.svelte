<script lang="ts">
  import { audioState } from '../stores/audioState.svelte';
  import { X, Cpu, Volume2, Mic, Activity, AlertCircle, Sliders } from 'lucide-svelte';

  let { onClose = () => {} } = $props<{ onClose?: () => void }>();

  let isStarting = $state(false);

  const latencyMs = $derived.by(() => {
    return (audioState.bufferSize / (audioState.sampleRate / 1000)).toFixed(2);
  });

  async function handleToggleEngine() {
    isStarting = true;
    try {
      if (audioState.isEngineRunning) {
        await audioState.stopEngine();
      } else {
        await audioState.startEngine();
      }
    } catch (e) {
      console.error(e);
    } finally {
      isStarting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="modal-backdrop"
  onclick={onClose}
  onkeydown={(e) => e.key === 'Escape' && onClose()}
  tabindex="-1"
  role="presentation"
>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-window"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="0"
  >
    <!-- Modal Header -->
    <div class="modal-header">
      <div class="header-title">
        <Cpu size={18} />
        <h2 id="modal-title">Audio Hardware & Driver Settings</h2>
      </div>
      <button class="close-btn" onclick={onClose} aria-label="Close settings">
        <X size={18} />
      </button>
    </div>

    <!-- Error Banner if any -->
    {#if audioState.errorMessage}
      <div class="error-banner">
        <AlertCircle size={16} />
        <span>{audioState.errorMessage}</span>
      </div>
    {/if}

    <!-- Modal Body -->
    <div class="modal-body">
      <!-- Driver Host -->
      <div class="setting-row">
        <span class="setting-title">Driver Architecture</span>
        <div class="readonly-badge">
          {audioState.hostInfo?.name ?? 'WASAPI Exclusive / cpal'}
        </div>
      </div>

      <!-- Input Device -->
      <div class="setting-row">
        <label for="input-select">
          <Mic size={14} /> Input Device (Guitar DI)
        </label>
        <select id="input-select" bind:value={audioState.selectedInput} class="form-select">
          <option value="">Default System Audio In</option>
          {#each audioState.hostInfo?.input_devices ?? [] as dev}
            <option value={dev.name}>
              {dev.name} {dev.is_default ? '(Default)' : ''}
            </option>
          {/each}
        </select>
      </div>

      <!-- Guitar Input Channel / Routing -->
      <div class="setting-row">
        <label for="input-channel-group">
          <Sliders size={14} /> Guitar Input Jack / Channel
        </label>
        <div class="channel-toggle-group" id="input-channel-group">
          <button
            type="button"
            class="chan-btn"
            class:active={audioState.inputChannel === 0}
            onclick={() => audioState.setInputChannel(0)}
          >
            Input 1 (Left)
          </button>
          <button
            type="button"
            class="chan-btn"
            class:active={audioState.inputChannel === 1}
            onclick={() => audioState.setInputChannel(1)}
          >
            Input 2 (Right / Guitar)
          </button>
          <button
            type="button"
            class="chan-btn"
            class:active={audioState.inputChannel === 2}
            onclick={() => audioState.setInputChannel(2)}
          >
            Both (Auto / Mix)
          </button>
        </div>
        <span class="helper-text">
          Focusrite Scarlett: Placed in Inst / Jack 2? Select <strong>Input 2</strong> or <strong>Both</strong>.
        </span>
      </div>

      <!-- Live Input Meter / Signal Test Card -->
      <div class="signal-test-card">
        <div class="signal-test-header">
          <span class="signal-test-title">Real-Time DI Signal Activity (Pluck Guitar)</span>
          <span class="signal-status-pill" class:active={audioState.inputActive}>
            {audioState.inputActive ? 'Input Capturing' : 'Input Inactive'}
          </span>
        </div>
        <div class="meter-bars-row">
          <div class="meter-bar-item">
            <span class="bar-name">In 1</span>
            <div class="meter-h-track">
              <div
                class="meter-h-fill"
                style="width: {Math.max(0, Math.min(100, ((audioState.inputCh1PeakDb + 60) / 60) * 100))}%;"
              ></div>
            </div>
            <span class="bar-val">{audioState.inputCh1PeakDb.toFixed(0)} dB</span>
          </div>
          <div class="meter-bar-item">
            <span class="bar-name">In 2</span>
            <div class="meter-h-track">
              <div
                class="meter-h-fill"
                style="width: {Math.max(0, Math.min(100, ((audioState.inputCh2PeakDb + 60) / 60) * 100))}%;"
              ></div>
            </div>
            <span class="bar-val">{audioState.inputCh2PeakDb.toFixed(0)} dB</span>
          </div>
        </div>
        {#if audioState.inputError}
          <div class="input-warn-text">
            Notice: {audioState.inputError}
          </div>
        {/if}
      </div>

      <!-- Output Device -->
      <div class="setting-row">
        <label for="output-select">
          <Volume2 size={14} /> Output Device (Monitors / Headphones)
        </label>
        <select id="output-select" bind:value={audioState.selectedOutput} class="form-select">
          <option value="">Default System Audio Out</option>
          {#each audioState.hostInfo?.output_devices ?? [] as dev}
            <option value={dev.name}>
              {dev.name} {dev.is_default ? '(Default)' : ''}
            </option>
          {/each}
        </select>
      </div>

      <!-- Sample Rate -->
      <div class="setting-row">
        <label for="rate-select">Sample Rate</label>
        <select id="rate-select" bind:value={audioState.sampleRate} class="form-select">
          <option value={44100}>44,100 Hz (Standard Audio CD)</option>
          <option value={48000}>48,000 Hz (Professional Studio Standard)</option>
          <option value={88200}>88,200 Hz (Hi-Res Audio 2x)</option>
          <option value={96000}>96,000 Hz (Ultra Low Latency Studio)</option>
        </select>
      </div>

      <!-- Buffer Size & Latency -->
      <div class="setting-row">
        <label for="buffer-select">
          <Activity size={14} /> Buffer Size (I/O Block Size)
        </label>
        <select id="buffer-select" bind:value={audioState.bufferSize} class="form-select">
          <option value={64}>64 Samples (~1.3 ms - Ultra Low Latency)</option>
          <option value={128}>128 Samples (~2.6 ms - Recommended)</option>
          <option value={256}>256 Samples (~5.3 ms - Balanced Safe)</option>
          <option value={512}>512 Samples (~10.6 ms - Relaxed)</option>
          <option value={1024}>1024 Samples (~21.3 ms - High Latency)</option>
        </select>
      </div>

      <!-- Calculated Roundtrip Latency Display -->
      <div class="latency-card">
        <span class="latency-label">Estimated Roundtrip Audio Latency:</span>
        <span class="latency-val">{latencyMs} ms</span>
      </div>
    </div>

    <!-- Modal Footer -->
    <div class="modal-footer">
      <button class="cancel-btn" onclick={onClose}>Close</button>
      <button
        class="action-btn"
        class:running={audioState.isEngineRunning}
        disabled={isStarting}
        onclick={handleToggleEngine}
      >
        {isStarting
          ? 'Connecting...'
          : audioState.isEngineRunning
            ? 'Stop Audio Engine'
            : 'Start Real-Time Audio Engine'}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal-window {
    width: 480px;
    background: #1c1e25;
    border: 1px solid #353b49;
    border-radius: 12px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.8);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    outline: none;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #2b303c;
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--accent-orange, #ff6b35);
  }

  .header-title h2 {
    font-size: 15px;
    font-weight: 700;
    color: #f0f2f5;
  }

  .close-btn {
    background: none;
    border: none;
    color: #9aa0a6;
    cursor: pointer;
  }

  .close-btn:hover {
    color: #fff;
  }

  .error-banner {
    background: rgba(231, 76, 60, 0.15);
    border-bottom: 1px solid rgba(231, 76, 60, 0.3);
    color: #e74c3c;
    font-size: 12px;
    padding: 10px 20px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .modal-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .setting-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .setting-title,
  .setting-row label {
    font-size: 12px;
    font-weight: 600;
    color: #abb2bf;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .form-select {
    background: #131418;
    border: 1px solid #2e3340;
    color: #fff;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 13px;
    outline: none;
  }

  .form-select:focus {
    border-color: var(--accent-orange, #ff6b35);
  }

  .readonly-badge {
    background: #252832;
    border: 1px solid #333847;
    color: #00d2ff;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
  }

  .latency-card {
    background: #14161a;
    border: 1px solid #282c35;
    border-radius: 6px;
    padding: 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .latency-label {
    font-size: 12px;
    color: #9aa0a6;
  }

  .latency-val {
    font-size: 14px;
    font-family: monospace;
    font-weight: 700;
    color: #2ecc71;
  }

  .modal-footer {
    padding: 14px 20px;
    background: #18191f;
    border-top: 1px solid #2b303c;
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .cancel-btn {
    background: #252832;
    border: 1px solid #333847;
    color: #abb2bf;
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .cancel-btn:hover {
    background: #2d313d;
    color: #fff;
  }

  .action-btn {
    background: var(--accent-orange, #ff6b35);
    border: none;
    color: #fff;
    padding: 8px 18px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(255, 107, 53, 0.4);
  }

  .channel-toggle-group {
    display: flex;
    gap: 8px;
  }

  .chan-btn {
    flex: 1;
    background: #14161a;
    border: 1px solid #2e3340;
    color: #abb2bf;
    padding: 8px 10px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .chan-btn:hover {
    border-color: #4b5263;
    color: #fff;
  }

  .chan-btn.active {
    background: rgba(255, 107, 53, 0.15);
    border-color: var(--accent-orange, #ff6b35);
    color: var(--accent-orange, #ff6b35);
  }

  .helper-text {
    font-size: 11px;
    color: #7f848e;
    line-height: 1.4;
  }

  .signal-test-card {
    background: #131519;
    border: 1px solid #242730;
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .signal-test-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .signal-test-title {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #abb2bf;
  }

  .signal-status-pill {
    font-size: 10px;
    padding: 2px 8px;
    border-radius: 10px;
    background: #252832;
    color: #7f848e;
    font-weight: 600;
  }

  .signal-status-pill.active {
    background: rgba(46, 204, 113, 0.15);
    color: #2ecc71;
  }

  .meter-bars-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .meter-bar-item {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .bar-name {
    font-size: 11px;
    font-weight: 600;
    color: #9aa0a6;
    width: 28px;
  }

  .meter-h-track {
    flex: 1;
    height: 8px;
    background: #0d0e11;
    border-radius: 4px;
    overflow: hidden;
    position: relative;
    border: 1px solid #1f2229;
  }

  .meter-h-fill {
    height: 100%;
    background: linear-gradient(90deg, #2ecc71 0%, #f1c40f 80%, #e74c3c 100%);
    border-radius: 3px;
    transition: width 0.04s linear;
  }

  .bar-val {
    font-size: 11px;
    font-family: monospace;
    color: #7f848e;
    width: 44px;
    text-align: right;
  }

  .input-warn-text {
    font-size: 11px;
    color: #e67e22;
    background: rgba(230, 126, 34, 0.1);
    padding: 6px 10px;
    border-radius: 4px;
  }

  .action-btn.running {
    background: #e74c3c;
    box-shadow: 0 2px 8px rgba(231, 76, 60, 0.4);
  }
</style>
