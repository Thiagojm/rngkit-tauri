<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'secondary' | 'ghost';

  let {
    variant = 'secondary',
    disabled = false,
    disabledReason = '',
    type = 'button',
    onclick,
    children,
  }: {
    variant?: Variant;
    disabled?: boolean;
    disabledReason?: string;
    type?: 'button' | 'submit';
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();

  const reasonId = $props.id();

  const variantClass = $derived(
    {
      primary:
        'bg-action text-text-inverse hover:bg-action-hover disabled:bg-surface-muted disabled:text-text-muted',
      secondary:
        'border border-border bg-surface text-text hover:bg-surface-muted disabled:text-text-muted',
      ghost: 'text-text hover:bg-surface-muted disabled:text-text-muted',
    }[variant],
  );
</script>

<div class="flex flex-col items-stretch gap-1">
  <button
    {type}
    class="rounded-md px-3 py-2 text-sm font-medium {variantClass}"
    {disabled}
    aria-describedby={disabled && disabledReason ? reasonId : undefined}
    {onclick}
  >
    {@render children()}
  </button>
  {#if disabled && disabledReason}
    <p id={reasonId} class="text-sm text-text-muted">{disabledReason}</p>
  {/if}
</div>
