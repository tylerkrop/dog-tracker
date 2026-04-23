<script>
  import Today from './lib/Today.svelte';
  import Calendar from './lib/Calendar.svelte';
  import DayDetail from './lib/DayDetail.svelte';
  import { getVersion } from './lib/api.js';

  let hash = $state(window.location.hash || '#/');
  let updateAvailable = $state(false);
  let initialBuildId = null;

  $effect(() => {
    const onHash = () => { hash = window.location.hash || '#/'; };
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  });

  async function checkVersion() {
    if (updateAvailable) return;
    try {
      const v = await getVersion();
      const id = `${v.version}:${v.build_id}`;
      if (initialBuildId === null) {
        initialBuildId = id;
      } else if (id !== initialBuildId) {
        updateAvailable = true;
      }
    } catch (_) { /* offline or transient — try again next time */ }
  }

  $effect(() => {
    checkVersion();
    const onVisibility = () => {
      if (document.visibilityState === 'visible') checkVersion();
    };
    document.addEventListener('visibilitychange', onVisibility);
    return () => document.removeEventListener('visibilitychange', onVisibility);
  });

  function reload() {
    window.location.reload();
  }

  let page = $derived.by(() => {
    if (hash.startsWith('#/calendar')) return 'calendar';
    if (hash.startsWith('#/day/')) return 'day';
    return 'today';
  });

  let dayDate = $derived.by(() => {
    const m = hash.match(/#\/day\/(\d{4}-\d{2}-\d{2})/);
    return m ? m[1] : null;
  });
</script>

<div class="shell">
  {#if updateAvailable}
    <button type="button" class="update-banner" onclick={reload}>
      <span>Update available</span>
      <span class="update-action">Tap to refresh</span>
    </button>
  {/if}

  <main>
    {#if page === 'today'}
      <Today />
    {:else if page === 'calendar'}
      <Calendar />
    {:else if page === 'day' && dayDate}
      <DayDetail date={dayDate} />
    {/if}
  </main>

  <nav class="tab-bar">
    <a
      href="#/"
      class="tab"
      class:active={page === 'today'}
    >
      <span class="tab-icon">🐾</span>
      <span class="tab-label">Today</span>
    </a>
    <a
      href="#/calendar"
      class="tab"
      class:active={page === 'calendar' || page === 'day'}
    >
      <span class="tab-icon">📅</span>
      <span class="tab-label">Calendar</span>
    </a>
  </nav>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    min-height: 100dvh;
  }

  main {
    flex: 1;
    padding: 20px 16px;
    padding-bottom: calc(80px + var(--safe-bottom));
  }

  .tab-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    background: var(--surface);
    border-top: 1px solid var(--border);
    padding-bottom: var(--safe-bottom);
    z-index: 100;
  }

  .tab {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 0;
    text-decoration: none;
    color: var(--text-muted);
    font-size: 11px;
    transition: color 0.15s;
  }

  .tab.active {
    color: var(--primary);
  }

  .tab-icon {
    font-size: 22px;
  }

  .tab-label {
    font-weight: 600;
  }

  .update-banner {
    position: sticky;
    top: 0;
    z-index: 200;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    padding: 10px 16px;
    padding-top: calc(10px + var(--safe-top, env(safe-area-inset-top)));
    background: var(--primary);
    color: white;
    border: none;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
    text-align: center;
  }

  .update-action {
    opacity: 0.85;
    font-weight: 500;
  }
  .update-action::before {
    content: "· ";
  }
</style>
