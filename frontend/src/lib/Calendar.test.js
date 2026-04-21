import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('./api.js', () => ({
  getCalendar: vi.fn(),
  subscribeEvents: vi.fn(() => () => {}),
}));

import * as api from './api.js';
import Calendar from './Calendar.svelte';

beforeEach(() => {
  vi.clearAllMocks();
  api.getCalendar.mockResolvedValue([]);
  api.subscribeEvents.mockReturnValue(() => {});
  vi.useFakeTimers();
  vi.setSystemTime(new Date(2024, 4, 15, 12, 0, 0));
});

afterEach(() => {
  vi.useRealTimers();
});

describe('Calendar', () => {
  it('shows current month and fetches its data on mount', async () => {
    render(Calendar);
    expect(await screen.findByText('May 2024')).toBeInTheDocument();
    await waitFor(() => expect(api.getCalendar).toHaveBeenCalledWith(2024, 5));
  });

  it('subscribes to live events on mount', async () => {
    render(Calendar);
    await screen.findByText('May 2024');
    expect(api.subscribeEvents).toHaveBeenCalled();
  });

  it('reloads when an SSE event fires', async () => {
    let trigger;
    api.subscribeEvents.mockImplementation((cb) => {
      trigger = cb;
      return () => {};
    });
    render(Calendar);
    await screen.findByText('May 2024');
    api.getCalendar.mockClear();

    trigger();
    await waitFor(() => expect(api.getCalendar).toHaveBeenCalledWith(2024, 5));
  });

  it('prev button navigates to previous month and refetches', async () => {
    render(Calendar);
    await screen.findByText('May 2024');
    api.getCalendar.mockClear();

    await fireEvent.click(screen.getByLabelText('Previous month'));

    expect(await screen.findByText('April 2024')).toBeInTheDocument();
    await waitFor(() => expect(api.getCalendar).toHaveBeenCalledWith(2024, 4));
  });

  it('next button navigates to next month', async () => {
    render(Calendar);
    await screen.findByText('May 2024');
    api.getCalendar.mockClear();

    await fireEvent.click(screen.getByLabelText('Next month'));

    expect(await screen.findByText('June 2024')).toBeInTheDocument();
    await waitFor(() => expect(api.getCalendar).toHaveBeenCalledWith(2024, 6));
  });

  it('prev from January wraps to previous December', async () => {
    vi.setSystemTime(new Date(2024, 0, 15));
    render(Calendar);
    await screen.findByText('January 2024');

    await fireEvent.click(screen.getByLabelText('Previous month'));
    expect(await screen.findByText('December 2023')).toBeInTheDocument();
  });

  it('next from December wraps to following January', async () => {
    vi.setSystemTime(new Date(2024, 11, 15));
    render(Calendar);
    await screen.findByText('December 2024');

    await fireEvent.click(screen.getByLabelText('Next month'));
    expect(await screen.findByText('January 2025')).toBeInTheDocument();
  });

  it('renders scoop totals and treat marker for days with data', async () => {
    api.getCalendar.mockResolvedValue([
      { date: '2024-05-01', total_scoops: 1.5, treat_count: 2 },
      { date: '2024-05-10', total_scoops: 1, treat_count: 0 },
    ]);
    render(Calendar);
    expect(await screen.findByText('1.5*')).toBeInTheDocument();
    expect(await screen.findByText('1', { selector: '.day-scoops' })).toBeInTheDocument();
  });

  it('selecting a day updates the summary card', async () => {
    api.getCalendar.mockResolvedValue([
      { date: '2024-05-10', total_scoops: 2, treat_count: 3 },
    ]);
    render(Calendar);
    await screen.findByText('May 2024');

    const dayButtons = screen
      .getAllByRole('button')
      .filter((b) => b.querySelector('.day-num')?.textContent === '10');
    expect(dayButtons).toHaveLength(1);
    await fireEvent.click(dayButtons[0]);

    await waitFor(() => {
      expect(screen.getByText('2', { selector: '.stat-value' })).toBeInTheDocument();
      expect(screen.getByText('3', { selector: '.stat-value.treat-value' })).toBeInTheDocument();
      expect(screen.getByText(/Friday, May 10/)).toBeInTheDocument();
    });
  });

  it('selecting a day with no data shows zero stats', async () => {
    render(Calendar);
    await screen.findByText('May 2024');

    const dayButtons = screen
      .getAllByRole('button')
      .filter((b) => b.querySelector('.day-num')?.textContent === '20');
    await fireEvent.click(dayButtons[0]);

    await waitFor(() => {
      const zeros = screen.getAllByText('0', { selector: '.stat-value' });
      expect(zeros.length).toBeGreaterThanOrEqual(1);
    });
  });

  it('View Details navigates to day route for non-today selection', async () => {
    render(Calendar);
    await screen.findByText('May 2024');

    const day1 = screen
      .getAllByRole('button')
      .filter((b) => b.querySelector('.day-num')?.textContent === '1')[0];
    await fireEvent.click(day1);

    window.location.hash = '';
    await fireEvent.click(screen.getByText('View Details'));
    expect(window.location.hash).toBe('#/day/2024-05-01');
  });

  it('View Details navigates to today route when today is selected', async () => {
    render(Calendar);
    await screen.findByText('May 2024');
    // Today (the 15th) is selected by default on mount.

    window.location.hash = '';
    await fireEvent.click(screen.getByText('View Details'));
    expect(window.location.hash).toBe('#/');
  });
});
