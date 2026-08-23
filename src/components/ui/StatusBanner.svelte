<script lang="ts">
  import { statusTone } from '../../state/controls';
  import type { CollectionState } from '../../state/types';

  let {
    state,
    label,
    detail,
  }: {
    state: CollectionState;
    label: string;
    detail?: string | null;
  } = $props();

  const icon = $derived(
    {
      idle: '○',
      discovering: '…',
      ready: '●',
      collecting: '▶',
      stopping: '■',
      completed: '✓',
      failed: '!',
    }[state],
  );
</script>

<div
  class="flex items-start gap-3 rounded-md border border-border bg-surface-muted px-4 py-3"
  role="status"
  aria-live="polite"
>
  <span aria-hidden="true" class="mt-0.5 font-mono {statusTone(state)}"
    >{icon}</span
  >
  <div class="min-w-0">
    <p class="font-medium {statusTone(state)}">{label}</p>
    {#if detail}
      <p class="mt-1 text-sm text-text-muted">{detail}</p>
    {/if}
  </div>
</div>
