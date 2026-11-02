import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { Tooltip } from './Tooltip';

afterEach(cleanup);

describe('Tooltip scrolling', () => {
  it('dismisses hover cards as successive rows pass under the pointer', () => {
    render(<div data-testid="table">{['First', 'Second', 'Last'].map(label =>
      <Tooltip key={label} trigger={label}>{label} risk evidence</Tooltip>,
    )}</div>);
    for (const label of ['Last', 'Second', 'First']) {
      fireEvent.mouseEnter(screen.getByRole('button', { name: label }));
      expect(screen.getAllByRole('tooltip')).toHaveLength(1);
      fireEvent.scroll(screen.getByTestId('table'));
      expect(screen.queryAllByRole('tooltip')).toHaveLength(0);
    }
  });

  it('does not revive a card when mouse leave and scroll occur together', () => {
    render(<Tooltip trigger="Risk">Evidence</Tooltip>);
    const trigger = screen.getByRole('button');
    fireEvent.mouseEnter(trigger);
    act(() => {
      fireEvent.mouseLeave(trigger);
      fireEvent.scroll(window);
    });
    expect(screen.queryAllByRole('tooltip')).toHaveLength(0);
  });

  it('keeps keyboard-focused evidence open while scrolling, and closes on Escape', () => {
    render(<Tooltip trigger="Risk">Evidence</Tooltip>);
    const trigger = screen.getByRole('button');
    act(() => trigger.focus());
    fireEvent.scroll(window);
    expect(screen.getAllByRole('tooltip')).toHaveLength(1);
    fireEvent.keyDown(trigger, { key: 'Escape' });
    fireEvent.scroll(window);
    expect(screen.queryAllByRole('tooltip')).toHaveLength(0);
  });

  it('allows scrolling within the evidence card', () => {
    render(<Tooltip trigger="Risk">Evidence</Tooltip>);
    fireEvent.mouseEnter(screen.getByRole('button'));
    fireEvent.scroll(screen.getByRole('tooltip'));
    expect(screen.getAllByRole('tooltip')).toHaveLength(1);
  });
});
