import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';

vi.mock('./api.js', () => ({
  getDay: vi.fn(),
  addFeeding: vi.fn(),
  deleteFeeding: vi.fn(),
  addTreat: vi.fn(),
  deleteTreat: vi.fn(),
  subscribeEvents: vi.fn(() => () => {}),
  localTodayDate: vi.fn(() => '2024-05-15'),
}));

import * as api from './api.js';
import Today from './Today.svelte';

const TODAY = '2024-05-15';

function makeData(overrides = {}) {
  return {
    date: TODAY,
    total_scoops: 0,
    feedings: [],
    treats: [],
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  api.localTodayDate.mockReturnValue(TODAY);
  api.subscribeEvents.mockReturnValue(() => {});
  api.getDay.mockResolvedValue(makeData());
});

describe('Today', () => {
  it('loads today via getDay(localTodayDate()) on mount', async () => {
    render(Today);
    await screen.findByText('No feedings yet today');
    expect(api.getDay).toHaveBeenCalledWith(TODAY);
  });

  it('subscribes to live events on mount', async () => {
    render(Today);
    await screen.findByText('No feedings yet today');
    expect(api.subscribeEvents).toHaveBeenCalled();
  });

  it('reloads when an SSE event fires', async () => {
    let trigger;
    api.subscribeEvents.mockImplementation((cb) => {
      trigger = cb;
      return () => {};
    });
    render(Today);
    await screen.findByText('No feedings yet today');
    api.getDay.mockClear();

    trigger();
    await waitFor(() => expect(api.getDay).toHaveBeenCalledWith(TODAY));
  });

  it('renders feedings and treats when present', async () => {
    api.getDay.mockResolvedValue(makeData({
      total_scoops: 1.5,
      feedings: [
        { id: 1, amount_half_scoops: 2, fed_at: `${TODAY} 08:00:00`, edited: false },
        { id: 2, amount_half_scoops: 1, fed_at: `${TODAY} 12:00:00`, edited: false },
      ],
      treats: [
        { id: 9, name: 'Bone', given_at: `${TODAY} 09:00:00`, edited: false },
      ],
    }));
    render(Today);
    expect(await screen.findByText('1.5')).toBeInTheDocument();
    expect(screen.getByText('Treats')).toBeInTheDocument();
    expect(screen.getByText('Bone')).toBeInTheDocument();
    expect(screen.getAllByLabelText('Remove')).toHaveLength(3);
  });

  it('uses singular "scoop" label when total is exactly 1', async () => {
    api.getDay.mockResolvedValue(makeData({ total_scoops: 1 }));
    render(Today);
    expect(await screen.findByText('scoop')).toBeInTheDocument();
  });

  it('+½ button calls addFeeding(1) and reloads', async () => {
    api.addFeeding.mockResolvedValue({});
    render(Today);
    await screen.findByText('No feedings yet today');
    api.getDay.mockClear();

    await fireEvent.click(screen.getByText('+½'));

    expect(api.addFeeding).toHaveBeenCalledWith(1);
    await waitFor(() => expect(api.getDay).toHaveBeenCalled());
  });

  it('+1 button calls addFeeding(2)', async () => {
    api.addFeeding.mockResolvedValue({});
    render(Today);
    await screen.findByText('No feedings yet today');

    await fireEvent.click(screen.getByText('+1'));
    expect(api.addFeeding).toHaveBeenCalledWith(2);
  });

  it('preset treat buttons call addTreat with their name', async () => {
    api.addTreat.mockResolvedValue({});
    for (const name of ['Bone', 'Pup Cup', 'Scraps']) {
      api.addTreat.mockClear();
      const { unmount } = render(Today);
      await screen.findByText('No feedings yet today');
      await fireEvent.click(screen.getByText(new RegExp(name)));
      await waitFor(() => expect(api.addTreat).toHaveBeenCalledWith(name));
      unmount();
    }
  });

  it('delete button on a feeding calls deleteFeeding with its id', async () => {
    api.getDay.mockResolvedValue(makeData({
      total_scoops: 1,
      feedings: [{ id: 42, amount_half_scoops: 2, fed_at: `${TODAY} 08:00:00`, edited: false }],
    }));
    api.deleteFeeding.mockResolvedValue();
    render(Today);
    await screen.findByText('1');

    await fireEvent.click(screen.getByLabelText('Remove'));
    expect(api.deleteFeeding).toHaveBeenCalledWith(42);
  });

  it('delete button on a treat calls deleteTreat with its id', async () => {
    api.getDay.mockResolvedValue(makeData({
      treats: [{ id: 7, name: 'Bone', given_at: `${TODAY} 09:00:00`, edited: false }],
    }));
    api.deleteTreat.mockResolvedValue();
    render(Today);
    await screen.findByText('Bone');

    await fireEvent.click(screen.getByLabelText('Remove'));
    expect(api.deleteTreat).toHaveBeenCalledWith(7);
  });

  it('custom treat input submits trimmed value and clears', async () => {
    api.addTreat.mockResolvedValue({});
    const user = userEvent.setup();
    render(Today);
    await screen.findByText('No feedings yet today');

    await user.click(screen.getByText(/Custom/));
    const input = await screen.findByPlaceholderText('Treat name…');
    await user.type(input, '  Carrot  ');
    await user.click(screen.getByText('Add'));

    expect(api.addTreat).toHaveBeenCalledWith('Carrot');
    await waitFor(() =>
      expect(screen.queryByPlaceholderText('Treat name…')).not.toBeInTheDocument()
    );
  });

  it('custom treat input does not submit empty/whitespace value', async () => {
    const user = userEvent.setup();
    render(Today);
    await screen.findByText('No feedings yet today');

    await user.click(screen.getByText(/Custom/));
    const input = await screen.findByPlaceholderText('Treat name…');
    await user.type(input, '   ');

    const addBtn = screen.getByText('Add');
    expect(addBtn).toBeDisabled();
    await user.keyboard('{Enter}');
    expect(api.addTreat).not.toHaveBeenCalled();
  });

  it('Escape key closes custom treat input and clears value', async () => {
    const user = userEvent.setup();
    render(Today);
    await screen.findByText('No feedings yet today');

    await user.click(screen.getByText(/Custom/));
    const input = await screen.findByPlaceholderText('Treat name…');
    await user.type(input, 'Carrot');
    await user.keyboard('{Escape}');

    await waitFor(() =>
      expect(screen.queryByPlaceholderText('Treat name…')).not.toBeInTheDocument()
    );
    expect(api.addTreat).not.toHaveBeenCalled();
  });

  it('Enter key in custom treat input submits', async () => {
    api.addTreat.mockResolvedValue({});
    const user = userEvent.setup();
    render(Today);
    await screen.findByText('No feedings yet today');

    await user.click(screen.getByText(/Custom/));
    const input = await screen.findByPlaceholderText('Treat name…');
    await user.type(input, 'Carrot{Enter}');

    expect(api.addTreat).toHaveBeenCalledWith('Carrot');
  });

  // Regression: tapping "Custom" used to render the input behind the fixed
  // tab-bar, leaving it un-focusable and feeling frozen. The input now
  // auto-focuses and scrolls into view as soon as it appears.
  describe('custom input visibility', () => {
    it('auto-focuses the input when the Custom toggle is opened', async () => {
      const user = userEvent.setup();
      render(Today);
      await screen.findByText('No feedings yet today');

      await user.click(screen.getByText(/Custom/));
      const input = await screen.findByPlaceholderText('Treat name…');

      await waitFor(() => expect(document.activeElement).toBe(input));
    });

    it('scrolls the input into view when the Custom toggle is opened', async () => {
      const scrollSpy = vi.fn();
      // jsdom does not implement scrollIntoView; install a spy on the prototype.
      const original = Element.prototype.scrollIntoView;
      Element.prototype.scrollIntoView = scrollSpy;
      try {
        const user = userEvent.setup();
        render(Today);
        await screen.findByText('No feedings yet today');

        await user.click(screen.getByText(/Custom/));
        await screen.findByPlaceholderText('Treat name…');

        await waitFor(() => expect(scrollSpy).toHaveBeenCalled());
        const args = scrollSpy.mock.calls[0][0];
        expect(args).toMatchObject({ block: 'center' });
      } finally {
        Element.prototype.scrollIntoView = original;
      }
    });
  });
});
