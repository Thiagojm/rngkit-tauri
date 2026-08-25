<script lang="ts">
  import CombineActions from '../components/combine/CombineActions.svelte';
  import CompatibilitySummary from '../components/combine/CompatibilitySummary.svelte';
  import InputTable from '../components/combine/InputTable.svelte';
  import { copy } from '../copy';
</script>

<div class="flex min-w-0 flex-col gap-4">
  <h1 class="text-2xl font-semibold">{copy.destinations.combine}</h1>
  <p class="text-text-muted">
    Add compatible current or legacy CSV files from one or more folders and
    create a provenance-bearing derived bundle without modifying the inputs.
  </p>
  <CombineActions />
  <InputTable />
  <CompatibilitySummary />
</div>
