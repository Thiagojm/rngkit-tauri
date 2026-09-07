import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import { copy } from '../copy';
import { ERROR_CODES } from '../ipc/types';
import { RNGKIT_CORE_REVISION } from '../library-revision';
import { MOCK_SCENARIOS } from '../state/mock-scenarios';
import { appState } from '../state/app-state.svelte';
import HelpPage from './HelpPage.svelte';

function setTauri(enabled: boolean): void {
  const host = globalThis as { isTauri?: boolean };
  if (enabled) {
    host.isTauri = true;
  } else {
    delete host.isTauri;
  }
}

describe('HelpPage', () => {
  afterEach(() => {
    setTauri(false);
    clearMocks();
  });

  it('presents the approved task-oriented workflow', () => {
    render(HelpPage);

    expect(
      screen.getByRole('heading', { name: copy.destinations.help }),
    ).toBeTruthy();
    for (const heading of [
      'Quick start',
      'Choosing a source',
      'Collecting and stopping safely',
      'Creating reports',
      'Combining files',
      'Understanding the chart',
      'Common problems',
      'File formats and version details',
    ]) {
      expect(screen.getByRole('heading', { name: heading })).toBeTruthy();
    }
    expect(screen.getByText(copy.fold.raw)).toBeTruthy();
    expect(
      screen.getByText(
        /Z shows balance over time; it does not certify randomness/,
      ),
    ).toBeTruthy();
    expect(screen.getByText(RNGKIT_CORE_REVISION)).toBeTruthy();
    expect(
      screen.getAllByText(/Nothing is selected automatically/i).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByText(
        /searches for sources automatically when the app opens/i,
      ),
    ).toBeTruthy();
    expect(screen.getByText(/Inputs are read-only/i)).toBeTruthy();
    expect(screen.getByText(/new Combine output uses schema 2/i)).toBeTruthy();
    expect(screen.getByText(/YYYYMMDDTHHMMSS/)).toBeTruthy();
    expect(
      screen.getAllByText(/BIN-only reports use sample numbers/i),
    ).toHaveLength(1);
    expect(screen.getByText(/outcome dialog appears once/i)).toBeTruthy();
    expect(
      screen.getByText(/canonical flat legacy concatenation CSV/i),
    ).toBeTruthy();
    expect(screen.queryByText(/estimated timestamps/i)).toBeNull();
    for (const code of ERROR_CODES) {
      expect(screen.getByText(code)).toBeTruthy();
    }
    expect(screen.queryByText(/p-value/i)).toBeNull();
    expect(screen.queryByText(/desktop side/i)).toBeNull();
    expect(screen.queryByText(/authoritative/i)).toBeNull();
    expect(
      screen.getByRole('heading', { name: copy.deviceSetup }),
    ).toBeTruthy();
    for (const summary of [
      'Windows / BitBabbler',
      'Windows / TrueRNG3',
      'Ubuntu-Debian / BitBabbler',
      'Ubuntu-Debian / TrueRNG3',
    ]) {
      expect(screen.getByText(summary)).toBeTruthy();
    }
    expect(screen.getAllByText('0403:7840').length).toBeGreaterThan(0);
    expect(screen.getAllByText('04d8:f5fe').length).toBeGreaterThan(0);
    expect(
      screen.getAllByText(/Linux hardware acceptance is pending/).length,
    ).toBeGreaterThan(0);
    expect(screen.getByText(/ATTR\{idVendor\}=="0403"/)).toBeTruthy();
    expect(screen.getByText(/ENV\{ID_MM_DEVICE_IGNORE\}="1"/)).toBeTruthy();
    expect(screen.getAllByText(/setup-rng-devices\.sh/).length).toBeGreaterThan(
      0,
    );
    expect(screen.queryByText(/bundled helper/i)).toBeNull();
    expect(
      screen.getAllByText(/bash setup-rng-devices\.sh --check/),
    ).toHaveLength(2);
    expect(
      screen.getByText(/Windows driver package is not included/),
    ).toBeTruthy();
    expect(
      screen.getByRole('button', { name: copy.openDeviceSetupFolder }),
    ).toBeTruthy();
  });
  it('provides topic links and current collection guidance', async () => {
    render(HelpPage);
    const links = screen
      .getByRole('navigation', { name: 'On this page' })
      .querySelectorAll('a');
    expect(links).toHaveLength(8);
    for (const link of links) {
      const target = document.querySelector(link.getAttribute('href')!);
      expect(target?.textContent?.trim()).toBe(link.textContent?.trim());
    }
    const heading = screen.getByRole('heading', { name: 'Creating reports' });
    heading.scrollIntoView = () => {};
    await fireEvent.click(
      screen.getByRole('link', { name: 'Creating reports' }),
    );
    expect(document.activeElement).toBe(heading);
    expect(
      screen.getByText(/positive whole number divisible by 8/),
    ).toBeTruthy();
    expect(screen.getByText(/enter whole seconds greater than 0/)).toBeTruthy();
    expect(screen.getByText(/collection has not started/)).toBeTruthy();
    expect(
      screen.getAllByText(/restore.*missing original file/i).length,
    ).toBeGreaterThan(0);
    for (const detail of document.querySelectorAll('details'))
      expect(detail.open).toBe(false);
  });

  it('focuses Device setup from a pending Collect jump and consumes it', async () => {
    HTMLElement.prototype.scrollIntoView = () => {};
    appState.helpFocusId = 'help-device-setup';
    render(HelpPage);
    const heading = screen.getByRole('heading', { name: copy.deviceSetup });
    await waitFor(() => {
      expect(document.activeElement).toBe(heading);
    });
    expect(appState.helpFocusId).toBeNull();
  });

  it('toggles a Device setup disclosure without leaving Choosing a source', async () => {
    render(HelpPage);
    const summary = screen.getByText('Windows / BitBabbler');
    const details = summary.closest('details');
    expect(details?.open).toBe(false);
    await fireEvent.click(summary);
    expect(details?.open).toBe(true);
    expect(screen.getByText(/Bind/)).toBeTruthy();
    expect(
      screen.getByRole('heading', { name: 'Choosing a source' }),
    ).toBeTruthy();
  });

  it('opens the device setup folder without a path and shows pending then success', async () => {
    setTauri(true);
    let finish: ((value: unknown) => void) | undefined;
    mockIPC((cmd, payload) => {
      expect(cmd).toBe('open_device_setup_folder');
      expect(JSON.stringify(payload ?? {})).not.toMatch(/path/i);
      return new Promise((resolve) => {
        finish = resolve;
      });
    });
    render(HelpPage);
    const button = screen.getByRole('button', {
      name: copy.openDeviceSetupFolder,
    });
    await fireEvent.click(button);
    await waitFor(() => {
      expect(button).toHaveProperty('disabled', true);
    });
    expect(screen.getByText(copy.openingDeviceSetupFolder)).toBeTruthy();
    finish?.(MOCK_SCENARIOS.idle);
    await waitFor(() => {
      expect(button).toHaveProperty('disabled', false);
    });
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('shows safe error feedback when the device setup folder cannot open', async () => {
    setTauri(true);
    mockIPC(() => {
      throw {
        code: 'invalid_configuration',
        message: 'The device setup folder is unavailable.',
        recovery: 'Reinstall RngKit if the support files are missing.',
      };
    });
    render(HelpPage);
    await fireEvent.click(
      screen.getByRole('button', { name: copy.openDeviceSetupFolder }),
    );
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain(
      'The device setup folder is unavailable.',
    );
    expect(alert.textContent).toContain(
      'Reinstall RngKit if the support files are missing.',
    );
  });

  it('does not open a folder from the browser mock', async () => {
    render(HelpPage);
    await fireEvent.click(
      screen.getByRole('button', { name: copy.openDeviceSetupFolder }),
    );
    expect(screen.queryByRole('alert')).toBeNull();
    expect(
      screen.getByRole('button', { name: copy.openDeviceSetupFolder }),
    ).toHaveProperty('disabled', false);
  });
});
