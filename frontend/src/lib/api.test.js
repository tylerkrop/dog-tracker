import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  addFeeding,
  deleteFeeding,
  getCalendar,
  getDay,
  addTreat,
  deleteTreat,
  localTodayDate,
  subscribeEvents,
} from './api.js';

function mockFetch(response) {
  const fetch = vi.fn().mockResolvedValue(response);
  vi.stubGlobal('fetch', fetch);
  return fetch;
}

function ok(body) {
  return { ok: true, json: async () => body };
}

function fail(status = 500) {
  return { ok: false, status, json: async () => ({}) };
}

beforeEach(() => {
  vi.unstubAllGlobals();
  vi.useFakeTimers();
  // Lock "now" to 2024-05-15 12:34:56 local time for deterministic timestamps.
  vi.setSystemTime(new Date(2024, 4, 15, 12, 34, 56));
});

afterEach(() => {
  vi.useRealTimers();
});

describe('localTodayDate', () => {
  it('returns the local date in YYYY-MM-DD format', () => {
    expect(localTodayDate()).toBe('2024-05-15');
  });
});

describe('addFeeding', () => {
  it('uses the local timestamp and edited=false when no date is provided', async () => {
    const fetch = mockFetch(ok({ id: 1 }));
    await addFeeding(2);

    expect(fetch).toHaveBeenCalledTimes(1);
    const [url, init] = fetch.mock.calls[0];
    expect(url).toBe('/api/feeding');
    expect(init.method).toBe('POST');
    expect(init.headers).toEqual({ 'Content-Type': 'application/json' });
    expect(JSON.parse(init.body)).toEqual({
      amount_half_scoops: 2,
      fed_at: '2024-05-15 12:34:56',
      edited: false,
    });
  });

  it('uses noon timestamp and edited=true for a past date', async () => {
    const fetch = mockFetch(ok({ id: 1 }));
    await addFeeding(1, '2022-05-01');

    const body = JSON.parse(fetch.mock.calls[0][1].body);
    expect(body).toEqual({
      amount_half_scoops: 1,
      fed_at: '2022-05-01 12:00:00',
      edited: true,
    });
  });

  it('treats an explicit "today" date the same as no date (edited=false)', async () => {
    const fetch = mockFetch(ok({ id: 1 }));
    await addFeeding(1, '2024-05-15');

    const body = JSON.parse(fetch.mock.calls[0][1].body);
    expect(body.edited).toBe(false);
    expect(body.fed_at).toBe('2024-05-15 12:34:56');
  });

  it('throws on non-ok response', async () => {
    mockFetch(fail(400));
    await expect(addFeeding(1)).rejects.toThrow('Failed to add feeding');
  });
});

describe('deleteFeeding', () => {
  it('DELETEs by id', async () => {
    const fetch = mockFetch({ ok: true });
    await deleteFeeding(42);
    expect(fetch).toHaveBeenCalledWith('/api/feeding/42', { method: 'DELETE' });
  });

  it('throws on non-ok response', async () => {
    mockFetch(fail(404));
    await expect(deleteFeeding(1)).rejects.toThrow('Failed to delete feeding');
  });
});

describe('getCalendar', () => {
  it('GETs /api/calendar/:year/:month', async () => {
    const fetch = mockFetch(ok([]));
    await getCalendar(2024, 5);
    expect(fetch).toHaveBeenCalledWith('/api/calendar/2024/5');
  });

  it('throws on non-ok response', async () => {
    mockFetch(fail());
    await expect(getCalendar(2024, 5)).rejects.toThrow('Failed to fetch calendar');
  });
});

describe('getDay', () => {
  it('GETs /api/day/:date', async () => {
    const fetch = mockFetch(ok({ date: '2024-01-15' }));
    await getDay('2024-01-15');
    expect(fetch).toHaveBeenCalledWith('/api/day/2024-01-15');
  });

  it('throws on non-ok response', async () => {
    mockFetch(fail());
    await expect(getDay('2024-01-15')).rejects.toThrow('Failed to fetch day');
  });
});

describe('addTreat', () => {
  it('uses local timestamp and edited=false with no date', async () => {
    const fetch = mockFetch(ok({ id: 1 }));
    await addTreat('Bone');

    const body = JSON.parse(fetch.mock.calls[0][1].body);
    expect(body).toEqual({
      name: 'Bone',
      given_at: '2024-05-15 12:34:56',
      edited: false,
    });
  });

  it('uses noon timestamp and edited=true for a past date', async () => {
    const fetch = mockFetch(ok({ id: 1 }));
    await addTreat('Scraps', '2022-01-01');

    const body = JSON.parse(fetch.mock.calls[0][1].body);
    expect(body).toEqual({
      name: 'Scraps',
      given_at: '2022-01-01 12:00:00',
      edited: true,
    });
  });

  it('throws on non-ok response', async () => {
    mockFetch(fail(400));
    await expect(addTreat('Bone')).rejects.toThrow('Failed to add treat');
  });
});

describe('deleteTreat', () => {
  it('DELETEs by id', async () => {
    const fetch = mockFetch({ ok: true });
    await deleteTreat(7);
    expect(fetch).toHaveBeenCalledWith('/api/treat/7', { method: 'DELETE' });
  });

  it('throws on non-ok response', async () => {
    mockFetch(fail(404));
    await expect(deleteTreat(1)).rejects.toThrow('Failed to delete treat');
  });
});

describe('subscribeEvents', () => {
  let instances;
  let api;

  beforeEach(async () => {
    instances = [];
    class FakeEventSource {
      constructor(url) {
        this.url = url;
        this.onmessage = null;
        instances.push(this);
      }
      close() {}
    }
    vi.stubGlobal('EventSource', FakeEventSource);
    // Reset module state (`_es`, `_listeners`) between tests.
    vi.resetModules();
    api = await import('./api.js');
  });

  it('opens a single EventSource on first subscribe', () => {
    api.subscribeEvents(vi.fn());
    api.subscribeEvents(vi.fn());
    expect(instances).toHaveLength(1);
    expect(instances[0].url).toBe('/api/events');
  });

  it('invokes all listeners on each message', () => {
    const a = vi.fn();
    const b = vi.fn();
    api.subscribeEvents(a);
    api.subscribeEvents(b);
    instances[0].onmessage({ data: 'invalidate' });
    expect(a).toHaveBeenCalledTimes(1);
    expect(b).toHaveBeenCalledTimes(1);
  });

  it('returns an unsubscribe function that stops further callbacks', () => {
    const a = vi.fn();
    const b = vi.fn();
    const unsub = api.subscribeEvents(a);
    api.subscribeEvents(b);

    unsub();
    instances[0].onmessage({ data: 'invalidate' });

    expect(a).not.toHaveBeenCalled();
    expect(b).toHaveBeenCalledTimes(1);
  });

  it('swallows listener exceptions so others still run', () => {
    const a = vi.fn(() => { throw new Error('boom'); });
    const b = vi.fn();
    api.subscribeEvents(a);
    api.subscribeEvents(b);
    expect(() => instances[0].onmessage({ data: 'x' })).not.toThrow();
    expect(b).toHaveBeenCalledTimes(1);
  });
});
