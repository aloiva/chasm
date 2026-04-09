import { writable } from 'svelte/store';

export interface AppSettings {
  enableDobby: boolean;
}

const STORAGE_KEY = 'chasm-settings';

const defaultSettings: AppSettings = {
  enableDobby: false,
};

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      return { ...defaultSettings, ...JSON.parse(raw) };
    }
  } catch {
    // ignore
  }
  return { ...defaultSettings };
}

function persist(s: AppSettings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
}

export const settings = writable<AppSettings>(loadSettings());

settings.subscribe(persist);

export function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
  settings.update(s => ({ ...s, [key]: value }));
}
