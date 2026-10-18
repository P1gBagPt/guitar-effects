import { tauriApi, type PedalMeta } from '../api/tauriBridge';

export interface Preset {
  name: string;
  pedals: {
    id: string;
    enabled: boolean;
    params: Record<number, number>;
  }[];
}

class PedalboardState {
  pedals = $state<PedalMeta[]>([]);
  paramValues = $state<Record<string, Record<number, number>>>({});
  selectedPedalId = $state<string | null>(null);

  // Curated guitarist presets
  presets: Preset[] = [
    {
      name: 'Texas Blues Crunch',
      pedals: [
        { id: 'noise_gate', enabled: true, params: { 0: -55, 1: 1.5, 2: 10, 3: 50 } },
        { id: 'vintage_drive', enabled: true, params: { 0: 38, 1: 58, 2: 75, 3: 0 } },
        { id: 'analog_delay', enabled: true, params: { 0: 280, 1: 30, 2: 25, 3: 50, 4: 15 } },
        { id: 'hall_reverb', enabled: true, params: { 0: 45, 1: 40, 2: 20, 3: 40 } },
        { id: 'cab_sim', enabled: true, params: { 0: 1, 1: 65, 2: 80, 3: 6500, 4: 0 } },
      ],
    },
    {
      name: 'British 80s Stadium Lead',
      pedals: [
        { id: 'noise_gate', enabled: true, params: { 0: -45, 1: 1.0, 2: 8, 3: 40 } },
        { id: 'vintage_drive', enabled: true, params: { 0: 75, 1: 62, 2: 80, 3: 1 } },
        { id: 'analog_delay', enabled: true, params: { 0: 420, 1: 48, 2: 35, 3: 65, 4: 20 } },
        { id: 'hall_reverb', enabled: true, params: { 0: 75, 1: 30, 2: 35, 3: 60 } },
        { id: 'cab_sim', enabled: true, params: { 0: 0, 1: 50, 2: 90, 3: 5800, 4: 1 } },
      ],
    },
    {
      name: 'Pristine Ambient Clean',
      pedals: [
        { id: 'noise_gate', enabled: false, params: { 0: -60, 1: 2.0, 2: 10, 3: 60 } },
        { id: 'vintage_drive', enabled: false, params: { 0: 10, 1: 50, 2: 60, 3: 0 } },
        { id: 'analog_delay', enabled: true, params: { 0: 550, 1: 65, 2: 45, 3: 75, 4: 30 } },
        { id: 'hall_reverb', enabled: true, params: { 0: 88, 1: 20, 2: 50, 3: 80 } },
        { id: 'cab_sim', enabled: true, params: { 0: 2, 1: 40, 2: 70, 3: 8500, 4: 0 } },
      ],
    },
  ];

  async init() {
    try {
      const list = await tauriApi.getPedalboard();
      this.pedals = list;

      const newValues: Record<string, Record<number, number>> = {};
      for (const pedal of list) {
        newValues[pedal.id] = {};
        for (const p of pedal.parameters) {
          newValues[pedal.id][p.id] = p.default_value;
        }
      }
      this.paramValues = newValues;

      if (this.pedals.length > 0 && !this.selectedPedalId) {
        this.selectedPedalId = this.pedals[0].id;
      }
    } catch {
      // Fallback pedal list for preview
      this.pedals = [
        {
          id: 'noise_gate',
          name: 'Noise Gate',
          category: 'Dynamics',
          enabled: true,
          parameters: [
            { id: 0, name: 'Threshold', min: -80, max: 0, default_value: -50, unit: 'dB' },
            { id: 1, name: 'Attack', min: 0.1, max: 20, default_value: 1.5, unit: 'ms' },
            { id: 2, name: 'Hold', min: 0, max: 100, default_value: 10, unit: 'ms' },
            { id: 3, name: 'Release', min: 5, max: 500, default_value: 60, unit: 'ms' },
          ],
        },
        {
          id: 'vintage_drive',
          name: 'Vintage Overdrive',
          category: 'Distortion',
          enabled: true,
          parameters: [
            { id: 0, name: 'Drive', min: 0, max: 100, default_value: 35, unit: '%' },
            { id: 1, name: 'Tone', min: 0, max: 100, default_value: 50, unit: '%' },
            { id: 2, name: 'Level', min: 0, max: 100, default_value: 70, unit: '%' },
            { id: 3, name: 'Mode', min: 0, max: 1, default_value: 0, unit: 'type' },
          ],
        },
        {
          id: 'analog_delay',
          name: 'Analog Delay',
          category: 'Delay',
          enabled: true,
          parameters: [
            { id: 0, name: 'Time', min: 10, max: 2000, default_value: 380, unit: 'ms' },
            { id: 1, name: 'Feedback', min: 0, max: 110, default_value: 45, unit: '%' },
            { id: 2, name: 'Mix', min: 0, max: 100, default_value: 35, unit: '%' },
            { id: 3, name: 'Damp', min: 0, max: 100, default_value: 60, unit: '%' },
            { id: 4, name: 'Flutter', min: 0, max: 100, default_value: 10, unit: '%' },
          ],
        },
        {
          id: 'hall_reverb',
          name: 'Ambient Reverb',
          category: 'Reverb',
          enabled: true,
          parameters: [
            { id: 0, name: 'Room Size', min: 0, max: 100, default_value: 65, unit: '%' },
            { id: 1, name: 'Damp', min: 0, max: 100, default_value: 35, unit: '%' },
            { id: 2, name: 'Mix', min: 0, max: 100, default_value: 30, unit: '%' },
            { id: 3, name: 'Decay', min: 0, max: 100, default_value: 50, unit: '%' },
          ],
        },
        {
          id: 'cab_sim',
          name: 'Cabinet Simulator (IR)',
          category: 'Cabinet',
          enabled: true,
          parameters: [
            { id: 0, name: 'Model', min: 0, max: 3, default_value: 0, unit: 'type' },
            { id: 1, name: 'Proximity', min: 0, max: 100, default_value: 50, unit: '%' },
            { id: 2, name: 'Low Cut', min: 20, max: 300, default_value: 80, unit: 'Hz' },
            { id: 3, name: 'High Cut', min: 2000, max: 16000, default_value: 6500, unit: 'Hz' },
            { id: 4, name: 'Level', min: -18, max: 12, default_value: 0, unit: 'dB' },
          ],
        },
      ];

      const newValues: Record<string, Record<number, number>> = {};
      for (const pedal of this.pedals) {
        newValues[pedal.id] = {};
        for (const p of pedal.parameters) {
          newValues[pedal.id][p.id] = p.default_value;
        }
      }
      this.paramValues = newValues;
      this.selectedPedalId = this.pedals[0]?.id ?? null;
    }
  }

  async togglePedal(pedalId: string) {
    const p = this.pedals.find((x) => x.id === pedalId);
    if (!p) return;
    p.enabled = !p.enabled;
    try {
      await tauriApi.setPedalState(pedalId, p.enabled);
    } catch (e) {
      console.warn('Backend IPC error (running preview):', e);
    }
  }

  async setParam(pedalId: string, paramId: number, value: number) {
    if (!this.paramValues[pedalId]) {
      this.paramValues[pedalId] = {};
    }
    this.paramValues[pedalId][paramId] = value;
    try {
      await tauriApi.setPedalParameter(pedalId, paramId, value);
    } catch (e) {
      console.warn('Backend IPC error (running preview):', e);
    }
  }

  async loadPreset(preset: Preset) {
    for (const p of preset.pedals) {
      const match = this.pedals.find((x) => x.id === p.id);
      if (match) {
        match.enabled = p.enabled;
        await tauriApi.setPedalState(p.id, p.enabled);
        for (const [paramIdStr, val] of Object.entries(p.params)) {
          const paramId = Number(paramIdStr);
          this.setParam(p.id, paramId, val);
        }
      }
    }
  }

  movePedal(fromIndex: number, toIndex: number) {
    if (fromIndex < 0 || fromIndex >= this.pedals.length || toIndex < 0 || toIndex >= this.pedals.length) {
      return;
    }
    const [moved] = this.pedals.splice(fromIndex, 1);
    this.pedals.splice(toIndex, 0, moved);
    const order = this.pedals.map((p) => p.id);
    tauriApi.reorderPedalboard(order).catch(console.warn);
  }
}

export const pedalboardState = new PedalboardState();
