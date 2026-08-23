<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from './components/app/AppShell.svelte';
  import { appState } from './state/app-state.svelte';

  onMount(() => {
    void appState.hydrate();
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void appState.listenForClose().then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisten = stop;
      }
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

<AppShell />
