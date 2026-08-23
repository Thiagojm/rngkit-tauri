<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from './components/app/AppShell.svelte';
  import { appState } from './state/app-state.svelte';

  onMount(() => {
    void appState.hydrate();
  });
</script>

<AppShell />
