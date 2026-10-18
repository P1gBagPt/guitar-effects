<script lang="ts">
  import Knob from './Knob.svelte';
  import { pedalboardState } from '../stores/pedalboardState.svelte';
  import type { PedalMeta } from '../api/tauriBridge';

  let {
    pedal,
    index,
  } = $props<{
    pedal: PedalMeta;
    index: number;
  }>();

  const theme = $derived.by(() => {
    switch (pedal.id) {
      case 'vintage_drive':
        return {
          gradient: 'linear-gradient(145deg, #2e7d32, #1b5e20)',
          border: '#388e3c',
          ledColor: '#00e676',
          textColor: '#e8f5e9',
        };
      case 'analog_delay':
        return {
          gradient: 'linear-gradient(145deg, #00838f, #004d40)',
          border: '#00acc1',
          ledColor: '#00e5ff',
          textColor: '#e0f7fa',
        };
      case 'hall_reverb':
        return {
          gradient: 'linear-gradient(145deg, #512da8, #311b92)',
          border: '#673ab7',
          ledColor: '#d500f9',
          textColor: '#ede7f6',
        };
      case 'noise_gate':
        return {
          gradient: 'linear-gradient(145deg, #37474f, #212121)',
          border: '#546e7a',
          ledColor: '#ff1744',
          textColor: '#eceff1',
        };
      case 'cab_sim':
        return {
          gradient: 'linear-gradient(145deg, #d84315, #bf360c)',
          border: '#f4511e',
          ledColor: '#ffab00',
          textColor: '#fbe9e7',
        };
      default:
        return {
          gradient: 'linear-gradient(145deg, #424242, #212121)',
          border: '#616161',
          ledColor: '#ff5252',
          textColor: '#f5f5f5',
        };
    }
  });

  function handleStomp() {
    pedalboardState.togglePedal(pedal.id);
  }

  function handleParamChange(paramId: number, val: number) {
    pedalboardState.setParam(pedal.id, paramId, val);
  }
</script>

<div
  class="pedal-chassis"
  style="background: {theme.gradient}; border-color: {theme.border};"
>
  <!-- Pedal Header -->
  <div class="pedal-header">
    <span class="pedal-category">{pedal.category}</span>
    <h3 class="pedal-name">{pedal.name}</h3>
  </div>

  <!-- LED Indicator -->
  <div class="led-container">
    <div
      class="led"
      class:active={pedal.enabled}
      style="--led-color: {theme.ledColor};"
    ></div>
  </div>

  <!-- Knobs Grid -->
  <div class="knobs-grid">
    {#each pedal.parameters as param}
      <Knob
        label={param.name}
        min={param.min}
        max={param.max}
        unit={param.unit === 'dB' ? 'dB' : param.unit === 'ms' ? 'ms' : param.unit === 'Hz' ? 'Hz' : '%'}
        defaultValue={param.default_value}
        value={pedalboardState.paramValues[pedal.id]?.[param.id] ?? param.default_value}
        onChange={(val) => handleParamChange(param.id, val)}
      />
    {/each}
  </div>

  <!-- Footswitch Stomp Button -->
  <div class="stomp-section">
    <button
      class="stomp-switch"
      onclick={handleStomp}
      title="True Bypass Footswitch"
    >
      <div class="switch-nut">
        <div class="switch-plunger"></div>
      </div>
    </button>
    <span class="bypass-label">{pedal.enabled ? 'ACTIVE' : 'BYPASS'}</span>
  </div>
</div>

<style>
  .pedal-chassis {
    width: 220px;
    min-height: 380px;
    border-radius: 12px;
    border: 2px solid;
    padding: 16px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.6), inset 0 1px 2px rgba(255, 255, 255, 0.2);
    position: relative;
    user-select: none;
    transition: transform 0.15s ease;
  }

  .pedal-chassis:hover {
    transform: translateY(-2px);
  }

  .pedal-header {
    text-align: center;
    width: 100%;
  }

  .pedal-category {
    font-size: 9px;
    letter-spacing: 1.5px;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.6);
  }

  .pedal-name {
    font-size: 14px;
    font-weight: 700;
    letter-spacing: 0.5px;
    margin-top: 2px;
    color: #ffffff;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.8);
  }

  .led-container {
    display: flex;
    justify-content: center;
    margin: 8px 0;
  }

  .led {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background-color: #222;
    border: 1px solid #111;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.8);
    transition: all 0.1s ease;
  }

  .led.active {
    background-color: var(--led-color);
    box-shadow: 0 0 10px var(--led-color), 0 0 16px var(--led-color), inset 0 1px 2px rgba(255, 255, 255, 0.8);
  }

  .knobs-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 14px 10px;
    width: 100%;
    margin: 12px 0;
  }

  .stomp-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    margin-top: auto;
    padding-top: 10px;
  }

  .stomp-switch {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    outline: none;
  }

  .switch-nut {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 35%, #7a828e, #3a3f47);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.7);
    border: 1px solid #2b2e35;
  }

  .switch-plunger {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 35%, #e0e0e0, #8d939c);
    box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.6), 0 2px 4px rgba(0, 0, 0, 0.5);
    transition: transform 0.05s ease;
  }

  .stomp-switch:active .switch-plunger {
    transform: scale(0.92);
  }

  .bypass-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 1px;
    color: rgba(255, 255, 255, 0.7);
  }
</style>
