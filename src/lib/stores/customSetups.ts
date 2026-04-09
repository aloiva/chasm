import { writable, get } from 'svelte/store';
import {
  viewMode,
  sortBy,
  selectedSources,
  groupFilter,
  collapsedGroups,
  filters,
  groupedSessions,
  defaultFilters,
  type FilterState,
} from './sessions';

export interface CustomSetupConfig {
  viewMode: 'source' | 'folder' | 'branch' | 'date';
  groupFilter: string;
  collapseAll: boolean;
  selectedSources: string[];
  sortBy: 'updated' | 'created' | 'turns' | 'size' | 'title' | 'branch';
  filters: FilterState;
}

export interface CustomSetup {
  id: string;
  name: string;
  builtIn: boolean;
  config: CustomSetupConfig;
}

const STORAGE_KEY = 'chasm-custom-setups';

const builtInSetups: CustomSetup[] = [
  {
    id: 'dobby',
    name: 'Dobby',
    builtIn: true,
    config: {
      viewMode: 'folder',
      groupFilter: 'C:\\dobby\\agents',
      collapseAll: true,
      selectedSources: ['copilot-cli'],
      sortBy: 'updated',
      filters: { ...defaultFilters },
    },
  },
];

function loadSetups(): CustomSetup[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const userSetups = JSON.parse(raw) as CustomSetup[];
      return [...builtInSetups, ...userSetups.filter((s) => !s.builtIn)];
    }
  } catch {
    /* ignore corrupt data */
  }
  return [...builtInSetups];
}

function persistSetups(setups: CustomSetup[]) {
  const userSetups = setups.filter((s) => !s.builtIn);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(userSetups));
}

export const customSetups = writable<CustomSetup[]>(loadSetups());
export const activeSetupId = writable<string | null>(null);

/** Apply a custom setup — sets all stores to match the config */
export function applySetup(setup: CustomSetup) {
  const { config } = setup;

  // viewMode subscriber clears groupFilter + collapsedGroups, so set it first
  viewMode.set(config.viewMode);

  // Now set the rest (after subscriber has fired)
  groupFilter.set(config.groupFilter);
  sortBy.set(config.sortBy);
  selectedSources.set(new Set(config.selectedSources));
  filters.set({ ...config.filters });

  if (config.collapseAll) {
    const groups = get(groupedSessions);
    collapsedGroups.set(new Set(Object.keys(groups)));
  }

  activeSetupId.set(setup.id);
}

/** Capture the current app state as a new custom setup */
export function saveCurrentAsSetup(name: string): CustomSetup {
  const id = `custom-${Date.now()}`;

  const groups = get(groupedSessions);
  const allKeys = Object.keys(groups);
  const collapsed = get(collapsedGroups);
  const collapseAll = allKeys.length > 0 && allKeys.every((k) => collapsed.has(k));

  const setup: CustomSetup = {
    id,
    name,
    builtIn: false,
    config: {
      viewMode: get(viewMode),
      groupFilter: get(groupFilter),
      collapseAll,
      selectedSources: Array.from(get(selectedSources)),
      sortBy: get(sortBy),
      filters: { ...get(filters) },
    },
  };

  customSetups.update((list) => {
    const updated = [...list, setup];
    persistSetups(updated);
    return updated;
  });

  activeSetupId.set(id);
  return setup;
}

/** Delete a user-created setup */
export function deleteSetup(id: string) {
  customSetups.update((list) => {
    const updated = list.filter((s) => s.id !== id || s.builtIn);
    persistSetups(updated);
    return updated;
  });
  if (get(activeSetupId) === id) activeSetupId.set(null);
}

/** Clear the active setup indicator */
export function clearActiveSetup() {
  activeSetupId.set(null);
}
