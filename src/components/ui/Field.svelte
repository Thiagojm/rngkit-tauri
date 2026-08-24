<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    id,
    label,
    hint,
    group = false,
    children,
  }: {
    id: string;
    label: string;
    hint?: string;
    group?: boolean;
    children: Snippet;
  } = $props();

  const labelId = $derived(`${id}-label`);
  const hintId = $derived(`${id}-hint`);
</script>

<div class="flex flex-col gap-1">
  {#if group}
    <span id={labelId} class="text-sm font-medium text-text">{label}</span>
  {:else}
    <label class="text-sm font-medium text-text" for={id}>{label}</label>
  {/if}
  <div
    class="field-control"
    role={group ? 'group' : undefined}
    aria-labelledby={group ? labelId : undefined}
    aria-describedby={hint ? hintId : undefined}
  >
    {@render children()}
  </div>
  {#if hint}
    <p id={hintId} class="text-sm text-text-muted">{hint}</p>
  {/if}
</div>
