<script lang="ts">
  import { onMount } from 'svelte';
  import TransportBar from './lib/components/TransportBar.svelte';
  import Pedalboard from './lib/components/Pedalboard.svelte';
  import TabViewer from './lib/components/TabViewer.svelte';
  import MixerBar from './lib/components/MixerBar.svelte';
  import AudioSettingsModal from './lib/components/AudioSettingsModal.svelte';
  import { audioState } from './lib/stores/audioState.svelte';
  import { pedalboardState } from './lib/stores/pedalboardState.svelte';

  let showSettingsModal = $state(false);

  // Resizable layout state
  let pedalboardHeight = $state(280); // in pixels
  let mixerWidth = $state(280); // in pixels
  let isDraggingVertical = $state(false);
  let isDraggingHorizontal = $state(false);

  onMount(async () => {
    await audioState.init();
    await pedalboardState.init();
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.code === 'Space' && (e.target as HTMLElement).tagName !== 'INPUT') {
      e.preventDefault();
      audioState.toggleTransport();
    }
  }

  function startVerticalDrag(e: MouseEvent) {
    e.preventDefault();
    isDraggingVertical = true;

    const startY = e.clientY;
    const startH = pedalboardHeight;

    function onMouseMove(ev: MouseEvent) {
      const delta = startY - ev.clientY; // dragging up increases pedalboard height
      pedalboardHeight = Math.max(120, Math.min(600, startH + delta));
      // Notify alphaTab of resize
      window.dispatchEvent(new Event('resize'));
    }

    function onMouseUp() {
      isDraggingVertical = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
      window.dispatchEvent(new Event('resize'));
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  function startHorizontalDrag(e: MouseEvent) {
    e.preventDefault();
    isDraggingHorizontal = true;

    const startX = e.clientX;
    const startW = mixerWidth;

    function onMouseMove(ev: MouseEvent) {
      const delta = startX - ev.clientX; // dragging left increases mixer width
      mixerWidth = Math.max(200, Math.min(500, startW + delta));
      // Notify alphaTab of resize
      window.dispatchEvent(new Event('resize'));
    }

    function onMouseUp() {
      isDraggingHorizontal = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
      window.dispatchEvent(new Event('resize'));
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<main class="workstation-app">
  <!-- Top Transport Navigation -->
  <TransportBar onOpenSettings={() => (showSettingsModal = true)} />

  <!-- Workspace Body: Tab + Pedalboard + Mixer -->
  <div class="workspace-layout">
    <!-- Main Center Area -->
    <div class="center-stage">
      <!-- Upper: AlphaTab Tablature Viewer -->
      <TabViewer />

      <!-- Horizontal Resizer Splitter Bar -->
      <div
        class="resizer-horizontal"
        class:dragging={isDraggingVertical}
        onmousedown={startVerticalDrag}
        role="separator"
        aria-orientation="horizontal"
        tabindex="0"
        title="Drag up or down to resize Tab & Pedalboard"
      >
        <div class="resizer-handle-pill"></div>
      </div>

      <!-- Lower: Modular Pedalboard Rack with dynamic height -->
      <div class="pedalboard-wrapper" style="height: {pedalboardHeight}px;">
        <Pedalboard />
      </div>
    </div>

    <!-- Vertical Resizer Splitter Bar between Stage & Mixer -->
    <div
      class="resizer-vertical"
      class:dragging={isDraggingHorizontal}
      onmousedown={startHorizontalDrag}
      role="separator"
      aria-orientation="vertical"
      tabindex="0"
      title="Drag left or right to resize Mixer Bus"
    >
      <div class="resizer-handle-pill-v"></div>
    </div>

    <!-- Right: Mixer Bus & Level Decibel Meters with dynamic width -->
    <div class="mixer-wrapper" style="width: {mixerWidth}px;">
      <MixerBar />
    </div>
  </div>

  <!-- Audio Settings Modal -->
  {#if showSettingsModal}
    <AudioSettingsModal onClose={() => (showSettingsModal = false)} />
  {/if}
</main>

<style>
  .workstation-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background-color: var(--bg-main, #121316);
    color: var(--text-primary, #f0f2f5);
    overflow: hidden;
  }

  .workspace-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .center-stage {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  /* Horizontal Resizer between TabViewer and Pedalboard */
  .resizer-horizontal {
    height: 8px;
    background: #171920;
    border-top: 1px solid #282d38;
    border-bottom: 1px solid #282d38;
    cursor: row-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease;
    user-select: none;
    z-index: 10;
  }

  .resizer-horizontal:hover,
  .resizer-horizontal.dragging {
    background: #0284c7;
  }

  .resizer-handle-pill {
    width: 48px;
    height: 3px;
    background: #475569;
    border-radius: 2px;
  }

  .resizer-horizontal:hover .resizer-handle-pill,
  .resizer-horizontal.dragging .resizer-handle-pill {
    background: #ffffff;
  }

  /* Vertical Resizer between Center Stage and Mixer */
  .resizer-vertical {
    width: 8px;
    background: #171920;
    border-left: 1px solid #282d38;
    border-right: 1px solid #282d38;
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease;
    user-select: none;
    z-index: 10;
  }

  .resizer-vertical:hover,
  .resizer-vertical.dragging {
    background: #0284c7;
  }

  .resizer-handle-pill-v {
    width: 3px;
    height: 48px;
    background: #475569;
    border-radius: 2px;
  }

  .resizer-vertical:hover .resizer-handle-pill-v,
  .resizer-vertical.dragging .resizer-handle-pill-v {
    background: #ffffff;
  }

  .pedalboard-wrapper {
    min-height: 120px;
    max-height: 600px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .mixer-wrapper {
    min-width: 200px;
    max-width: 500px;
    overflow: hidden;
    display: flex;
  }
</style>
