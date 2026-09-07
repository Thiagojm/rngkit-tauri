<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { copy, FOLD_OPTIONS } from '../copy';
  import { ERROR_CODES } from '../ipc/types';
  import { RNGKIT_CORE_REVISION } from '../library-revision';
  import { appState } from '../state/app-state.svelte';

  const topics = [
    ['help-quick-start', 'Quick start'],
    ['help-choosing-source', 'Choosing a source'],
    ['help-collecting-safely', 'Collecting and stopping safely'],
    ['help-creating-reports', 'Creating reports'],
    ['help-combining-files', 'Combining files'],
    ['help-understanding-chart', 'Understanding the chart'],
    ['help-common-problems', 'Common problems'],
    ['help-file-formats', 'File formats and version details'],
  ];

  function goToTopic(event: MouseEvent, id: string): void {
    event.preventDefault();
    focusHeading(id);
  }

  function focusHeading(id: string): void {
    const heading = document.getElementById(id);
    heading?.focus({ preventScroll: true });
    heading?.scrollIntoView({ behavior: 'instant', block: 'start' });
  }

  onMount(() => {
    const id = appState.helpFocusId;
    appState.helpFocusId = null;
    if (!id) return;
    void tick().then(() => focusHeading(id));
  });
</script>

<div class="help-page @container max-w-5xl">
  <header class="mb-6">
    <h1 class="text-2xl font-semibold">{copy.destinations.help}</h1>
    <p class="mt-2 text-text-muted">
      Collect samples, create reports, and find answers to common questions.
    </p>
  </header>
  <div class="grid min-w-0 gap-6 @[56rem]:grid-cols-[12rem_minmax(0,1fr)]">
    <nav
      aria-label="On this page"
      class="self-start rounded-md border border-border bg-surface-muted p-4"
    >
      <p class="mb-3 text-sm font-semibold">On this page</p>
      <ul class="flex flex-wrap gap-2 @[56rem]:flex-col">
        {#each topics as [id, title] (id)}
          <li>
            <a
              href={'#' + id}
              onclick={(event) => goToTopic(event, id)}
              class="block rounded-sm px-2 py-1 text-sm text-text-muted hover:bg-surface hover:text-text"
              >{title}</a
            >
          </li>
        {/each}
      </ul>
    </nav>
    <article class="min-w-0 space-y-8 leading-relaxed">
      <section
        class="quick-start flex flex-col gap-3 rounded-md border border-border bg-surface-muted p-5"
        aria-labelledby="help-quick-start"
      >
        <h2 tabindex="-1" id="help-quick-start" class="text-lg font-medium">
          Quick start
        </h2>
        <ol class="list-decimal space-y-3 ps-5">
          <li>
            <strong>Choose a source.</strong> Open Collect, wait for discovery, and
            select one source.
          </li>
          <li>
            <strong>Set up your session.</strong> Check the sample size,
            interval, and fold when shown. New users start with 2048 bits and
            <code>Documents/RngKit</code>; saved settings take precedence.
            Choose another output folder if needed.
          </li>
          <li>
            <strong>Start, then stop.</strong> Select Start to collect samples
            and Stop to finish saving. Find the saved files with
            <strong>Open session folder</strong> in Session.
          </li>
        </ol>
      </section>

      <section
        class="flex flex-col gap-2"
        aria-labelledby="help-choosing-source"
      >
        <h2 tabindex="-1" id="help-choosing-source" class="text-lg font-medium">
          Choosing a source
        </h2>
        <p>
          RngKit searches for sources automatically when the app opens and
          supports one explicitly selected BitBabbler, TrueRNG, RDSEED, or
          PseudoRNG source per session. Nothing is selected automatically.
          Select <strong>Refresh sources</strong> when the list is empty or when you
          want to search again; a refresh can invalidate an earlier selection.
        </p>
        <p>BitBabbler shows these fold choices:</p>
        <ul class="flex flex-wrap gap-2">
          {#each FOLD_OPTIONS as option (option.value)}
            <li class="rounded-sm border border-border px-3 py-1 text-sm">
              {option.label}
            </li>
          {/each}
        </ul>
        <p>
          RDSEED and PseudoRNG use their library defaults and do not show a fold
          control. PseudoRNG needs no device setup. RDSEED appears only when the
          CPU provides it. If a source disappears, refresh and choose an
          available source again; RngKit does not silently switch devices.
        </p>
        <h3
          tabindex="-1"
          id="help-device-setup"
          class="mt-2 text-base font-medium"
        >
          {copy.deviceSetup}
        </h3>
        <p>
          Windows and Ubuntu-Debian guides are both here; the app does not pick
          one from your operating system. Linux hardware acceptance is pending.
          RngKit never starts an installer, terminal, or script.
        </p>
        <div class="space-y-2">
          <details>
            <summary>Windows / BitBabbler</summary>
            <div class="space-y-3 p-4">
              <p>
                Use this only for a BitBabbler with USB ID
                <code>0403:7840</code>. Bind <strong>WinUSB</strong> to that device.
                Do not replace an arbitrary FTDI driver with WinUSB.
              </p>
              <ol class="list-decimal space-y-2 ps-5">
                <li>
                  Use <code>zadig-2.9.exe</code> from the device-setup kit, or the
                  same version from the official Zadig site. You need permission to
                  change the driver for this device. RngKit never starts Zadig.
                </li>
                <li>
                  Plug in the BitBabbler. In Zadig, enable Options → List All
                  Devices and select the BitBabbler interface
                  <code>0403:7840</code>.
                </li>
                <li>
                  Install WinUSB for that interface, not the FTDI VCP driver.
                </li>
                <li>
                  Unplug the device, wait a moment, then plug it in again.
                </li>
                <li>
                  In Collect, select <strong>Refresh sources</strong>, choose
                  the BitBabbler, then Start and Stop to confirm a session is
                  saved.
                </li>
              </ol>
            </div>
          </details>
          <details>
            <summary>Windows / TrueRNG3</summary>
            <div class="space-y-3 p-4">
              <p>
                Use this only for TrueRNG v1/v2/v3 with USB ID
                <code>04d8:f5fe</code>. Install the manufacturer CDC /
                <code>usbser</code> driver for that ID. Do not bind WinUSB to this
                device.
              </p>
              <ol class="list-decimal space-y-2 ps-5">
                <li>
                  Use the manufacturer TrueRNG INF and CAT in the device-setup
                  kit for <code>04d8:f5fe</code>. You need permission to install
                  them. RngKit never starts the installer.
                </li>
                <li>
                  Plug in the TrueRNG3 and install that CDC / usbser driver for
                  this device only.
                </li>
                <li>
                  Unplug the device, wait a moment, then plug it in again.
                </li>
                <li>
                  In Collect, select <strong>Refresh sources</strong>, choose
                  TrueRNG, then Start and Stop to confirm a session is saved.
                </li>
              </ol>
            </div>
          </details>
          <details>
            <summary>Ubuntu-Debian / BitBabbler</summary>
            <div class="space-y-3 p-4">
              <p>
                These steps only set USB permissions on Ubuntu or Debian with
                systemd/udev. Linux hardware acceptance is pending. The
                device-setup kit includes <code>setup-rng-devices.sh</code>. Run
                it yourself; RngKit never starts it.
              </p>
              <ol class="list-decimal space-y-2 ps-5">
                <li>
                  From the kit <code>linux</code> folder, run
                  <code
                    >./setup-rng-devices.sh --check --device bitbabbler --user
                    YOUR_USER</code
                  >
                  then the same command with sudo and without
                  <code>--check</code>.
                </li>
                <li>
                  The helper creates group <code>rngkit</code> if needed, adds
                  your login user, and installs
                  <code>/etc/udev/rules.d/60-rngkit-bitbabbler.rules</code> as a
                  root-owned 0644 file. It does not chmod the rules directory,
                  overwrite a different existing file, unload
                  <code>ftdi_sio</code>, or trigger every device.
                </li>
                <li>
                  The rule matches only USB device nodes <code>0403:7840</code>
                  with group <code>rngkit</code> and mode 0660:
                  <code
                    >{'SUBSYSTEM=="usb", ATTR{idVendor}=="0403", ATTR{idProduct}=="7840", MODE="0660", GROUP="rngkit"'}</code
                  >.
                </li>
                <li>
                  Log out and back in, reconnect the BitBabbler, then in Collect
                  select <strong>Refresh sources</strong>, choose the
                  BitBabbler, and Start and Stop to confirm a session is saved.
                </li>
              </ol>
            </div>
          </details>
          <details>
            <summary>Ubuntu-Debian / TrueRNG3</summary>
            <div class="space-y-3 p-4">
              <p>
                These steps only set tty permissions on Ubuntu or Debian with
                systemd/udev. Linux hardware acceptance is pending. RngKit
                configures serial settings; do not add OS serial hooks or a
                shared symlink. Run <code>setup-rng-devices.sh</code> yourself; RngKit
                never starts it.
              </p>
              <ol class="list-decimal space-y-2 ps-5">
                <li>
                  From the kit <code>linux</code> folder, run
                  <code
                    >./setup-rng-devices.sh --check --device truerng3 --user
                    YOUR_USER</code
                  >
                  then the same command with sudo and without
                  <code>--check</code>.
                </li>
                <li>
                  The helper uses group <code>rngkit</code> and installs
                  <code>/etc/udev/rules.d/60-rngkit-truerng3.rules</code> as a root-owned
                  0644 file. It leaves a different existing file unchanged.
                </li>
                <li>
                  The rule ignores ModemManager only for USB ID
                  <code>04d8:f5fe</code> and matches only that device's tty
                  nodes with group <code>rngkit</code> and mode 0660:
                  <code
                    >{'ACTION=="add", SUBSYSTEM=="usb", ENV{DEVTYPE}=="usb_device", ATTR{idVendor}=="04d8", ATTR{idProduct}=="f5fe", ENV{ID_MM_DEVICE_IGNORE}="1"'}</code
                  >
                  and
                  <code
                    >{'SUBSYSTEM=="tty", ATTRS{idVendor}=="04d8", ATTRS{idProduct}=="f5fe", ENV{ID_MM_PORT_IGNORE}="1", MODE="0660", GROUP="rngkit"'}</code
                  >.
                </li>
                <li>
                  Log out and back in, reconnect the TrueRNG3, then in Collect
                  select <strong>Refresh sources</strong>, choose TrueRNG, and
                  Start and Stop to confirm a session is saved.
                </li>
              </ol>
            </div>
          </details>
        </div>
      </section>

      <section
        class="flex flex-col gap-2"
        aria-labelledby="help-collecting-safely"
      >
        <h2
          tabindex="-1"
          id="help-collecting-safely"
          class="text-lg font-medium"
        >
          Collecting and stopping safely
        </h2>
        <p>
          Stop lets the current sample finish and save before the session
          closes. If you close the window while collecting, choose Keep
          collecting or <strong>Stop and exit</strong>. When the app is
          stopping, wait until it finishes.
        </p>
        <div class="rounded-md border border-border p-3">
          <p>
            <strong>Sample size:</strong> enter a positive whole number divisible
            by 8, such as 8, 1024, or 2048 bits.
          </p>
          <p>
            <strong>Sample interval:</strong> enter whole seconds greater than 0,
            such as 1, 2, or 10.
          </p>
          <p>
            Invalid settings open a correction dialog; collection has not
            started. Correct the indicated field and select <strong
              >Start</strong
            > again.
          </p>
        </div>
        <p>
          When collection finishes, use <strong>Open session folder</strong> or
          <strong>Start another session</strong> in Session.
        </p>
      </section>

      <section
        class="flex flex-col gap-2"
        aria-labelledby="help-creating-reports"
      >
        <h2
          tabindex="-1"
          id="help-creating-reports"
          class="text-lg font-medium"
        >
          Creating reports
        </h2>
        <ol class="list-decimal space-y-2 ps-5">
          <li>
            In Reports, select <strong>Choose input</strong> once. You can choose
            a native session artifact, a derived bundle artifact, a current CSV or
            BIN, or a legacy v3 CSV or BIN.
          </li>
          <li>
            Review the preview and its timestamp note, then select Generate
            report. Inputs are read-only.
          </li>
          <li>
            If an XLSX already exists, Cancel keeps it. Replace is a separate,
            explicit confirmation.
          </li>
        </ol>
        <p>
          If the chosen file belongs to a folder with <code>manifest.json</code
          >, RngKit checks the complete bundle. A native session needs its
          matching BIN and CSV files; restore any missing original file before
          retrying. If there is no manifest, it validates the standalone file
          from its filename and contents. BIN-only reports use sample numbers on
          the chart's horizontal axis; CSV-based reports use recorded
          timestamps. A BIN with a valid sibling CSV can use the recorded
          timestamps from that bundle. Derived reports copy timestamps from
          their combined inputs.
        </p>
        <p>
          After generation, an outcome dialog appears once with the saved file
          path. Open the report or its folder using the available buttons, or
          select <strong>Dismiss</strong>.
        </p>
      </section>

      <section
        class="flex flex-col gap-2"
        aria-labelledby="help-combining-files"
      >
        <h2 tabindex="-1" id="help-combining-files" class="text-lg font-medium">
          Combining files
        </h2>
        <ol class="list-decimal space-y-2 ps-5">
          <li>
            In Combine, select <strong>Add files</strong> and choose CSV inputs.
            Repeat <strong>Add files</strong>
            to select compatible files from another folder.
          </li>
          <li>
            Review each row's format, source, sample size, interval, fold, time
            range, row count, and validation state. Remove targets one row;
            <strong>Clear all</strong> resets the selection.
          </li>
          <li>
            Select <strong>Create derived bundle</strong> only when the complete
            selection is compatible. Then use <strong>Generate XLSX</strong> for its
            report.
          </li>
        </ol>
        <p>
          Combine accepts current CSV, legacy v3 CSV, or a compatible mixture.
          It does not accept BIN files. Inputs must have matching source, sample
          size, interval, and fold, and their time ranges cannot overlap.
        </p>
        <p>
          The result dialog lists the saved CSV, manifest, and folder paths. Use
          its folder button to find the new bundle. Your input files are
          unchanged.
        </p>
      </section>

      <section
        class="flex flex-col gap-2"
        aria-labelledby="help-understanding-chart"
      >
        <h2
          tabindex="-1"
          id="help-understanding-chart"
          class="text-lg font-medium"
        >
          Understanding the chart
        </h2>
        <p>
          {copy.chart.boundary} The zero line and the ±1.96 lines are visual guides
          only. They do not certify randomness or produce a pass/fail result.
        </p>
        <p>
          The chart keeps every committed point for the current session. Zooming
          or panning pauses automatic following. <strong>Fit all</strong> frames
          the complete retained range. During collection it follows new points
          after <strong>Fit all</strong>; after collection ends, it frames the
          data without automatically following.
        </p>
      </section>

      <section
        class="flex flex-col gap-2"
        aria-labelledby="help-common-problems"
      >
        <h2 tabindex="-1" id="help-common-problems" class="text-lg font-medium">
          Common problems
        </h2>
        <div class="space-y-2">
          <details>
            <summary>No sources appear</summary>
            <p>
              Select <strong>Refresh sources</strong>. Check the device
              connection if the list remains empty, or select PseudoRNG when
              available. Nothing is selected automatically.
            </p>
          </details>
          <details>
            <summary>Sample settings are invalid</summary>
            <p>
              Bits must be a positive whole multiple of 8. The interval must be
              a positive whole number of seconds. Correct the field named in the
              dialog, then select <strong>Start</strong>.
            </p>
          </details>
          <details>
            <summary>The output folder is unavailable</summary>
            <p>
              Select <strong>Choose folder</strong>, choose an available
              directory, and retry.
            </p>
          </details>
          <details>
            <summary>A report bundle is incomplete</summary>
            <p>
              If the folder contains <code>manifest.json</code>, Reports checks
              the complete bundle. A native session needs its matching BIN and
              CSV. Restore the missing original file, then choose the input
              again. A standalone BIN is a different input type and needs a
              valid filename; without a timestamp CSV, its chart uses sample
              numbers.
            </p>
          </details>
          <details>
            <summary>Combine says the files are incompatible</summary>
            <p>
              Check the invalid row. Source, sample size, interval, and fold
              must match, and time ranges cannot overlap. Use <strong
                >Remove</strong
              >
              or <strong>Clear all</strong> and select a compatible set. The input
              files are not changed.
            </p>
          </details>
          <details>
            <summary>A report already exists</summary>
            <p>
              <strong>Cancel</strong> keeps the existing XLSX. Choose
              <strong>Replace</strong> only when you intend to overwrite it.
            </p>
          </details>
          <details>
            <summary>Where are my saved files?</summary>
            <p>
              Use <strong>Open working folder</strong> in Collect, Reports, or
              Combine. For a finished collection, use
              <strong>Open session folder</strong> in Session. Result dialogs also
              show saved paths and available opening actions.
            </p>
          </details>
          <details>
            <summary>Keyboard and display options</summary>
            <p>
              Use Tab to move between controls and Enter or Space to activate
              them. Choose a theme in the top bar. Reduced motion removes extra
              chart animation.
            </p>
          </details>
        </div>
      </section>

      <section class="flex flex-col gap-2" aria-labelledby="help-file-formats">
        <h2 tabindex="-1" id="help-file-formats" class="text-lg font-medium">
          File formats and version details
        </h2>
        <details>
          <summary>Show file formats, version, and diagnostic codes</summary>
          <div class="space-y-3 p-4">
            <ul class="list-disc space-y-2 ps-5">
              <li>
                A native session contains a same-stem BIN, CSV, and
                <code>manifest.json</code>.
              </li>
              <li>
                A current CSV has a seven-column header and RFC 3339 timestamps.
                A legacy v3 CSV has no header and uses compact rows such as
                <code>YYYYMMDDTHHMMSS,&lt;ones&gt;</code>. Reports also accepts
                standalone current or legacy BIN files when their filenames
                contain valid metadata.
              </li>
              <li>
                A canonical flat legacy concatenation CSV has a <code
                  >_concat_</code
                >
                stem and no manifest. It is read-only input; reports use its recorded
                timestamps.
              </li>
              <li>
                A derived bundle contains a same-stem CSV and manifest. Schema-1
                <code>legacy_csv_concatenation</code> bundles remain supported,
                while new Combine output uses schema 2 and
                <code>csv_concatenation</code>.
              </li>
            </ul>
            <p>RngKit application 0.1.0.</p>
            <p>
              Library revision <code>{RNGKIT_CORE_REVISION}</code>.
            </p>
            <p>Stable error codes shown in diagnostics:</p>
            <ul
              class="flex flex-wrap gap-x-3 gap-y-1"
              aria-label="Stable error codes"
            >
              {#each ERROR_CODES as code (code)}
                <li><code>{code}</code></li>
              {/each}
            </ul>
          </div>
        </details>
      </section>
    </article>
  </div>
</div>

<style>
  .help-page :is(h2, h3, a, summary):focus-visible {
    outline: 2px solid var(--color-text);
    outline-offset: 4px;
  }
  .help-page :is(h2, h3) {
    scroll-margin-top: 1rem;
  }
  .help-page code {
    overflow-wrap: anywhere;
  }
  .help-page details {
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
  }
  .help-page summary {
    cursor: pointer;
    padding: 0.75rem 1rem;
    font-weight: 500;
  }
  .help-page details > p {
    padding: 0 1rem 1rem;
  }
  .quick-start {
    border-inline-start: 3px solid var(--color-text-muted);
  }
</style>
