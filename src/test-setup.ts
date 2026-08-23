import { cleanup } from '@testing-library/svelte';
import { afterEach } from 'vitest';
import { resetAppState } from './state/app-state.svelte';
import { applyTheme } from './state/theme';

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
