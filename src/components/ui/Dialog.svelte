<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    open = false,
    dismissible = true,
    onClose,
    children,
    actions,
  }: {
    title: string;
    open?: boolean;
    dismissible?: boolean;
    onClose: () => void;
    children: Snippet;
    actions: Snippet;
  } = $props();

  let dialog = $state<HTMLDialogElement | undefined>(undefined);
  const titleId = $props.id();

  $effect(() => {
    const el = dialog;
    if (!el) {
      return;
    }
    if (open) {
      if (!el.open) {
        if (typeof el.showModal === 'function') {
          el.showModal();
        } else {
          el.setAttribute('open', '');
        }
      }
    } else if (el.open) {
      if (typeof el.close === 'function') {
        el.close();
      } else {
        el.removeAttribute('open');
      }
    }
  });
</script>

<dialog
  bind:this={dialog}
  class="m-auto w-[calc(100%_-_2rem)] max-w-lg rounded-md border border-border bg-surface p-6 text-text shadow-lg backdrop:bg-surface-inverse/40"
  aria-labelledby={titleId}
  oncancel={(event) => {
    if (!dismissible) {
      event.preventDefault();
    }
  }}
  onclose={() => {
    if (dismissible) {
      onClose();
    }
  }}
>
  <h2 id={titleId} class="text-lg font-semibold">{title}</h2>
  <div class="mt-3 text-sm text-text-muted">
    {@render children()}
  </div>
  <div class="mt-6 flex flex-wrap justify-end gap-2">
    {@render actions()}
  </div>
</dialog>
