<script lang="ts">
  import { copy } from '../../copy';
  import { appState } from '../../state/app-state.svelte';
  import { DESTINATIONS } from '../../state/types';

  const items = DESTINATIONS.map((id) => ({
    id,
    label: copy.destinations[id],
  }));
</script>

<nav
  class="w-36 shrink-0 border-r border-border bg-surface-muted px-2 py-4 narrow:w-44"
  aria-label={copy.primaryNav}
>
  <ul class="flex flex-col gap-1">
    {#each items as item (item.id)}
      <li>
        <button
          type="button"
          class="w-full rounded-md px-3 py-2 text-left text-sm font-medium text-text hover:bg-surface"
          class:bg-surface={appState.destination === item.id}
          aria-current={appState.destination === item.id ? 'page' : undefined}
          onclick={() => (appState.destination = item.id)}
        >
          {item.label}
        </button>
      </li>
    {/each}
  </ul>
</nav>
