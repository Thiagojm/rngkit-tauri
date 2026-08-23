import { cleanup } from '@testing-library/svelte';
import { afterEach, vi } from 'vitest';
import { resetAppState } from './state/app-state.svelte';
import { applyTheme } from './state/theme';

vi.mock('uplot', () => {
  class FakeUPlot {
    static rangeNum(min: number, max: number): [number, number] {
      return [min, max];
    }

    root = document.createElement('div');
    over = document.createElement('div');
    ctx = {
      save() {},
      restore() {},
      beginPath() {},
      rect() {},
      clip() {},
      moveTo() {},
      lineTo() {},
      stroke() {},
      fillText() {},
      setLineDash() {},
    } as unknown as CanvasRenderingContext2D;
    bbox = { left: 0, top: 0, width: 100, height: 100 };
    scales: Record<string, { min: number; max: number }> = {
      x: { min: 0, max: 1 },
      y: { min: -2, max: 2 },
    };
    constructor(_opts: unknown, _data: unknown, target: HTMLElement) {
      this.root.className = 'uplot';
      this.over.className = 'u-over';
      this.root.append(this.over);
      target.append(this.root);
    }
    setData = vi.fn();
    setScale = vi.fn();
    setSize = vi.fn();
    destroy = vi.fn(() => {
      this.root.remove();
    });
    valToPos = () => 0;
    posToVal = () => 0;
    redraw = vi.fn();
  }

  return { default: FakeUPlot };
});

if (typeof HTMLDialogElement !== 'undefined') {
  const proto = HTMLDialogElement.prototype;
  if (typeof proto.showModal !== 'function') {
    proto.showModal = function showModal(this: HTMLDialogElement) {
      this.setAttribute('open', '');
    };
  }
  if (typeof proto.close !== 'function') {
    proto.close = function close(this: HTMLDialogElement) {
      this.removeAttribute('open');
      this.dispatchEvent(new Event('close'));
    };
  }
}

afterEach(() => {
  cleanup();
  resetAppState();
  applyTheme('system');
});
