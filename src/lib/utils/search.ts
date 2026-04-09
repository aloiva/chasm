/**
 * Advanced search operators for session and group search.
 *
 * Supported operators (case-insensitive):
 *   startswith=value   — matches if text starts with value
 *   endswith=value     — matches if text ends with value
 *   contains=value     — matches if text contains value (explicit)
 *   not=value  or !value — matches if text does NOT contain value
 *   /regex/flags       — matches via regex
 *   plain text          — default: contains (substring match)
 *
 * Multiple terms are comma-separated; OR logic across terms.
 */

export type Matcher = (text: string) => boolean;

/** Parse a single search term into a Matcher function. */
function parseTerm(raw: string): Matcher {
  const term = raw.trim();
  if (!term) return () => true;

  // Regex: /pattern/flags
  const rxMatch = term.match(/^\/(.+)\/([gimsuy]*)$/);
  if (rxMatch) {
    try {
      const rx = new RegExp(rxMatch[1], rxMatch[2] || 'i');
      return (text) => rx.test(text);
    } catch {
      // Invalid regex — fall through to contains
    }
  }

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
 * Parse a comma-separated search string into an array of Matchers.
 * Returns empty array if input is blank.
 */
export function parseSearchTerms(input: string): Matcher[] {
  if (!input.trim()) return [];
  return input
    .split(',')
    .map((t) => t.trim())
    .filter(Boolean)
    .map(parseTerm);
}

/**
 * Test if any matcher matches the given text (OR logic).
 * Returns true if matchers is empty (no filter = match all).
 */
export function matchesAny(matchers: Matcher[], text: string): boolean {
  if (matchers.length === 0) return true;
  return matchers.some((m) => m(text));
}
