<script>
  import Today from './lib/Today.svelte';
  import Calendar from './lib/Calendar.svelte';
  import DayDetail from './lib/DayDetail.svelte';

  let hash = $state(window.location.hash || '#/');

  $effect(() => {
    const onHash = () => { hash = window.location.hash || '#/'; };
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  });

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
    padding-bottom: 80px;
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
</style>
