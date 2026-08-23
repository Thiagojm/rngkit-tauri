<script lang="ts">
  let { label, value }: { label: string; value: string } = $props();
</script>

<div class="rounded-md border border-border bg-surface px-3 py-3">
  <p class="text-sm text-text-muted">{label}</p>
  <p class="mt-1 font-mono text-lg text-text">{value}</p>
</div>
