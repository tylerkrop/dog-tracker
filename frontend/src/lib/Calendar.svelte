<script>
  import { getCalendar } from './api.js';

  const MONTH_NAMES = [
    'January','February','March','April','May','June',
    'July','August','September','October','November','December',
  ];
  const DAY_LABELS = ['S','M','T','W','T','F','S'];

  let now = new Date();
  let year = $state(now.getFullYear());
  let month = $state(now.getMonth() + 1);
  let calendarData = $state([]);
  let loading = $state(false);
  let selectedDay = $state(now.getDate());

  let loadSeq = { current: 0 };

  async function load() {
    const seq = ++loadSeq.current;
    loading = true;
    try {
      const result = await getCalendar(year, month);
      if (seq === loadSeq.current) calendarData = result;
    } finally {
      if (seq === loadSeq.current) loading = false;
    }
  }

  function prev() {
    if (month === 1) { month = 12; year--; }
    else { month--; }
    selectedDay = 1;
  }

  function next() {
    if (month === 12) { month = 1; year++; }
    else { month++; }
    selectedDay = 1;
  }

  $effect(() => {
    void year; void month;
    load();
  });

  let weeks = $derived.by(() => {
    const days = new Date(year, month, 0).getDate();
    const first = new Date(year, month - 1, 1).getDay();
    const result = [];
    let week = Array(first).fill(null);

    for (let d = 1; d <= days; d++) {
      week.push(d);
      if (week.length === 7) { result.push(week); week = []; }
    }
    if (week.length) {
      while (week.length < 7) week.push(null);
      result.push(week);
    }
    return result;
  });

  let scoopMap = $derived.by(() => {
    const map = {};
    for (const e of calendarData) {
      map[e.date] = { scoops: e.total_scoops, treats: e.treat_count };
    }
    return map;
  });

  function dateStr(day) {
    return `${year}-${String(month).padStart(2,'0')}-${String(day).padStart(2,'0')}`;
  }

  function isToday(day) {
    if (!day) return false;
    const t = new Date();
    return year === t.getFullYear() && month === t.getMonth() + 1 && day === t.getDate();
  }

  function goToDay(day) {
    selectedDay = day;
  }

  let selectedInfo = $derived.by(() => {
    if (!selectedDay) return null;
    const ds = dateStr(selectedDay);
    return scoopMap[ds] || { scoops: 0, treats: 0 };
  });

  function prettySelectedDate() {
    if (!selectedDay) return '';
    const d = new Date(year, month - 1, selectedDay);
    return d.toLocaleDateString([], { weekday: 'long', month: 'long', day: 'numeric' });
  }

  function viewDetails() {
    if (!selectedDay) return;
    if (isToday(selectedDay)) {
      window.location.hash = '#/';
    } else {
      window.location.hash = `#/day/${dateStr(selectedDay)}`;
    }
  }
</script>

<header class="cal-header">
  <button class="nav-btn" onclick={prev} aria-label="Previous month">‹</button>
  <h2>{MONTH_NAMES[month - 1]} {year}</h2>
  <button class="nav-btn" onclick={next} aria-label="Next month">›</button>
</header>

<div class="cal-grid">
  {#each DAY_LABELS as lbl}
    <div class="day-label">{lbl}</div>
  {/each}

  {#each weeks as week}
    {#each week as day}
      {#if day}
        {@const info = scoopMap[dateStr(day)]}
        <button
          class="day-cell"
          class:today={isToday(day)}
          class:has-data={info != null}
          class:selected={selectedDay === day}
          onclick={() => goToDay(day)}
        >
          <span class="day-num">{day}</span>
          {#if info}
            <span class="day-scoops">{info.scoops}{#if info.treats > 0}*{/if}</span>
          {/if}
        </button>
      {:else}
        <div class="day-cell empty"></div>
      {/if}
    {/each}
  {/each}
</div>

{#if selectedDay}
  <div class="summary-card">
    <h3 class="summary-date">{prettySelectedDate()}</h3>
    <div class="summary-stats">
      <div class="stat">
        <span class="stat-value">{selectedInfo.scoops}</span>
        <span class="stat-label">{selectedInfo.scoops === 1 ? 'scoop' : 'scoops'}</span>
      </div>
      <div class="stat">
        <span class="stat-value treat-value">{selectedInfo.treats}</span>
        <span class="stat-label">{selectedInfo.treats === 1 ? 'treat' : 'treats'}</span>
      </div>
    </div>
    <button class="details-btn" onclick={viewDetails}>View Details</button>
  </div>
{/if}

<style>
  .cal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .cal-header h2 {
    font-size: 20px;
    font-weight: 700;
  }

  .nav-btn {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--surface);
    font-size: 24px;
    color: var(--primary);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  }

  .nav-btn:active {
    background: var(--border);
  }

  .cal-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
  }

  .day-label {
    text-align: center;
    font-size: 12px;
    font-weight: 700;
    color: var(--text-muted);
    padding: 8px 0;
  }

  .day-cell {
    aspect-ratio: 1;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: var(--surface);
    gap: 2px;
    font-size: 14px;
    transition: transform 0.1s;
  }

  .day-cell.empty {
    background: transparent;
  }

  .day-cell:not(.empty):active {
    transform: scale(0.92);
  }

  .day-cell.today {
    border: 2px solid var(--primary);
  }

  .day-cell.has-data {
    background: var(--primary-light);
    color: white;
  }

  .day-num {
    font-weight: 600;
    line-height: 1;
  }

  .day-scoops {
    font-size: 10px;
    font-weight: 700;
    opacity: 0.85;
  }

  .day-cell.selected {
    box-shadow: 0 0 0 2px var(--accent);
  }

  .summary-card {
    margin-top: 20px;
    background: var(--surface);
    border-radius: var(--radius);
    padding: 20px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.06);
    text-align: center;
  }

  .summary-date {
    font-size: 16px;
    font-weight: 700;
    margin-bottom: 16px;
  }

  .summary-stats {
    display: flex;
    justify-content: center;
    gap: 40px;
    margin-bottom: 18px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .stat-value {
    font-size: 36px;
    font-weight: 800;
    color: var(--primary);
    line-height: 1;
  }

  .stat-value.treat-value {
    color: var(--accent);
  }

  .stat-label {
    font-size: 14px;
    color: var(--text-muted);
    margin-top: 4px;
  }

  .details-btn {
    width: 100%;
    padding: 14px;
    border-radius: var(--radius);
    background: var(--primary);
    color: white;
    font-size: 16px;
    font-weight: 700;
    transition: transform 0.1s;
  }

  .details-btn:active {
    transform: scale(0.97);
  }
</style>
