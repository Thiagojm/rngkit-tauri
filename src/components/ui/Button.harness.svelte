<script lang="ts">
  import Button from './Button.svelte';

  let {
    disabled = false,
    disabledReason = '',
    label = 'Action',
  }: {
    disabled?: boolean;
    disabledReason?: string;
    label?: string;
  } = $props();
</script>

<Button {disabled} {disabledReason}>{label}</Button>
