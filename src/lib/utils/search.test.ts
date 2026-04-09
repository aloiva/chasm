import { describe, it, expect } from 'vitest';
import { parseSearchTerms, matchesAny } from './search';

/** Helper: returns true if search matches the text. */
function matches(search: string, text: string): boolean {
  return matchesAny(parseSearchTerms(search), text);
}

// --------------- parseTerm / basic operators ---------------

describe('contains (default)', () => {
  it('matches substring', () => {
    expect(matches('chat', 'test chat - 1')).toBe(true);
  });
  it('is case-insensitive', () => {
    expect(matches('CHAT', 'test chat - 1')).toBe(true);
  });
  it('rejects non-matching', () => {
    expect(matches('foobar', 'test chat - 1')).toBe(false);
  });
  it('matches when value equals text', () => {
    expect(matches('hello', 'hello')).toBe(true);
  });
});

describe('contains= (explicit)', () => {
  it('matches substring', () => {
    expect(matches('contains=chat', 'test chat - 1')).toBe(true);
  });
  it('is case-insensitive', () => {
    expect(matches('CONTAINS=Chat', 'Test Chat - 1')).toBe(true);
  });
  it('rejects non-matching', () => {
    expect(matches('contains=xyz', 'test chat')).toBe(false);
  });
});

describe('startswith=', () => {
  it('matches prefix', () => {
    expect(matches('startswith=test', 'test chat - 1')).toBe(true);
  });
  it('matches prefix with spaces', () => {
    expect(matches('startswith=test chat', 'test chat - 1')).toBe(true);
  });
  it('is case-insensitive', () => {
    expect(matches('STARTSWITH=TEST', 'test chat - 1')).toBe(true);
  });
  it('rejects when not at start', () => {
    expect(matches('startswith=chat', 'test chat - 1')).toBe(false);
  });
  it('matches full string', () => {
    expect(matches('startswith=test chat - 1', 'test chat - 1')).toBe(true);
  });
});

describe('endswith=', () => {
  it('matches suffix', () => {
    expect(matches('endswith=1', 'test chat - 1')).toBe(true);
  });
  it('matches suffix with spaces', () => {
    expect(matches('endswith=- 1', 'test chat - 1')).toBe(true);
  });
  it('is case-insensitive', () => {
    expect(matches('ENDSWITH=CHAT', 'new chat')).toBe(true);
  });
  it('rejects when not at end', () => {
    expect(matches('endswith=test', 'test chat - 1')).toBe(false);
  });
  it('matches full string', () => {
    expect(matches('endswith=new chat', 'new chat')).toBe(true);
  });
  it('matches single char suffix', () => {
    expect(matches('endswith=t', 'new chat')).toBe(true);
  });
});

describe('not= / ! (negation)', () => {
  it('not= excludes matching text', () => {
    expect(matches('not=chat', 'test chat - 1')).toBe(false);
  });
  it('not= includes non-matching text', () => {
    expect(matches('not=xyz', 'test chat - 1')).toBe(true);
  });
  it('! prefix excludes matching text', () => {
    expect(matches('!chat', 'test chat - 1')).toBe(false);
  });
  it('! prefix includes non-matching text', () => {
    expect(matches('!xyz', 'test chat - 1')).toBe(true);
  });
  it('is case-insensitive', () => {
    expect(matches('!CHAT', 'test chat')).toBe(false);
  });
});

// --------------- OR logic (comma) ---------------

describe('OR logic (comma separator)', () => {
  it('matches first alternative', () => {
    expect(matches('foo,bar', 'foo')).toBe(true);
  });
  it('matches second alternative', () => {
    expect(matches('foo,bar', 'bar')).toBe(true);
  });
  it('rejects when neither matches', () => {
    expect(matches('foo,bar', 'baz')).toBe(false);
  });
  it('works with operators on each side', () => {
    expect(matches('startswith=a,endswith=z', 'alpha')).toBe(true);
    expect(matches('startswith=a,endswith=z', 'fizz')).toBe(true); // fizz ends with z
    expect(matches('startswith=a,endswith=z', 'jazz')).toBe(true);
  });
  it('three alternatives', () => {
    expect(matches('x,y,z', 'xyz')).toBe(true); // all contain
    expect(matches('x,y,z', 'hello z world')).toBe(true);
    expect(matches('x,y,z', 'hello')).toBe(false);
  });
});

// --------------- AND logic (plus) ---------------

describe('AND logic (plus separator)', () => {
  it('requires all terms to match', () => {
    expect(matches('test+chat', 'test chat - 1')).toBe(true);
  });
  it('fails if one term does not match', () => {
    expect(matches('test+xyz', 'test chat - 1')).toBe(false);
  });
  it('works with startswith + endswith', () => {
    expect(matches('startswith=test+endswith=1', 'test chat - 1')).toBe(true);
  });
  it('fails startswith + endswith when suffix wrong', () => {
    expect(matches('startswith=test+endswith=2', 'test chat - 1')).toBe(false);
  });
  it('fails startswith + endswith when prefix wrong', () => {
    expect(matches('startswith=foo+endswith=1', 'test chat - 1')).toBe(false);
  });
  it('works with startswith + endswith (spaces in values)', () => {
    expect(matches('startswith=test chat+endswith=- 1', 'test chat - 1')).toBe(true);
  });
  it('AND with negation', () => {
    expect(matches('startswith=feat+!test', 'feat/auth')).toBe(true);
    expect(matches('startswith=feat+!test', 'feat/test-utils')).toBe(false);
  });
  it('three terms ANDed', () => {
    expect(matches('startswith=a+contains=b+endswith=c', 'a-b-c')).toBe(true);
    expect(matches('startswith=a+contains=b+endswith=c', 'a-b-d')).toBe(false);
  });
});

// --------------- Combined AND/OR ---------------

describe('combined AND/OR', () => {
  it('(A+B),C — matches A AND B', () => {
    expect(matches('startswith=test+endswith=1,other', 'test chat - 1')).toBe(true);
  });
  it('(A+B),C — matches C alone', () => {
    expect(matches('startswith=test+endswith=1,other', 'something other')).toBe(true);
  });
  it('(A+B),C — rejects when neither group matches', () => {
    expect(matches('startswith=test+endswith=1,other', 'hello world')).toBe(false);
  });
  it('A,(B+C) — matches second AND group', () => {
    expect(matches('nope,startswith=new+endswith=chat', 'new chat')).toBe(true);
  });
  it('complex: multiple AND groups ORed', () => {
    const q = 'startswith=a+endswith=z,startswith=x+endswith=y';
    expect(matches(q, 'a to z')).toBe(true);
    expect(matches(q, 'x y')).toBe(true);
    expect(matches(q, 'a y')).toBe(false); // neither group fully matches
    expect(matches(q, 'hello')).toBe(false);
  });
});

// --------------- Edge cases ---------------

describe('edge cases', () => {
  it('empty search matches everything', () => {
    expect(matches('', 'anything')).toBe(true);
  });
  it('whitespace-only search matches everything', () => {
    expect(matches('   ', 'anything')).toBe(true);
  });
  it('single comma gives two empty groups — matches all', () => {
    // split(',') on ',' gives ['', ''] — both filtered out
    expect(matches(',', 'anything')).toBe(true);
  });
  it('trailing comma is ignored', () => {
    expect(matches('test,', 'test chat')).toBe(true);
    expect(matches('test,', 'nope')).toBe(false);
  });
  it('leading comma is ignored', () => {
    expect(matches(',test', 'test chat')).toBe(true);
  });
  it('plus only gives empty group — matches all', () => {
    expect(matches('+', 'anything')).toBe(true);
  });
  it('operator with empty value is treated as always-true', () => {
    // 'startswith=' → regex captures empty string → startsWith('') is true
    // Actually this would fail regex: /^startswith=(.+)$/i — requires .+
    // So it falls through to default contains with 'startswith=' as literal text
    expect(matches('startswith=', 'startswith=')).toBe(true);
    expect(matches('startswith=', 'hello')).toBe(false);
  });
  it('paths with backslashes work', () => {
    expect(matches('startswith=C:\\repos', 'C:\\repos\\myproject')).toBe(true);
    expect(matches('endswith=myproject', 'C:\\repos\\myproject')).toBe(true);
  });
  it('paths with forward slashes work', () => {
    expect(matches('contains=/home/user', '/home/user/repos/proj')).toBe(true);
  });
});

// --------------- matchesAny empty matchers ---------------

describe('matchesAny', () => {
  it('returns true when no matchers (no filter)', () => {
    expect(matchesAny([], 'anything')).toBe(true);
  });
  it('returns false when matcher rejects', () => {
    const matchers = parseSearchTerms('xyz');
    expect(matchesAny(matchers, 'hello')).toBe(false);
  });
});

// --------------- Real-world scenarios ---------------

describe('real-world session search scenarios', () => {
  it('Dobby setup: startswith + endswith (AND)', () => {
    const q = 'startswith=C:\\dobby\\agents+endswith=_agent-cli';
    expect(matches(q, 'C:\\dobby\\agents\\weather_agent-cli')).toBe(true);
    expect(matches(q, 'C:\\dobby\\agents\\search_agent-cli')).toBe(true);
    expect(matches(q, 'C:\\dobby\\agents\\search_agent')).toBe(false);
    expect(matches(q, 'C:\\other\\agents\\weather_agent-cli')).toBe(false);
  });
  it('filter branches: release/', () => {
    expect(matches('startswith=release/', 'release/1.0')).toBe(true);
    expect(matches('startswith=release/', 'main')).toBe(false);
  });
  it('exclude test sessions', () => {
    expect(matches('!test', 'feat/dashboard')).toBe(true);
    expect(matches('!test', 'test/unit-login')).toBe(false);
  });
  it('multiple branch patterns OR', () => {
    const q = 'startswith=feat/,startswith=fix/';
    expect(matches(q, 'feat/auth')).toBe(true);
    expect(matches(q, 'fix/login-bug')).toBe(true);
    expect(matches(q, 'main')).toBe(false);
  });
});
