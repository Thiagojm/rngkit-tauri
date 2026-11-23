<script lang="ts">
  import Button from './Button.svelte';

  let {
    disabled = false,
    disabledReason = '',
    label = 'Action',
    variant,
    describedBy,
  }: {
    disabled?: boolean;
    disabledReason?: string;
    label?: string;
    variant?: 'primary' | 'secondary' | 'ghost' | 'stop';
    describedBy?: string;
  } = $props();
</script>

<Button {disabled} {disabledReason} {variant} {describedBy}>{label}</Button>
