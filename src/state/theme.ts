import type { ThemePreference } from './types';

export function applyTheme(theme: ThemePreference): void {
  document.documentElement.dataset.theme = theme;
}
