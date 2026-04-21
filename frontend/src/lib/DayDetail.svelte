<script>
  import { onMount } from 'svelte';
  import { getDay, addFeeding, deleteFeeding, addTreat, deleteTreat, subscribeEvents } from './api.js';

  let { date } = $props();

  let data = $state(null);
  let loading = $state(true);
  let busy = $state(false);
  let customTreatInput = $state('');
  let showCustomInput = $state(false);

  async function load() {
    loading = true;
    try { data = await getDay(date); }
    finally { loading = false; }
  }

  async function feed(halfScoops) {
    if (busy) return;
    busy = true;
    try {
      await addFeeding(halfScoops, date);
      await load();
    } finally { busy = false; }
  }

  async function remove(id) {
    if (busy) return;
    busy = true;
    try {
      await deleteFeeding(id);
      await load();
    } finally { busy = false; }
  }

  async function giveTreat(name) {
    if (busy) return;
    busy = true;
    try {
      await addTreat(name, date);
      await load();
    } finally { busy = false; }
  }

  async function removeTreat(id) {
    if (busy) return;
    busy = true;
    try {
      await deleteTreat(id);
      await load();
    } finally { busy = false; }
  }

  async function submitCustomTreat() {
    const name = customTreatInput.trim();
    if (!name) return;
    await giveTreat(name);
    customTreatInput = '';
    showCustomInput = false;
  }

  function handleCustomKeydown(e) {
    if (e.key === 'Enter') submitCustomTreat();
    if (e.key === 'Escape') { showCustomInput = false; customTreatInput = ''; }
  }

  function formatTime(dateStr) {
    const d = new Date(dateStr.replace(' ', 'T'));
    return d.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });
  }

  function prettyDate(dateStr) {
    const d = new Date(dateStr + 'T00:00:00');
    return d.toLocaleDateString([], { weekday: 'long', month: 'long', day: 'numeric' });
  }

  function scoopLabel(half) {
    return half === 1 ? '½' : '1';
  }

  $effect(() => {
    void date;
    load();
  });

  onMount(() => subscribeEvents(() => { load(); }));
</script>

<a href="#/calendar" class="back-link">‹ Calendar</a>

{#if loading && !data}
  <div class="center"><span class="spinner"></span></div>
{:else if data}
  <header class="day-header">
    <h1>{prettyDate(data.date)}</h1>
  </header>

  <div class="scoop-display">
    <span class="scoop-number">{data.total_scoops}</span>
    <span class="scoop-unit">{data.total_scoops === 1 ? 'scoop' : 'scoops'}</span>
  </div>

  {#if data.feedings.length === 0 && data.treats.length === 0}
    <p class="empty">No feedings recorded</p>
  {:else}
    {#if data.feedings.length > 0}
      <ul class="feeding-list">
        {#each data.feedings as f (f.id)}
          <li class="feeding-item">
            {#if f.edited}
              <span class="feeding-time edited-badge">edited</span>
            {:else}
              <span class="feeding-time">{formatTime(f.fed_at)}</span>
            {/if}
            <span class="feeding-amount">{scoopLabel(f.amount_half_scoops)} scoop</span>
            <button class="delete-btn" onclick={() => remove(f.id)} aria-label="Remove">✕</button>
          </li>
        {/each}
      </ul>
    {/if}

    {#if data.treats.length > 0}
      <h3 class="section-label">Treats</h3>
      <ul class="feeding-list">
        {#each data.treats as t (t.id)}
          <li class="feeding-item">
            {#if t.edited}
              <span class="feeding-time edited-badge">edited</span>
            {:else}
              <span class="feeding-time">{formatTime(t.given_at)}</span>
            {/if}
            <span class="feeding-amount treat-name">{t.name}</span>
            <button class="delete-btn" onclick={() => removeTreat(t.id)} aria-label="Remove">✕</button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}

  <div class="actions">
    <button class="feed-btn half" onclick={() => feed(1)} disabled={busy}>+½</button>
    <button class="feed-btn full" onclick={() => feed(2)} disabled={busy}>+1</button>
  </div>

  <div class="treat-actions">
    <button class="treat-btn" onclick={() => giveTreat('Bone')} disabled={busy}>🦴 Bone</button>
    <button class="treat-btn" onclick={() => giveTreat('Pup Cup')} disabled={busy}>🍦 Pup Cup</button>
    <button class="treat-btn" onclick={() => giveTreat('Scraps')} disabled={busy}>🍖 Scraps</button>
    <button class="treat-btn custom-toggle" onclick={() => { showCustomInput = !showCustomInput }} disabled={busy}>✏️ Custom</button>
  </div>

  {#if showCustomInput}
    <div class="custom-input-row">
      <input
        type="text"
        class="custom-input"
        placeholder="Treat name…"
        bind:value={customTreatInput}
        onkeydown={handleCustomKeydown}
      />
      <button class="custom-submit" onclick={submitCustomTreat} disabled={busy || !customTreatInput.trim()}>Add</button>
    </div>
  {/if}
{/if}

<style>
  .back-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--primary);
    text-decoration: none;
    font-size: 16px;
    font-weight: 600;
    padding: 4px 0;
    margin-bottom: 8px;
  }

  .center {
    display: flex;
    justify-content: center;
    padding: 60px 0;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  .day-header {
    text-align: center;
    margin-bottom: 8px;
  }

  .day-header h1 {
    font-size: 20px;
    font-weight: 700;
  }

  .scoop-display {
    text-align: center;
    padding: 28px 0 24px;
  }

  .scoop-number {
    font-size: 72px;
    font-weight: 800;
    color: var(--primary);
    line-height: 1;
  }

  .scoop-unit {
    display: block;
    font-size: 18px;
    color: var(--text-muted);
    margin-top: 4px;
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: 24px 0;
    font-size: 15px;
  }

  .feeding-list {
    list-style: none;
    background: var(--surface);
    border-radius: var(--radius);
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  }

  .feeding-item {
    display: flex;
    align-items: center;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }

  .feeding-item:last-child {
    border-bottom: none;
  }

  .feeding-time {
    font-size: 15px;
    font-weight: 600;
    min-width: 90px;
  }

  .feeding-amount {
    flex: 1;
    font-size: 15px;
    color: var(--text-muted);
  }

  .section-label {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 20px 0 8px 4px;
  }

  .treat-name {
    color: var(--text) !important;
    font-weight: 500;
  }

  .edited-badge {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent) !important;
    font-style: italic;
  }

  .delete-btn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: none;
    color: var(--text-muted);
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
  }

  .delete-btn:active {
    background: rgba(204,68,68,0.1);
    color: var(--danger);
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-top: 24px;
    padding: 0 4px;
  }

  .feed-btn {
    flex: 1;
    padding: 18px 0;
    border-radius: var(--radius);
    font-size: 28px;
    font-weight: 800;
    color: white;
    transition: transform 0.1s, opacity 0.15s;
  }

  .feed-btn:active:not(:disabled) {
    transform: scale(0.96);
  }

  .feed-btn:disabled {
    opacity: 0.5;
  }

  .feed-btn.half {
    background: var(--accent);
  }

  .feed-btn.full {
    background: var(--primary);
  }

  .treat-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-top: 12px;
    padding: 0 4px;
  }

  .treat-btn {
    padding: 14px 0;
    border-radius: var(--radius);
    font-size: 15px;
    font-weight: 700;
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 1px 3px rgba(0,0,0,0.06);
    transition: transform 0.1s, opacity 0.15s;
  }

  .treat-btn:active:not(:disabled) {
    transform: scale(0.96);
  }

  .treat-btn:disabled {
    opacity: 0.5;
  }

  .custom-input-row {
    display: flex;
    gap: 8px;
    margin-top: 10px;
    padding: 0 4px;
  }

  .custom-input {
    flex: 1;
    padding: 12px 14px;
    border: 2px solid var(--border);
    border-radius: var(--radius);
    font-size: 16px;
    font-family: inherit;
    background: var(--surface);
    color: var(--text);
    outline: none;
    transition: border-color 0.15s;
  }

  .custom-input:focus {
    border-color: var(--primary);
  }

  .custom-submit {
    padding: 12px 20px;
    border-radius: var(--radius);
    font-size: 15px;
    font-weight: 700;
    background: var(--primary);
    color: white;
    transition: opacity 0.15s;
  }

  .custom-submit:disabled {
    opacity: 0.4;
  }
</style>
