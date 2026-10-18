<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { audioState } from '../stores/audioState.svelte';
  import { tauriApi } from '../api/tauriBridge';
  import { Download, Globe, FileCode, Play, Pause, ZoomIn, ZoomOut, Upload } from 'lucide-svelte';

  let containerEl: HTMLDivElement | null = $state(null);
  let api: any = $state(null);
  let urlInput = $state('');
  let isFetching = $state(false);
  let tabTitle = $state('Hendrix Blues in E Minor');
  let playbackSpeed = $state(1.0);
  let zoomLevel = $state(1.0);

  let tracks: Array<{ index: number; name: string }> = $state([]);
  let selectedTrackIndex = $state(0);

  // Built-in starter AlphaTex riff
  const defaultAlphaTex = `
\\title "Hendrix Blues Riff in E"
\\tempo 110
.
:4 0.6 2.5 2.4 0.3 | :8 2.3 3.3 0.2 2.3 :4 0.1 |
:8 0.6 3.6 5.6 7.6 :4 5.5 7.5 | :8 5.4 7.4 5.3 7.3 :2 5.3.4 |
:4 0.6 2.5 2.4 0.3 | :8 3.3 2.3 0.3 2.4 :2 0.6 |
`;

  onMount(async () => {
    try {
      const alphaTab = await import('@coderline/alphatab');
      if (!containerEl) return;

      const settings = new alphaTab.Settings();
      settings.core.engine = 'svg';
      settings.core.fontDirectory = '/font/';
      settings.core.enableLazyLoading = false; // Render all staves immediately
      settings.display.staveProfile = alphaTab.StaveProfile.ScoreTab;
      settings.display.layoutMode = alphaTab.LayoutMode.Page;
      settings.player.enablePlayer = false; // We use our native real-time low-latency Rust audio engine!

      api = new alphaTab.AlphaTabApi(containerEl, settings);

      api.scoreLoaded.on((score: any) => {
        console.log('Score loaded successfully:', score.title, 'Tracks:', score.tracks?.length);
        if (score.title) {
          tabTitle = score.title;
        } else if (score.tracks && score.tracks.length > 0) {
          tabTitle = score.tracks[0].name || tabTitle;
        }

        if (score.tracks && score.tracks.length > 0) {
          tracks = score.tracks.map((t: any, idx: number) => ({
            index: idx,
            name: t.name || `Track ${idx + 1}`,
          }));
          selectedTrackIndex = 0;
        } else {
          tracks = [];
        }
      });

      api.renderStarted.on(() => {
        console.log('AlphaTab render started');
      });

      api.renderFinished.on((e: any) => {
        console.log('AlphaTab render finished:', e);
      });

      api.postRenderFinished.on(() => {
        console.log('AlphaTab post render finished completely');
      });

      api.error.on((err: any) => {
        console.error('AlphaTab rendering error:', err);
        alert(`AlphaTab Error: ${err.message || err}`);
      });

      api.tex(defaultAlphaTex);
    } catch (err) {
      console.warn('AlphaTab initialization error (running in preview):', err);
    }
  });

  onDestroy(() => {
    if (api) {
      api.destroy();
    }
  });

  function handleTrackSelect(e: Event) {
    const target = e.target as HTMLSelectElement;
    const idx = parseInt(target.value, 10);
    selectedTrackIndex = idx;
    if (api && api.score && api.score.tracks && api.score.tracks[idx]) {
      api.renderTracks([api.score.tracks[idx]]);
    }
  }

  async function handleFetchUrl() {
    if (!urlInput.trim()) return;
    isFetching = true;
    try {
      const [filename, base64Bytes] = await tauriApi.importTabUrl(urlInput.trim());
      tabTitle = filename;
      if (api) {
        const binaryString = window.atob(base64Bytes);
        const len = binaryString.length;
        const bytes = new Uint8Array(len);
        for (let i = 0; i < len; i++) {
          bytes[i] = binaryString.charCodeAt(i);
        }
        api.load(bytes);
      }
      urlInput = '';
    } catch (err: any) {
      alert(`Failed to load tab from URL: ${err}`);
    } finally {
      isFetching = false;
    }
  }

  async function handleFileUpload(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    try {
      tabTitle = file.name;
      const buffer = await file.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      if (api) {
        api.load(bytes, [0]);
      }
    } catch (err: any) {
      console.error('File load failed:', err);
      alert(`Failed to load file "${file.name}": ${err.message || err}`);
    } finally {
      target.value = '';
    }
  }

  function handleSpeedChange(speed: number) {
    playbackSpeed = speed;
    if (api) {
      api.playbackSpeed = speed;
    }
  }

  function handleZoom(delta: number) {
    zoomLevel = Math.max(0.6, Math.min(2.0, zoomLevel + delta));
    if (api) {
      api.settings.display.scale = zoomLevel;
      api.updateSettings();
      api.render();
    }
  }

  function handleLoadDefaultTex() {
    tracks = [];
    if (api) {
      tabTitle = 'Hendrix Blues Riff in E';
      api.tex(defaultAlphaTex);
    }
  }
</script>

<div class="tab-viewer-container">
  <!-- Top Tab Toolbar -->
  <div class="tab-toolbar">
    <!-- Left: Title & File info -->
    <div class="tab-info">
      <FileCode size={16} class="tab-icon" />
      <span class="tab-title">{tabTitle}</span>

      {#if tracks.length > 1}
        <div class="track-picker-wrapper">
          <select class="track-select" value={selectedTrackIndex} onchange={handleTrackSelect}>
            {#each tracks as track}
              <option value={track.index}>{track.name}</option>
            {/each}
          </select>
        </div>
      {/if}
    </div>

    <!-- Center: URL Importer -->
    <div class="url-importer">
      <Globe size={14} class="globe-icon" />
      <input
        type="text"
        placeholder="Paste Guitar Pro (.gp/.gpx) or Web Tab URL..."
        bind:value={urlInput}
        onkeydown={(e) => e.key === 'Enter' && handleFetchUrl()}
        class="url-input"
      />
      <button
        class="fetch-btn"
        disabled={isFetching || !urlInput.trim()}
        onclick={handleFetchUrl}
      >
        {isFetching ? 'Fetching...' : 'Import Tab'}
      </button>
    </div>

    <!-- Right: Zoom & Speed -->
    <div class="tab-controls">
      <div class="speed-selector">
        <button
          class="speed-btn"
          class:active={playbackSpeed === 0.75}
          onclick={() => handleSpeedChange(0.75)}
        >
          0.75x
        </button>
        <button
          class="speed-btn"
          class:active={playbackSpeed === 1.0}
          onclick={() => handleSpeedChange(1.0)}
        >
          1.0x
        </button>
        <button
          class="speed-btn"
          class:active={playbackSpeed === 1.25}
          onclick={() => handleSpeedChange(1.25)}
        >
          1.25x
        </button>
      </div>

      <div class="zoom-buttons">
        <button class="icon-btn" onclick={() => handleZoom(-0.1)} title="Zoom Out">
          <ZoomOut size={14} />
        </button>
        <button class="icon-btn" onclick={() => handleZoom(0.1)} title="Zoom In">
          <ZoomIn size={14} />
        </button>
      </div>

      <label class="file-upload-btn" title="Open .gp, .gpx, .gp5, .musicxml, or .cap file">
        <Upload size={14} />
        <span>Open File</span>
        <input
          type="file"
          accept=".gp,.gpx,.gp5,.gp4,.gp3,.musicxml,.xml,.cap,.midi"
          onchange={handleFileUpload}
          style="display: none;"
        />
      </label>

      <button class="tex-btn" onclick={handleLoadDefaultTex} title="Reload Demo Riff">
        Demo Riff
      </button>
    </div>
  </div>

  <!-- AlphaTab Render Viewport -->
  <div class="alphatab-viewport">
    <div bind:this={containerEl} class="alphatab-canvas"></div>
  </div>
</div>

<style>
  .tab-viewer-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #17181d;
    overflow: hidden;
  }

  .tab-toolbar {
    height: 44px;
    background: #1c1e24;
    border-bottom: 1px solid var(--border-color, #2e333d);
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px;
    gap: 16px;
    user-select: none;
  }

  .tab-info {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 200px;
  }

  .tab-title {
    font-size: 13px;
    font-weight: 700;
    color: #e6ebf2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .url-importer {
    flex: 1;
    max-width: 500px;
    display: flex;
    align-items: center;
    background: #131418;
    border: 1px solid #2d323f;
    border-radius: 6px;
    padding: 2px 6px 2px 10px;
    gap: 8px;
  }

  .url-input {
    flex: 1;
    background: transparent;
    border: none;
    color: #fff;
    font-size: 12px;
    outline: none;
  }

  .fetch-btn {
    background: var(--accent-orange, #ff6b35);
    border: none;
    color: #fff;
    font-size: 11px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .fetch-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .tab-controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .speed-selector {
    display: flex;
    background: #121317;
    border: 1px solid #2b303d;
    border-radius: 4px;
    overflow: hidden;
  }

  .speed-btn {
    background: transparent;
    border: none;
    color: #8c93a0;
    font-size: 11px;
    padding: 3px 8px;
    cursor: pointer;
  }

  .speed-btn.active {
    background: #2b303d;
    color: #00d2ff;
    font-weight: 700;
  }

  .zoom-buttons {
    display: flex;
    gap: 4px;
  }

  .icon-btn {
    background: #20232c;
    border: 1px solid #2e3442;
    color: #9aa0a6;
    width: 26px;
    height: 26px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .icon-btn:hover {
    color: #fff;
    background: #2d3342;
  }

  .tex-btn {
    background: #232732;
    border: 1px solid #323847;
    color: #9aa0a6;
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
  }

  .tex-btn:hover {
    color: #fff;
    background: #2d3342;
  }

  .file-upload-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #1e293b;
    border: 1px solid #334155;
    color: #38bdf8;
    font-size: 11px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .file-upload-btn:hover {
    background: #0284c7;
    color: #ffffff;
    border-color: #38bdf8;
  }

  .track-picker-wrapper {
    margin-left: 8px;
  }

  .track-select {
    background: #121317;
    border: 1px solid #38bdf8;
    color: #38bdf8;
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 4px;
    cursor: pointer;
    outline: none;
  }

  .track-select:focus {
    box-shadow: 0 0 6px rgba(56, 189, 248, 0.4);
  }

  .alphatab-viewport {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 24px;
    background: #131418;
    display: flex;
    justify-content: center;
  }

  .alphatab-canvas {
    width: 100%;
    max-width: 950px;
    margin: 0 auto;
  }

  /* Style alphaTab's native rendering surface so every page/system has a continuous white sheet */
  :global(.alphatab-canvas .at-surface) {
    background-color: #ffffff !important;
    border-radius: 8px;
    padding: 24px 32px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.7);
    color: #111111;
    margin: 0 auto 30px auto;
  }
</style>
