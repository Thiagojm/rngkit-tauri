<script lang="ts">
  import { copy, FOLD_OPTIONS } from '../copy';
  import { RNGKIT_CORE_REVISION } from '../library-revision';
</script>

<article class="flex max-w-3xl flex-col gap-6">
  <h1 class="text-2xl font-semibold">{copy.destinations.help}</h1>

  <section class="flex flex-col gap-2" aria-labelledby="help-sources">
    <h2 id="help-sources" class="text-lg font-medium">Sources</h2>
    <p>
      v1 collects from one explicitly selected BitBabbler, TrueRNG, RDSEED, or
      PseudoRNG source per session. Multiple physical devices stay separate
      choices. Nothing is selected automatically, and there is no live XOR,
      fallback, reconnect, or resume.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-folds">
    <h2 id="help-folds" class="text-lg font-medium">{copy.fold.label}</h2>
    <p>
      BitBabbler sessions require a fold. The control uses these exact labels:
    </p>
    <ul class="list-disc ps-5">
      {#each FOLD_OPTIONS as option (option.value)}
        <li>{option.label}</li>
      {/each}
    </ul>
    <p>
      RDSEED and PseudoRNG use library defaults and do not show the fold
      control.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-collection">
    <h2 id="help-collection" class="text-lg font-medium">Collection</h2>
    <p>
      Configure {copy.sampleBits}, {copy.sampleInterval}, output folder, and
      fold when required, then start. Collection continues until you stop it or
      a terminal error occurs. Stop is cooperative and idempotent. Closing the
      window while collecting asks whether to keep collecting or stop and exit.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-files">
    <h2 id="help-files" class="text-lg font-medium">File formats</h2>
    <p>
      A native session is a same-stem BIN, CSV, and <code>manifest.json</code>.
      Reports also accept RngKitPSG v3 BIN or CSV. Combine creates a derived CSV
      plus manifest bundle with its own directory grammar; it is not a collected
      session and never modifies the input files.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-reports">
    <h2 id="help-reports" class="text-lg font-medium">Reports</h2>
    <p>
      XLSX reports are generated from validated native, legacy v3, or derived
      inputs. Legacy v3 BIN and CSV files stay read-only. BIN-only reports use
      estimated timestamps. Derived reports copy timestamps from concatenated
      inputs. If the destination already exists, Cancel is the default; Replace
      requires an explicit second request.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-stats">
    <h2 id="help-stats" class="text-lg font-medium">Descriptive statistics</h2>
    <p>{copy.chart.boundary}</p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-access">
    <h2 id="help-access" class="text-lg font-medium">Accessibility</h2>
    <p>
      Every workflow is usable from the keyboard. Status text and icons stay
      visible together. Theme colors keep readable contrast. The window minimum
      is 800 by 600 logical pixels. Chart drawing does not add extra motion when
      the system requests reduced motion.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-codes">
    <h2 id="help-codes" class="text-lg font-medium">Troubleshooting codes</h2>
    <p>
      Safe application codes cover invalid configuration, expired selection,
      unavailable, busy, disconnected, or timed-out sources, permission failure,
      existing output, corrupt or unsupported input, operation conflict, and
      unexpected failure. Copied diagnostics never include entropy, seeds,
      serials, OS device paths, or absolute legacy input paths.
    </p>
  </section>

  <section class="flex flex-col gap-2" aria-labelledby="help-versions">
    <h2 id="help-versions" class="text-lg font-medium">Versions</h2>
    <p>RngKit application 0.1.0.</p>
    <p>
      Library revision <code>{RNGKIT_CORE_REVISION}</code>.
    </p>
  </section>
</article>
