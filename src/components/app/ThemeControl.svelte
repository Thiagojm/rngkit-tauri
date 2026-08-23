<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import { THEME_PREFERENCES, type ThemePreference } from '../../state/types';

  function onThemeChange(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if ((THEME_PREFERENCES as readonly string[]).includes(value)) {
      appState.setTheme(value as ThemePreference);
    }
  }
</script>

<label class="flex items-center gap-2 text-sm">
  {copy.theme.legend}
  <select
    class="rounded-md border border-border bg-surface px-2 py-1"
    value={appState.theme}
    onchange={onThemeChange}
  >
    {#each THEME_PREFERENCES as preference (preference)}
      <option value={preference}>{copy.theme[preference]}</option>
    {/each}
  </select>
</label>
