import { writable, derived } from 'svelte/store';
import type { SessionSummary, SourceInfo } from '$lib/types/session';
import { parseSearchTerms, matchesAny } from '$lib/utils/search';

export const sessions = writable<SessionSummary[]>([]);
export const sources = writable<SourceInfo[]>([]);
export const loading = writable(false);
export const searchQuery = writable('');
export const sortBy = writable<'updated' | 'created' | 'turns' | 'size' | 'title' | 'branch' | 'folder' | 'source'>('updated');
export const viewMode = writable<'source' | 'folder' | 'branch' | 'date'>('source');
export const selectedSessionId = writable<string | null>(null);
export const selectedGroupKey = writable<string | null>(null);
export const collapsedGroups = writable<Set<string>>(new Set());
export const groupFilter = writable('');
/** Bumped on every scan to trigger detail panel re-fetch */
export const refreshCounter = writable(0);

/* ── Pinned sessions (persisted to localStorage) ── */
function loadPinned(): Set<string> {
  try {
    const raw = localStorage.getItem('chasm:pinned');
    if (raw) return new Set(JSON.parse(raw));
  } catch { /* ignore */ }
  return new Set();
}

function savePinned(s: Set<string>) {
  localStorage.setItem('chasm:pinned', JSON.stringify([...s]));
}

export const pinnedSessions = writable<Set<string>>(loadPinned());
pinnedSessions.subscribe(savePinned);

export function togglePin(compositeId: string) {
  pinnedSessions.update(s => {
    const next = new Set(s);
    if (next.has(compositeId)) next.delete(compositeId);
    else next.add(compositeId);
    return next;
  });
}

export function isPinned(compositeId: string, pinned: Set<string>): boolean {
  return pinned.has(compositeId);
}

// Reset collapsed groups and group filter when view mode changes
viewMode.subscribe(() => {
  collapsedGroups.set(new Set());
  groupFilter.set('');
});

/** Select a group — clears any selected session */
export function selectGroup(key: string) {
  selectedSessionId.set(null);
  selectedGroupKey.set(key);
}

/** Select a session — clears any selected group */
export function selectSession(compositeId: string) {
  selectedGroupKey.set(null);
  selectedSessionId.set(compositeId);
}

/** Advanced filters — all optional, applied cumulatively */
export interface FilterState {
  hasCheckpoints: boolean | null;
  hideDeleted: boolean;
  hideEmpty: boolean;
  status: string | null;
  minTurns: number | null;
  maxTurns: number | null;
  dateFrom: string;
  dateTo: string;
  titleFilter: string;
  folderFilter: string;
  branchFilter: string;
}

export const defaultFilters: FilterState = {
  hasCheckpoints: null,
  hideDeleted: true,
  hideEmpty: false,
  status: null,
  minTurns: null,
  maxTurns: null,
  dateFrom: '',
  dateTo: '',
  titleFilter: '',
  folderFilter: '',
  branchFilter: '',
};

export const filters = writable<FilterState>({ ...defaultFilters });

export function resetFilters() {
  filters.set({ ...defaultFilters });
}

/** Count active (non-default) filters */
export const activeFilterCount = derived(filters, ($f) => {
  let count = 0;
  if ($f.hasCheckpoints !== null) count++;
  if (!$f.hideDeleted) count++;
  if ($f.hideEmpty) count++;
  if ($f.status !== null) count++;
  if ($f.minTurns !== null) count++;
  if ($f.maxTurns !== null) count++;
  if ($f.dateFrom) count++;
  if ($f.dateTo) count++;
  if ($f.titleFilter) count++;
  if ($f.folderFilter) count++;
  if ($f.branchFilter) count++;
  return count;
});

export const filteredSessions = derived(
  [sessions, searchQuery, sortBy, filters, pinnedSessions],
  ([$sessions, $query, $sort, $filters, $pinned]) => {
    let result = $sessions;

    // Filter by search query — comma-separated terms with operators, OR logic
    if ($query.trim()) {
      const matchers = parseSearchTerms($query);
      if (matchers.length > 0) {
        result = result.filter(s => {
          // Test each field independently so startswith=/endswith= work per-field
          const fields = [s.title, s.first_message, s.cwd, s.branch, s.id]
            .map(v => v ?? '');
          return matchers.some(m => fields.some(f => m(f)));
        });
      }
    }

    // Advanced filters
    if ($filters.hasCheckpoints !== null) {
      result = result.filter(s => s.has_checkpoints === $filters.hasCheckpoints);
    }
    if ($filters.hideDeleted) {
      result = result.filter(s => s.exists_on_disk !== false);
    }
    if ($filters.hideEmpty) {
      result = result.filter(s => s.turn_count > 0);
    }
    if ($filters.status !== null) {
      if ($filters.status === 'active') {
        result = result.filter(s => s.exists_on_disk !== false);
      } else if ($filters.status === 'deleted') {
        result = result.filter(s => s.exists_on_disk === false);
      }
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

    // Title filter — comma-separated with operators, OR logic
    if ($filters.titleFilter.trim()) {
      const matchers = parseSearchTerms($filters.titleFilter);
      if (matchers.length > 0) {
        result = result.filter(s => {
          // Match against the display title: title, or first_message fallback
          const display = s.title ?? s.first_message ?? '';
          return matchesAny(matchers, display);
        });
      }
    }

    // Folder filter — comma-separated with operators, OR logic
    if ($filters.folderFilter.trim()) {
      const matchers = parseSearchTerms($filters.folderFilter);
      if (matchers.length > 0) {
        result = result.filter(s => {
          const cwd = s.cwd ?? '';
          return matchesAny(matchers, cwd);
        });
      }
    }

    // Branch filter — comma-separated with operators, OR logic
    if ($filters.branchFilter.trim()) {
      const matchers = parseSearchTerms($filters.branchFilter);
      if (matchers.length > 0) {
        result = result.filter(s => {
          const branch = s.branch ?? '';
          return matchesAny(matchers, branch);
        });
      }
    }

    // Sort — pinned first, then by chosen sort field
    result = [...result].sort((a, b) => {
      const aPinned = $pinned.has(a.id + ':' + a.source) ? 1 : 0;
      const bPinned = $pinned.has(b.id + ':' + b.source) ? 1 : 0;
      if (aPinned !== bPinned) return bPinned - aPinned;

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
        case 'folder':
          return (a.cwd ?? '').localeCompare(b.cwd ?? '');
        case 'source':
          return (a.source ?? '').localeCompare(b.source ?? '');
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
  [filteredSessions, viewMode, sources],
  ([$filtered, $view, $sources]) => {
    const groups: Record<string, SessionSummary[]> = {};

    // Build a lookup from internal source name to display name
    const sourceDisplayMap = new Map<string, string>();
    for (const src of $sources) {
      sourceDisplayMap.set(src.name, src.display_name);
    }

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
          key = sourceDisplayMap.get(s.source) ?? s.source;
      }
      if (!groups[key]) groups[key] = [];
      groups[key].push(s);
    }

    return groups;
  }
);

/** Apply groupFilter to groupedSessions — filters group keys.
 *  Supports comma-separated patterns with operators (startswith=, endswith=, not=, !); , for OR, + for AND. */
export const filteredGroupedSessions = derived(
  [groupedSessions, groupFilter],
  ([$groups, $filter]) => {
    if (!$filter.trim()) return $groups;
    const matchers = parseSearchTerms($filter);
    if (matchers.length === 0) return $groups;

    const filtered: Record<string, SessionSummary[]> = {};
    for (const [key, sessions] of Object.entries($groups)) {
      if (matchesAny(matchers, key)) {
        filtered[key] = sessions;
      }
    }
    return filtered;
  }
);
