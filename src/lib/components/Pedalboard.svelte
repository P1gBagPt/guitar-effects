<script lang="ts">
  import Pedal from './Pedal.svelte';
  import { pedalboardState } from '../stores/pedalboardState.svelte';
  import { ChevronLeft, ChevronRight, Sliders, Sparkles } from 'lucide-svelte';

  let selectedPreset = $state('');

  function handlePresetSelect(e: Event) {
    const target = e.target as HTMLSelectElement;
    const found = pedalboardState.presets.find((p) => p.name === target.value);
    if (found) {
      pedalboardState.loadPreset(found);
      selectedPreset = found.name;
    }
  }

  function moveLeft(index: number) {
    if (index > 0) {
      pedalboardState.movePedal(index, index - 1);
    }
  }

  function moveRight(index: number) {
    if (index < pedalboardState.pedals.length - 1) {
      pedalboardState.movePedal(index, index + 1);
    }
  }
</script>

<div class="pedalboard-container">
  <!-- Top Bar Controls -->
  <div class="board-toolbar">
    <div class="toolbar-left">
      <span class="toolbar-title">
        <Sliders size={16} /> SIGNAL CHAIN RACK
      </span>
      <span class="chain-status">
        {pedalboardState.pedals.filter((p) => p.enabled).length} ACTIVE STOMPBOXES
      </span>
    </div>

    <!-- Presets -->
    <div class="preset-selector">
      <Sparkles size={14} class="preset-icon" />
      <select onchange={handlePresetSelect} value={selectedPreset}>
        <option value="" disabled selected>Load Factory Preset...</option>
        {#each pedalboardState.presets as preset}
          <option value={preset.name}>{preset.name}</option>
        {/each}
      </select>
    </div>
  </div>

  <!-- Signal Flow Line -->
  <div class="signal-cable-track">
    <div class="cable-inlet">GUITAR IN</div>
    <div class="cable-line"></div>
    <div class="cable-inlet">AMP OUT</div>
  </div>

  <!-- Pedals Horizontal Rack -->
  <div class="pedals-rack">
    {#each pedalboardState.pedals as pedal, i}
      <div class="pedal-slot">
        <!-- Reorder buttons -->
        <div class="reorder-controls">
          <button
            class="reorder-btn"
            disabled={i === 0}
            onclick={() => moveLeft(i)}
            title="Move Earlier in Chain"
          >
            <ChevronLeft size={14} />
          </button>
          <span class="order-badge">#{i + 1}</span>
          <button
            class="reorder-btn"
            disabled={i === pedalboardState.pedals.length - 1}
            onclick={() => moveRight(i)}
            title="Move Later in Chain"
          >
            <ChevronRight size={14} />
          </button>
        </div>

        <Pedal {pedal} index={i} />
      </div>
    {/each}
  </div>
</div>

<style>
  .pedalboard-container {
    height: 100%;
    background-color: var(--bg-pedalboard, #16181d);
    background-image: radial-gradient(#262933 1px, transparent 1px);
    background-size: 20px 20px;
    border-top: 1px solid var(--border-color, #2e333d);
    border-bottom: 1px solid var(--border-color, #2e333d);
    padding: 14px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: inset 0 4px 12px rgba(0, 0, 0, 0.4);
    overflow: auto;
  }

  .board-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .toolbar-title {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 1px;
    color: #e0e4eb;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .chain-status {
    font-size: 11px;
    color: #00d2ff;
    font-weight: 600;
    background: rgba(0, 210, 255, 0.1);
    padding: 3px 8px;
    border-radius: 4px;
    border: 1px solid rgba(0, 210, 255, 0.25);
  }

  .preset-selector {
    display: flex;
    align-items: center;
    background: #1f222a;
    border: 1px solid #333844;
    border-radius: 6px;
    padding: 2px 8px;
    gap: 6px;
  }

  .preset-selector select {
    background: transparent;
    border: none;
    color: #f0f2f5;
    font-size: 12px;
    outline: none;
    cursor: pointer;
    padding: 4px 0;
  }

  .preset-selector select option {
    background: #1a1c22;
    color: #fff;
  }

  .signal-cable-track {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 10px;
  }

  .cable-inlet {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 1px;
    color: #6a7180;
    background: #20232c;
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid #2d323e;
  }

  .cable-line {
    flex: 1;
    height: 3px;
    background: linear-gradient(90deg, #ff6b35, #00d2ff, #2ecc71);
    border-radius: 2px;
    opacity: 0.7;
  }

  .pedals-rack {
    display: flex;
    gap: 24px;
    padding: 10px 4px 16px 4px;
    overflow-x: auto;
  }

  .pedal-slot {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .reorder-controls {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #1a1c22;
    padding: 2px 6px;
    border-radius: 6px;
    border: 1px solid #2b303c;
  }

  .reorder-btn {
    background: transparent;
    border: none;
    color: #9aa0a6;
    cursor: pointer;
    padding: 2px;
    display: flex;
    align-items: center;
    border-radius: 4px;
  }

  .reorder-btn:hover:not(:disabled) {
    color: #fff;
    background: #2b303c;
  }

  .reorder-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .order-badge {
    font-size: 10px;
    font-weight: 700;
    color: #f7a028;
  }
</style>
