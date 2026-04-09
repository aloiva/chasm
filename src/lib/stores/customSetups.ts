import { writable, get } from 'svelte/store';
import {
  viewMode,
  sortBy,
  groupFilter,
  collapsedGroups,
  filters,
  groupedSessions,
  searchQuery,
  defaultFilters,
  type FilterState,
} from './sessions';

export interface CustomSetupConfig {
  viewMode: 'source' | 'folder' | 'branch' | 'date';
  groupFilter: string;
  collapseAll: boolean;
  sortBy: 'updated' | 'created' | 'turns' | 'size' | 'title' | 'branch' | 'folder' | 'source';
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
    id: 'copilot-cli-sessions',
    name: 'Copilot CLI Sessions',
    builtIn: true,
    config: {
      viewMode: 'source',
      groupFilter: 'Copilot CLI',
      collapseAll: false,
      sortBy: 'updated',
      filters: { ...defaultFilters },
    },
  },
  {
    id: 'vscode-chat-sessions',
    name: 'VS Code Chat Sessions',
    builtIn: true,
    config: {
      viewMode: 'source',
      groupFilter: 'VS Code Copilot',
      collapseAll: false,
      sortBy: 'updated',
      filters: { ...defaultFilters },
    },
  },
  {
    id: 'dobby',
    name: 'Dobby',
    builtIn: true,
    config: {
      viewMode: 'folder',
      groupFilter: 'C:\\dobby\\agents',
      collapseAll: true,
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

/** Delete all user-created setups */
export function deleteAllUserSetups() {
  customSetups.update((list) => {
    const updated = list.filter((s) => s.builtIn);
    persistSetups(updated);
    return updated;
  });
  activeSetupId.set(null);
}

/** Clear the active setup and reset all stores to defaults */
export function clearActiveSetup() {
  activeSetupId.set(null);
  viewMode.set('source');
  sortBy.set('updated');
  searchQuery.set('');
  groupFilter.set('');
  collapsedGroups.set(new Set());
  filters.set({ ...defaultFilters });
}
