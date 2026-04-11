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
 *   ( )        — grouping to override precedence
 *
 * Precedence: AND (+) binds tighter than OR (,), parentheses override.
 *
 * Examples:
 *   startswith=C:\foo+endswith=bar,baz
 *     → (starts with C:\foo AND ends with bar) OR (contains baz)
 *   (A,B)+C
 *     → (A OR B) AND C
 */

export type Matcher = (text: string) => boolean;

/**
 * Structured search expression tree.
 * Used by multi-field search so AND terms can match across different fields.
 */
export type SearchExpr =
  | { type: 'term'; matcher: Matcher }
  | { type: 'and'; parts: SearchExpr[] }
  | { type: 'or'; parts: SearchExpr[] };

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
 * Tokenize input into atoms, preserving operator=value as single tokens.
 * Tokens: '(' ')' ',' '+' and text atoms.
 */
function tokenize(input: string): string[] {
  const tokens: string[] = [];
  let i = 0;
  while (i < input.length) {
    const ch = input[i];
    if (ch === '(' || ch === ')' || ch === ',' || ch === '+') {
      tokens.push(ch);
      i++;
    } else if (ch === ' ') {
      i++;
    } else {
      // Consume text atom until we hit a combinator or paren
      let atom = '';
      while (i < input.length && input[i] !== ',' && input[i] !== '+' && input[i] !== '(' && input[i] !== ')') {
        atom += input[i];
        i++;
      }
      const trimmed = atom.trim();
      if (trimmed) tokens.push(trimmed);
    }
  }
  return tokens;
}

/**
 * Recursive descent parser for search expressions.
 *
 * Grammar:
 *   expr     → andExpr (',' andExpr)*
 *   andExpr  → primary ('+' primary)*
 *   primary  → '(' expr ')' | TERM
 *
 * Returns a single Matcher representing the full expression.
 */
function parseExpr(tokens: string[], pos: { i: number }): Matcher {
  // Skip leading commas
  while (pos.i < tokens.length && tokens[pos.i] === ',') pos.i++;
  if (pos.i >= tokens.length) return () => true;
  const orParts: Matcher[] = [parseAndExpr(tokens, pos)];
  while (pos.i < tokens.length && tokens[pos.i] === ',') {
    pos.i++; // consume ','
    // Skip consecutive commas
    while (pos.i < tokens.length && tokens[pos.i] === ',') pos.i++;
    if (pos.i < tokens.length && tokens[pos.i] !== ')') {
      orParts.push(parseAndExpr(tokens, pos));
    }
  }
  if (orParts.length === 1) return orParts[0];
  return (text) => orParts.some((m) => m(text));
}

function parseAndExpr(tokens: string[], pos: { i: number }): Matcher {
  const andParts: Matcher[] = [parsePrimary(tokens, pos)];
  while (pos.i < tokens.length && tokens[pos.i] === '+') {
    pos.i++; // consume '+'
    andParts.push(parsePrimary(tokens, pos));
  }
  if (andParts.length === 1) return andParts[0];
  return (text) => andParts.every((m) => m(text));
}

function parsePrimary(tokens: string[], pos: { i: number }): Matcher {
  if (pos.i >= tokens.length) return () => true;
  const tok = tokens[pos.i];
  if (tok === ',' || tok === '+' || tok === ')') return () => true; // empty operand
  if (tok === '(') {
    pos.i++; // consume '('
    const matcher = parseExpr(tokens, pos);
    if (pos.i < tokens.length && tokens[pos.i] === ')') {
      pos.i++; // consume ')'
    }
    return matcher;
  }
  return parseTerm(tokens[pos.i++]);
}

/**
 * Parse into a SearchExpr tree — used by multi-field search so AND terms
 * can match across different fields.
 */
function parseExprTree(tokens: string[], pos: { i: number }): SearchExpr {
  while (pos.i < tokens.length && tokens[pos.i] === ',') pos.i++;
  if (pos.i >= tokens.length) return { type: 'term', matcher: () => true };
  const orParts: SearchExpr[] = [parseAndExprTree(tokens, pos)];
  while (pos.i < tokens.length && tokens[pos.i] === ',') {
    pos.i++;
    while (pos.i < tokens.length && tokens[pos.i] === ',') pos.i++;
    if (pos.i < tokens.length && tokens[pos.i] !== ')') {
      orParts.push(parseAndExprTree(tokens, pos));
    }
  }
  return orParts.length === 1 ? orParts[0] : { type: 'or', parts: orParts };
}

function parseAndExprTree(tokens: string[], pos: { i: number }): SearchExpr {
  const andParts: SearchExpr[] = [parsePrimaryTree(tokens, pos)];
  while (pos.i < tokens.length && tokens[pos.i] === '+') {
    pos.i++;
    andParts.push(parsePrimaryTree(tokens, pos));
  }
  return andParts.length === 1 ? andParts[0] : { type: 'and', parts: andParts };
}

function parsePrimaryTree(tokens: string[], pos: { i: number }): SearchExpr {
  if (pos.i >= tokens.length) return { type: 'term', matcher: () => true };
  const tok = tokens[pos.i];
  if (tok === ',' || tok === '+' || tok === ')') return { type: 'term', matcher: () => true };
  if (tok === '(') {
    pos.i++;
    const expr = parseExprTree(tokens, pos);
    if (pos.i < tokens.length && tokens[pos.i] === ')') pos.i++;
    return expr;
  }
  return { type: 'term', matcher: parseTerm(tokens[pos.i++]) };
}

/**
 * Parse a search string into an array of Matchers (single-field matching).
 * Comma = OR, + = AND, parentheses for grouping.
 * Returns empty array if input is blank.
 */
export function parseSearchTerms(input: string): Matcher[] {
  if (!input.trim()) return [];
  const tokens = tokenize(input);
  if (tokens.length === 0) return [];
  const pos = { i: 0 };
  const matcher = parseExpr(tokens, pos);
  return [matcher];
}

/**
 * Parse a search string into a SearchExpr tree for multi-field matching.
 * AND terms can match across different fields.
 */
export function parseSearchExpr(input: string): SearchExpr | null {
  if (!input.trim()) return null;
  const tokens = tokenize(input);
  if (tokens.length === 0) return null;
  const pos = { i: 0 };
  return parseExprTree(tokens, pos);
}

/**
 * Evaluate a SearchExpr against multiple fields.
 * AND: each part must match at least one field (different parts can match different fields).
 * OR: any part matching any field is sufficient.
 * Term: must match at least one field.
 */
export function matchesMultiField(expr: SearchExpr, fields: string[]): boolean {
  switch (expr.type) {
    case 'term':
      return fields.some(f => expr.matcher(f));
    case 'and':
      return expr.parts.every(part => matchesMultiField(part, fields));
    case 'or':
      return expr.parts.some(part => matchesMultiField(part, fields));
  }
}

/**
 * Test if any matcher matches the given text (OR logic across groups).
 * Returns true if matchers is empty (no filter = match all).
 */
export function matchesAny(matchers: Matcher[], text: string): boolean {
  if (matchers.length === 0) return true;
  return matchers.some((m) => m(text));
}
