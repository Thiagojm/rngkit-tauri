<script lang="ts">
  import Button from './Button.svelte';

  type Action = {
    label: string;
    variant?: 'primary' | 'secondary' | 'ghost';
    enabled: boolean;
    reason: string;
    onclick: () => void;
  };

  let { actions }: { actions: Action[] } = $props();

  const groupId = $props.id();
  // One reason line per distinct text, below the row, shared by its buttons.
  const reasons = $derived([
    ...new Set(
      actions
        .filter((action) => !action.enabled && action.reason)
        .map((action) => action.reason),
    ),
  ]);
  const reasonId = (reason: string) =>
    reasons.includes(reason)
      ? `${groupId}-reason-${reasons.indexOf(reason)}`
      : undefined;
</script>

<div class="flex flex-col gap-1">
  <div class="flex flex-wrap gap-2">
    {#each actions as action (action.label)}
      <Button
        variant={action.variant}
        disabled={!action.enabled}
        describedBy={reasonId(action.reason)}
        onclick={action.onclick}>{action.label}</Button
      >
    {/each}
  </div>
  {#each reasons as reason, index (reason)}
    <p id="{groupId}-reason-{index}" class="text-sm text-text-muted">
      {reason}
    </p>
  {/each}
</div>
