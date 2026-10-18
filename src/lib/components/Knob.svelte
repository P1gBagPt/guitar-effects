<script lang="ts">
  let {
    label = 'Gain',
    value = 50,
    min = 0,
    max = 100,
    unit = '%',
    defaultValue = 50,
    size = 54,
    onChange = (val: number) => {},
  } = $props<{
    label?: string;
    value: number;
    min?: number;
    max?: number;
    unit?: string;
    defaultValue?: number;
    size?: number;
    onChange?: (val: number) => void;
  }>();

  let isDragging = $state(false);
  let startY = $state(0);
  let startVal = $state(0);

  // Angle ranges from -135deg (min) to +135deg (max) -> 270deg total sweep
  const angle = $derived.by(() => {
    const norm = (value - min) / (max - min || 1);
    return -135 + norm * 270;
  });

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDragging = true;
    startY = e.clientY;
    startVal = value;

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    const deltaY = startY - e.clientY;
    const range = max - min;
    const step = (deltaY / 150) * range;
    const newVal = Math.min(max, Math.max(min, startVal + step));
    const rounded = Math.round(newVal * 10) / 10;
    onChange(rounded);
  }

  function handleMouseUp() {
    isDragging = false;
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }

  function handleDoubleClick() {
    onChange(defaultValue);
  }
</script>

<div class="knob-container">
  <div
    class="knob-body"
    style="width: {size}px; height: {size}px;"
    onmousedown={handleMouseDown}
    ondblclick={handleDoubleClick}
    role="slider"
    tabindex="0"
    aria-valuenow={value}
    aria-valuemin={min}
    aria-valuemax={max}
  >
    <!-- SVG Circular Arc Track -->
    <svg class="knob-svg" viewBox="0 0 100 100">
      <circle
        cx="50"
        cy="50"
        r="38"
        class="track-bg"
      />
      <circle
        cx="50"
        cy="50"
        r="38"
        class="track-active"
        style="stroke-dasharray: {((value - min) / (max - min || 1)) * 180} 300;"
      />
    </svg>

    <!-- Dial Pointer Notch -->
    <div
      class="dial"
      style="transform: rotate({angle}deg);"
    >
      <div class="notch"></div>
    </div>
  </div>

  <span class="knob-label">{label}</span>
  <span class="knob-value">{value}{unit}</span>
</div>

<style>
  .knob-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    user-select: none;
  }

  .knob-body {
    position: relative;
    cursor: ns-resize;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .knob-svg {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    transform: rotate(135deg);
  }

  .track-bg {
    fill: none;
    stroke: #2b303c;
    stroke-width: 6;
    stroke-dasharray: 180 300;
  }

  .track-active {
    fill: none;
    stroke: var(--accent-orange, #ff6b35);
    stroke-width: 6;
    stroke-linecap: round;
    transition: stroke-dasharray 0.04s ease-out;
  }

  .dial {
    position: relative;
    width: 72%;
    height: 72%;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 35%, #3a3f4d, #1c1e24);
    box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.2), 0 3px 6px rgba(0, 0, 0, 0.6);
    border: 1px solid #18191f;
    display: flex;
    justify-content: center;
    transition: transform 0.04s ease-out;
  }

  .notch {
    position: absolute;
    top: 4px;
    width: 3px;
    height: 9px;
    background: #f0f2f5;
    border-radius: 2px;
    box-shadow: 0 0 4px rgba(255, 255, 255, 0.6);
  }

  .knob-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #9aa0a6;
  }

  .knob-value {
    font-size: 10px;
    font-family: monospace;
    color: #00d2ff;
  }
</style>
