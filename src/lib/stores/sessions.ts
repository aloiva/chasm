import { writable, derived } from 'svelte/store';
import type { SessionSummary, SourceInfo } from '$lib/types/session';

export const sessions = writable<SessionSummary[]>([]);
export const sources = writable<SourceInfo[]>([]);
export const loading = writable(false);
export const searchQuery = writable('');
export const selectedSources = writable<Set<string>>(new Set());
export const sortBy = writable<'updated' | 'created' | 'turns' | 'size' | 'title' | 'branch'>('updated');
export const viewMode = writable<'source' | 'folder' | 'branch' | 'date'>('source');
export const selectedSessionId = writable<string | null>(null);

/** Advanced filters — all optional, applied cumulatively */
export interface FilterState {
  folderStartsWith: string;
  folderContains: string;
  branch: string;
  hasCheckpoints: boolean | null;
  existsOnDisk: boolean | null;
  hideDeleted: boolean;
  status: string | null;
  minTurns: number | null;
  maxTurns: number | null;
  dateFrom: string;
  dateTo: string;
}

const defaultFilters: FilterState = {
  folderStartsWith: '',
  folderContains: '',
  branch: '',
  hasCheckpoints: null,
  existsOnDisk: null,
  hideDeleted: true,
  status: null,
  minTurns: null,
  maxTurns: null,
  dateFrom: '',
  dateTo: '',
};

export const filters = writable<FilterState>({ ...defaultFilters });

export function resetFilters() {
  filters.set({ ...defaultFilters });
}

/** Count active (non-default) filters */
export const activeFilterCount = derived(filters, ($f) => {
  let count = 0;
  if ($f.folderStartsWith) count++;
  if ($f.folderContains) count++;
  if ($f.branch) count++;
  if ($f.hasCheckpoints !== null) count++;
  if ($f.existsOnDisk !== null) count++;
  if (!$f.hideDeleted) count++;
  if ($f.status !== null) count++;
  if ($f.minTurns !== null) count++;
  if ($f.maxTurns !== null) count++;
  if ($f.dateFrom) count++;
  if ($f.dateTo) count++;
  return count;
});

export const filteredSessions = derived(
  [sessions, searchQuery, selectedSources, sortBy, filters],
  ([$sessions, $query, $sources, $sort, $filters]) => {
    let result = $sessions;

    // Filter by selected sources (empty set = show all)
    if ($sources.size > 0) {
      result = result.filter(s => $sources.has(s.source));
    }

    // Filter by search query
    if ($query.trim()) {
      const q = $query.toLowerCase();
      result = result.filter(s =>
        (s.title?.toLowerCase().includes(q)) ||
        (s.first_message?.toLowerCase().includes(q)) ||
        (s.cwd?.toLowerCase().includes(q)) ||
        (s.branch?.toLowerCase().includes(q)) ||
        s.id.toLowerCase().includes(q)
      );
    }

    // Advanced filters
    if ($filters.folderStartsWith) {
      const prefix = $filters.folderStartsWith.toLowerCase();
      result = result.filter(s => s.cwd?.toLowerCase().startsWith(prefix));
    }
    if ($filters.folderContains) {
      const substr = $filters.folderContains.toLowerCase();
      result = result.filter(s => s.cwd?.toLowerCase().includes(substr));
    }
    if ($filters.branch) {
      const br = $filters.branch.toLowerCase();
      result = result.filter(s => s.branch?.toLowerCase().includes(br));
    }
    if ($filters.hasCheckpoints !== null) {
      result = result.filter(s => s.has_checkpoints === $filters.hasCheckpoints);
    }
    if ($filters.existsOnDisk !== null) {
      result = result.filter(s => s.exists_on_disk === $filters.existsOnDisk);
    }
    if ($filters.hideDeleted) {
      result = result.filter(s => s.exists_on_disk !== false);
    }
    if ($filters.status !== null) {
      result = result.filter(s => s.status === $filters.status);
    }
    if ($filters.minTurns !== null) {
      result = result.filter(s => s.turn_count >= ($filters.minTurns ?? 0));
    }
    if ($filters.maxTurns !== null) {
      result = result.filter(s => s.turn_count <= ($filters.maxTurns ?? Infinity));
    }
    if ($filters.dateFrom) {
      result = result.filter(s => (s.created_at ?? '') >= $filters.dateFrom);
    }
    if ($filters.dateTo) {
      result = result.filter(s => (s.created_at ?? '') <= $filters.dateTo);
    }

    // Sort
    result = [...result].sort((a, b) => {
      switch ($sort) {
        case 'updated':
          return (b.updated_at ?? '').localeCompare(a.updated_at ?? '');
        case 'created':
          return (b.created_at ?? '').localeCompare(a.created_at ?? '');
        case 'turns':
          return b.turn_count - a.turn_count;
        case 'size':
          return (b.size_bytes ?? 0) - (a.size_bytes ?? 0);
        case 'title':
          return (a.title ?? '').localeCompare(b.title ?? '');
        case 'branch':
          return (a.branch ?? '').localeCompare(b.branch ?? '');
        default:
          return 0;
      }
    });

    return result;
  }
);

/** Compute a week bucket key for date grouping */
function weekBucket(dateStr: string | null): string {
  if (!dateStr) return '(no date)';
  try {
    const d = new Date(dateStr);
    const day = d.getDay();
    const diff = d.getDate() - day + (day === 0 ? -6 : 1);
    const monday = new Date(d.setDate(diff));
    return `Week of ${monday.toISOString().slice(0, 10)}`;
  } catch {
    return '(no date)';
  }
}

// Group sessions by viewMode
export const groupedSessions = derived(
  [filteredSessions, viewMode],
  ([$filtered, $view]) => {
    const groups: Record<string, SessionSummary[]> = {};

    for (const s of $filtered) {
      let key: string;
      switch ($view) {
        case 'folder':
          key = s.cwd ?? '(no folder)';
          break;
        case 'branch':
          key = s.branch ?? '(no branch)';
          break;
        case 'date':
          key = weekBucket(s.updated_at);
          break;
        default:
          key = s.source;
      }
      if (!groups[key]) groups[key] = [];
      groups[key].push(s);
    }

    return groups;
  }
);
