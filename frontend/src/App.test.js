import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

vi.mock('./lib/api.js', () => ({
  getDay: vi.fn().mockResolvedValue({
    date: '2024-05-15',
    total_scoops: 0,
    feedings: [],
    treats: [],
  }),
  getCalendar: vi.fn().mockResolvedValue([]),
  addFeeding: vi.fn(),
  deleteFeeding: vi.fn(),
  addTreat: vi.fn(),
  deleteTreat: vi.fn(),
  subscribeEvents: vi.fn(() => () => {}),
  localTodayDate: vi.fn(() => '2024-05-15'),
}));

import App from './App.svelte';

beforeEach(() => {
  window.location.hash = '';
});

describe('App routing', () => {
  it('renders the Today view by default', async () => {
    window.location.hash = '#/';
    render(App);
    expect(await screen.findByText('No feedings yet today')).toBeInTheDocument();
  });

  it('renders the Calendar view at #/calendar', async () => {
    window.location.hash = '#/calendar';
    render(App);
    await waitFor(() => {
      expect(document.querySelector('.cal-header h2')).not.toBeNull();
    });
  });

  it('renders the DayDetail view at #/day/:date', async () => {
    window.location.hash = '#/day/2024-05-10';
    render(App);
    expect(await screen.findByText('No feedings recorded')).toBeInTheDocument();
    expect(document.querySelector('a.back-link')).not.toBeNull();
  });

  it('updates the page when the hash changes', async () => {
    window.location.hash = '#/';
    render(App);
    await screen.findByText('No feedings yet today');

    window.location.hash = '#/calendar';
    window.dispatchEvent(new HashChangeEvent('hashchange'));

    await waitFor(() => {
      expect(document.querySelector('.cal-header h2')).not.toBeNull();
    });
  });

  it('marks the Today tab active on the today route', async () => {
    window.location.hash = '#/';
    render(App);
    await screen.findByText('No feedings yet today');
    const todayTab = screen.getByText('Today').closest('a');
    expect(todayTab).toHaveClass('active');
  });

  it('marks the Calendar tab active on calendar and day routes', async () => {
    window.location.hash = '#/day/2024-05-10';
    render(App);
    await screen.findByText('No feedings recorded');
    const calTab = screen.getByText('Calendar').closest('a');
    expect(calTab).toHaveClass('active');
  });
});
