/**
 * Advanced search operators for session and group search.
 *
 * Supported operators (case-insensitive):
 *   startswith=value   — matches if text starts with value
 *   endswith=value     — matches if text ends with value
 *   contains=value     — matches if text contains value (explicit)
 *   not=value  or !value — matches if text does NOT contain value
 *   plain text          — default: contains (substring match)
 *
 * Combinators:
 *   ,  (comma) — OR logic across groups
 *   +  (plus)  — AND logic within a group
 *
 * Example: startswith=C:\foo+endswith=bar,baz
 *   → (starts with C:\foo AND ends with bar) OR (contains baz)
 */

export type Matcher = (text: string) => boolean;

/** Parse a single search term into a Matcher function. */
function parseTerm(raw: string): Matcher {
  const term = raw.trim();
  if (!term) return () => true;

  // startswith=
  const sw = term.match(/^startswith=(.+)$/i);
  if (sw) {
    const val = sw[1].toLowerCase();
    return (text) => text.toLowerCase().startsWith(val);
  }

  // endswith=
  const ew = term.match(/^endswith=(.+)$/i);
  if (ew) {
    const val = ew[1].toLowerCase();
    return (text) => text.toLowerCase().endsWith(val);
  }

  // contains= (explicit)
  const cw = term.match(/^contains=(.+)$/i);
  if (cw) {
    const val = cw[1].toLowerCase();
    return (text) => text.toLowerCase().includes(val);
  }

  // not= or ! prefix
  const nw = term.match(/^(?:not=|!)(.+)$/i);
  if (nw) {
    const val = nw[1].toLowerCase();
    return (text) => !text.toLowerCase().includes(val);
  }

  // Default: contains (substring match)
  const q = term.toLowerCase();
  return (text) => text.toLowerCase().includes(q);
}

/**
 * Parse a single OR-group (may contain + for AND) into one compound Matcher.
 * If the group has no +, returns a single-term matcher.
 * If the group has +, all sub-terms must match (AND logic).
 */
function parseGroup(group: string): Matcher {
  const parts = group.split('+').map((t) => t.trim()).filter(Boolean);
  if (parts.length === 0) return () => true;
  if (parts.length === 1) return parseTerm(parts[0]);
  const matchers = parts.map(parseTerm);
  return (text) => matchers.every((m) => m(text));
}

/**
 * Parse a search string into an array of Matchers.
 * Comma = OR across groups, + = AND within a group.
 * Returns empty array if input is blank.
 */
export function parseSearchTerms(input: string): Matcher[] {
  if (!input.trim()) return [];
  return input
    .split(',')
    .map((g) => g.trim())
    .filter(Boolean)
    .map(parseGroup);
}

/**
 * Test if any matcher matches the given text (OR logic across groups).
 * Returns true if matchers is empty (no filter = match all).
 */
export function matchesAny(matchers: Matcher[], text: string): boolean {
  if (matchers.length === 0) return true;
  return matchers.some((m) => m(text));
}
