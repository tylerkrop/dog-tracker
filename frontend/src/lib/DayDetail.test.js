import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('./api.js', () => ({
  getDay: vi.fn(),
  addFeeding: vi.fn(),
  deleteFeeding: vi.fn(),
  addTreat: vi.fn(),
  deleteTreat: vi.fn(),
  subscribeEvents: vi.fn(() => () => {}),
}));

import * as api from './api.js';
import DayDetail from './DayDetail.svelte';

const DATE = '2022-03-10';

function makeData(overrides = {}) {
  return {
    date: DATE,
    total_scoops: 0,
    feedings: [],
    treats: [],
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  api.getDay.mockResolvedValue(makeData());
  api.subscribeEvents.mockReturnValue(() => {});
});

describe('DayDetail', () => {
  it('fetches data for the date prop on mount', async () => {
    render(DayDetail, { props: { date: DATE } });
    await waitFor(() => expect(api.getDay).toHaveBeenCalledWith(DATE));
    expect(await screen.findByText('No feedings recorded')).toBeInTheDocument();
  });

  it('subscribes to live events on mount', async () => {
    render(DayDetail, { props: { date: DATE } });
    await screen.findByText('No feedings recorded');
    expect(api.subscribeEvents).toHaveBeenCalled();
  });

  it('renders an "edited" badge for entries with edited=true', async () => {
    api.getDay.mockResolvedValue(makeData({
      total_scoops: 1,
      feedings: [
        { id: 1, amount_half_scoops: 2, fed_at: `${DATE} 12:00:00`, edited: true },
      ],
      treats: [
        { id: 9, name: 'Bone', given_at: `${DATE} 12:00:00`, edited: true },
      ],
    }));
    render(DayDetail, { props: { date: DATE } });
    const badges = await screen.findAllByText('edited');
    expect(badges).toHaveLength(2);
  });

  it('+½ button passes the historical date to addFeeding', async () => {
    api.addFeeding.mockResolvedValue();
    render(DayDetail, { props: { date: DATE } });
    await screen.findByText('No feedings recorded');

    await fireEvent.click(screen.getByText('+½'));
    expect(api.addFeeding).toHaveBeenCalledWith(1, DATE);
  });

  it('+1 button passes the historical date to addFeeding', async () => {
    api.addFeeding.mockResolvedValue();
    render(DayDetail, { props: { date: DATE } });
    await screen.findByText('No feedings recorded');

    await fireEvent.click(screen.getByText('+1'));
    expect(api.addFeeding).toHaveBeenCalledWith(2, DATE);
  });

  it('preset treat buttons pass the historical date to addTreat', async () => {
    api.addTreat.mockResolvedValue();
    render(DayDetail, { props: { date: DATE } });
    await screen.findByText('No feedings recorded');

    await fireEvent.click(screen.getByText(/Bone/));
    expect(api.addTreat).toHaveBeenCalledWith('Bone', DATE);
  });

  it('delete button on a feeding calls deleteFeeding', async () => {
    api.getDay.mockResolvedValue(makeData({
      total_scoops: 1,
      feedings: [{ id: 11, amount_half_scoops: 2, fed_at: `${DATE} 12:00:00`, edited: false }],
    }));
    api.deleteFeeding.mockResolvedValue();
    render(DayDetail, { props: { date: DATE } });
    await screen.findByText('1');

    await fireEvent.click(screen.getByLabelText('Remove'));
    expect(api.deleteFeeding).toHaveBeenCalledWith(11);
  });

  it('delete button on a treat calls deleteTreat', async () => {
    api.getDay.mockResolvedValue(makeData({
      treats: [{ id: 22, name: 'Bone', given_at: `${DATE} 12:00:00`, edited: false }],
    }));
    api.deleteTreat.mockResolvedValue();
    render(DayDetail, { props: { date: DATE } });
    await screen.findByText('Bone');

    await fireEvent.click(screen.getByLabelText('Remove'));
    expect(api.deleteTreat).toHaveBeenCalledWith(22);
  });

  it('renders a back link to the calendar', async () => {
    render(DayDetail, { props: { date: DATE } });
    const link = await screen.findByText(/Calendar/);
    expect(link.closest('a')).toHaveAttribute('href', '#/calendar');
  });

  it('refetches when the date prop changes', async () => {
    const { rerender } = render(DayDetail, { props: { date: DATE } });
    await waitFor(() => expect(api.getDay).toHaveBeenCalledWith(DATE));

    api.getDay.mockClear();
    api.getDay.mockResolvedValue(makeData({ date: '2022-04-01' }));
    await rerender({ date: '2022-04-01' });

    await waitFor(() => expect(api.getDay).toHaveBeenCalledWith('2022-04-01'));
  });
});
