<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'secondary' | 'ghost' | 'stop';

  let {
    variant = 'secondary',
    disabled = false,
    disabledReason = '',
    describedBy,
    type = 'button',
    onclick,
    children,
  }: {
    variant?: Variant;
    disabled?: boolean;
    disabledReason?: string;
    /** Id of a reason rendered elsewhere; suppresses the inline reason. */
    describedBy?: string;
    type?: 'button' | 'submit';
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();

  const reasonId = $props.id();
  const inlineReason = $derived(disabled && !describedBy ? disabledReason : '');

  const variantClass = $derived(
    {
      primary:
        'bg-action text-text-inverse hover:bg-action-hover disabled:bg-surface-muted disabled:text-text-muted',
      secondary:
        'border border-border bg-surface text-text hover:bg-surface-muted disabled:border-transparent disabled:bg-surface-muted disabled:text-text-muted',
      ghost:
        'text-text hover:bg-surface-muted disabled:bg-surface-muted disabled:text-text-muted',
      stop: 'bg-action-stop text-action-stop-text hover:bg-action-stop-hover disabled:bg-surface-muted disabled:text-text-muted',
    }[variant],
  );
</script>

<div class="flex flex-col items-start gap-1">
  <button
    {type}
    class="rounded-md px-3 py-2 text-sm font-medium disabled:cursor-not-allowed {variantClass}"
    {disabled}
    aria-describedby={disabled
      ? describedBy || (inlineReason ? reasonId : undefined)
      : undefined}
    {onclick}
  >
    {@render children()}
  </button>
  {#if inlineReason}
    <p id={reasonId} class="text-sm text-text-muted">{inlineReason}</p>
  {/if}
</div>
