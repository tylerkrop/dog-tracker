const API = '';

export async function getToday() {
  const res = await fetch(`${API}/api/today`);
  if (!res.ok) throw new Error('Failed to fetch today');
  return res.json();
}

export async function addFeeding(amountHalfScoops, date = null) {
  const body = { amount_half_scoops: amountHalfScoops };
  if (date) body.date = date;
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
  const body = { name };
  if (date) body.date = date;
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
