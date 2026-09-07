<script lang="ts">
  let {
    label,
    value,
    compact = false,
  }: { label: string; value: string; compact?: boolean } = $props();
</script>

<div
  class="min-w-0 rounded-md border border-border bg-surface px-3 {compact
    ? 'py-1'
    : 'py-3'}"
>
  <p class="text-sm text-text-muted">{label}</p>
  <p class="break-words font-mono text-lg text-text {compact ? '' : 'mt-1'}">
    {value}
  </p>
</div>
