import { writable, derived } from 'svelte/store';
import type { SessionSummary, SourceInfo } from '$lib/types/session';

export const sessions = writable<SessionSummary[]>([]);
export const sources = writable<SourceInfo[]>([]);
export const loading = writable(false);
export const searchQuery = writable('');
export const selectedSource = writable<string>('all');
export const sortBy = writable<'updated' | 'created' | 'turns' | 'size'>('updated');
export const selectedSessionId = writable<string | null>(null);

export const filteredSessions = derived(
  [sessions, searchQuery, selectedSource, sortBy],
  ([$sessions, $query, $source, $sort]) => {
    let result = $sessions;

    // Filter by source
    if ($source !== 'all') {
      result = result.filter(s => s.source === $source);
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
        default:
          return 0;
      }
    });

    return result;
  }
);

// Group sessions by source
export const groupedSessions = derived(filteredSessions, ($filtered) => {
  const groups: Record<string, SessionSummary[]> = {};
  for (const s of $filtered) {
    if (!groups[s.source]) groups[s.source] = [];
    groups[s.source].push(s);
  }
  return groups;
});
