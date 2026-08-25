<script lang="ts">
  import { copy, FOLD_OPTIONS } from '../copy';
  import { ERROR_CODES } from '../ipc/types';
  import { RNGKIT_CORE_REVISION } from '../library-revision';
</script>

<article class="flex max-w-3xl flex-col gap-6">
  <h1 class="text-2xl font-semibold">{copy.destinations.help}</h1>

  <section class="flex flex-col gap-2" aria-labelledby="help-quick-start">
    <h2 id="help-quick-start" class="text-lg font-medium">Quick start</h2>
    <ol class="list-decimal space-y-2 ps-5">
      <li>
        Open Collect. New users start with the <code>Documents/RngKit</code>
        folder and 2048 sample bits. If the folder cannot be used, choose another
        output folder.
      </li>
      <li>
        Wait for sources to appear, choose exactly one source, and set the
        interval and fold when the controls are shown.
      </li>
      <li>
        Select Start. Samples continue until you select Stop or collection
        fails. The saved session folder is available when collection finishes.
      </li>
    </ol>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-choosing-source">
    <h2 id="help-choosing-source" class="text-lg font-medium">
      Choosing a source
    </h2>
    <p>
      RngKit searches for sources automatically when the app opens and supports
      one explicitly selected BitBabbler, TrueRNG, RDSEED, or PseudoRNG source
      per session. Nothing is selected automatically. Select Refresh sources
      when the list is empty or when you want to search again; a refresh can
      invalidate an earlier selection.
    </p>
    <p>BitBabbler shows these fold choices:</p>
    <ul class="list-disc space-y-1 ps-5">
      {#each FOLD_OPTIONS as option (option.value)}
        <li>{option.label}</li>
      {/each}
    </ul>
    <p>
      RDSEED and PseudoRNG use their library defaults and do not show a fold
      control. If a source disappears, refresh and choose an available source
      again; RngKit does not silently switch devices.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-collecting-safely">
    <h2 id="help-collecting-safely" class="text-lg font-medium">
      Collecting and stopping safely
    </h2>
    <p>
      Stop lets the current sample finish and save before the session closes. If
      you close the window while collecting, choose Keep collecting or Stop and
      exit. When the app is stopping, wait until it finishes.
    </p>
    <p>
      Every committed sample remains available to the chart. Use Fit all to
      frame the retained points. Zooming or panning pauses automatic following;
      Fit all resumes following while collection is active.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-creating-reports">
    <h2 id="help-creating-reports" class="text-lg font-medium">
      Creating reports
    </h2>
    <ol class="list-decimal space-y-2 ps-5">
      <li>
        In Reports, select Choose input once. You can choose a native session
        artifact, a derived bundle artifact, a current CSV or BIN, or a legacy
        v3 CSV or BIN.
      </li>
      <li>
        Review the preview and its timestamp note, then select Generate report.
        Inputs are read-only.
      </li>
      <li>
        If an XLSX already exists, Cancel keeps it. Replace is a separate,
        explicit confirmation.
      </li>
    </ol>
    <p>
      If the chosen file belongs to a folder with <code>manifest.json</code>,
      RngKit checks the complete bundle. Otherwise it validates the standalone
      file from its filename and contents. BIN-only reports use estimated
      timestamps; derived reports copy timestamps from their combined inputs.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-combining-files">
    <h2 id="help-combining-files" class="text-lg font-medium">
      Combining files
    </h2>
    <ol class="list-decimal space-y-2 ps-5">
      <li>
        In Combine, select Add files and choose CSV inputs. Repeat Add files to
        select compatible files from another folder.
      </li>
      <li>
        Review each row's format, source, sample size, interval, fold, time
        range, row count, and validation state. Remove targets one row; Clear
        all resets the selection.
      </li>
      <li>
        Select Create derived bundle only when the complete selection is
        compatible. Then use Generate XLSX for its report.
      </li>
    </ol>
    <p>
      Combine accepts current CSV, legacy v3 CSV, or a compatible mixture. It
      does not accept BIN files. Inputs must have matching source, sample size,
      interval, and fold, and their time ranges cannot overlap.
    </p>
  </section>

  <section
    class="flex flex-col gap-2"
    aria-labelledby="help-understanding-chart"
  >
    <h2 id="help-understanding-chart" class="text-lg font-medium">
      Understanding the chart
    </h2>
    <p>
      {copy.chart.boundary} The zero line and the ±1.96 lines are visual guides only.
      They do not certify randomness or produce a pass/fail result.
    </p>
    <p>
      The chart keeps every committed point for the current session. Fit all
      frames the complete retained range. During collection it follows new
      points after Fit all; after collection ends, it frames the data without
      automatically following.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-common-problems">
    <h2 id="help-common-problems" class="text-lg font-medium">
      Common problems
    </h2>
    <ul class="list-disc space-y-2 ps-5">
      <li>
        <strong>No sources:</strong> select Refresh sources. If discovery still finds
        nothing, verify the device connection or choose PseudoRNG.
      </li>
      <li>
        <strong>No output folder:</strong> select Choose folder, choose an available
        directory, and retry the action.
      </li>
      <li>
        <strong>Combine is incompatible:</strong> inspect the invalid row, remove
        it, or Clear all and select a compatible set again. The input files are not
        changed.
      </li>
      <li>
        <strong>An output already exists:</strong> use Cancel to keep the existing
        file or choose Replace only when overwriting is intended.
      </li>
      <li>
        <strong>Keyboard or display needs:</strong> all actions are available from
        the keyboard, themes are available in the top bar, and reduced motion removes
        extra chart animation.
      </li>
    </ul>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-file-formats">
    <h2 id="help-file-formats" class="text-lg font-medium">
      File formats and version details
    </h2>
    <ul class="list-disc space-y-2 ps-5">
      <li>
        A native session contains a same-stem BIN, CSV, and
        <code>manifest.json</code>.
      </li>
      <li>
        A current CSV has a seven-column header and RFC 3339 timestamps. A
        legacy v3 CSV has no header and uses compact rows such as
        <code>YYYYMMDDTHHMMSS,&lt;ones&gt;</code>. Reports also accepts
        standalone current or legacy BIN files when their filenames contain
        valid metadata.
      </li>
      <li>
        A derived bundle contains a same-stem CSV and manifest. Schema-1
        <code>legacy_csv_concatenation</code> bundles remain supported, while
        new Combine output uses schema 2 and <code>csv_concatenation</code>.
      </li>
    </ul>
    <p>RngKit application 0.1.0.</p>
    <p>
      Library revision <code>{RNGKIT_CORE_REVISION}</code>.
    </p>
    <p>Stable error codes shown in diagnostics:</p>
    <ul class="flex flex-wrap gap-x-3 gap-y-1" aria-label="Stable error codes">
      {#each ERROR_CODES as code (code)}
        <li><code>{code}</code></li>
      {/each}
    </ul>
  </section>
</article>
