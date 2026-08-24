<script lang="ts">
  import ReportActions from '../components/reports/ReportActions.svelte';
  import ReportInput from '../components/reports/ReportInput.svelte';
  import ReportPreview from '../components/reports/ReportPreview.svelte';
  import { copy } from '../copy';
</script>

<div class="flex max-w-3xl flex-col gap-4">
  <h1 class="text-2xl font-semibold">{copy.destinations.reports}</h1>
  <p class="text-text-muted">
    Inspect a native session directory, legacy v3 BIN/CSV, or derived bundle,
    then generate a same-stem XLSX report.
  </p>
  <ReportInput />
  <ReportPreview />
  <ReportActions />
</div>
