const API = '';

function localTimestamp() {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
         `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

export function localTodayDate() {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export async function addFeeding(amountHalfScoops, date = null) {
  const today = localTodayDate();
  const targetDate = date || today;
  const fed_at = targetDate === today ? localTimestamp() : `${targetDate} 12:00:00`;
  const body = { amount_half_scoops: amountHalfScoops, fed_at, edited: targetDate !== today };
  const res = await fetch(`${API}/api/feeding`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error('Failed to add feeding');
  return res.json();
}

export async function deleteFeeding(id) {
  const res = await fetch(`${API}/api/feeding/${id}`, { method: 'DELETE' });
  if (!res.ok) throw new Error('Failed to delete feeding');
}

export async function getCalendar(year, month) {
  const res = await fetch(`${API}/api/calendar/${year}/${month}`);
  if (!res.ok) throw new Error('Failed to fetch calendar');
  return res.json();
}

export async function getDay(date) {
  const res = await fetch(`${API}/api/day/${date}`);
  if (!res.ok) throw new Error('Failed to fetch day');
  return res.json();
}

export async function addTreat(name, date = null) {
  const today = localTodayDate();
  const targetDate = date || today;
  const given_at = targetDate === today ? localTimestamp() : `${targetDate} 12:00:00`;
  const body = { name, given_at, edited: targetDate !== today };
  const res = await fetch(`${API}/api/treat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error('Failed to add treat');
  return res.json();
}

export async function deleteTreat(id) {
  const res = await fetch(`${API}/api/treat/${id}`, { method: 'DELETE' });
  if (!res.ok) throw new Error('Failed to delete treat');
}

// ── Live updates (SSE) ──────────────────────────────────────────

let _es = null;
const _listeners = new Set();

function ensureEventSource() {
  if (_es) return;
  try {
    _es = new EventSource(`${API}/api/events`);
    _es.onmessage = () => {
      for (const cb of _listeners) {
        try { cb(); } catch (_) { /* ignore */ }
      }
    };
    // EventSource auto-reconnects on error; nothing else needed.
  } catch (_) {
    _es = null;
  }
}

/**
 * Subscribe to server "data changed" events. Returns an unsubscribe function.
 */
export function subscribeEvents(callback) {
  _listeners.add(callback);
  ensureEventSource();
  return () => {
    _listeners.delete(callback);
  };
}

// ── App version / update detection ──────────────────────────────

export async function getVersion() {
  const res = await fetch(`${API}/api/version`, { cache: 'no-store' });
  if (!res.ok) throw new Error('Failed to fetch version');
  return res.json();
}
