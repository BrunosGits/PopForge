<script lang="ts">
  import type { Job, ConversionProgress, Mode } from '$lib/types';

  let {
    jobs,
    progress,
    mode,
    selectedJobIds,
    perJobProgress,
    isRunning,
    collapsed,
    onToggle,
    onRunAll,
    onClearQueue,
    onRemoveJob,
    onRetryJob,
    onToggleSelection,
    onMergeSelected,
    onUngroupJob,
    onCancel,
    onUpdateJobMetadata,
  }: {
    jobs: Job[];
    progress: ConversionProgress;
    mode: Mode;
    selectedJobIds: Set<number>;
    perJobProgress: Record<number, { filePercent: number | null; stage: string }>;
    isRunning: boolean;
    collapsed: boolean;
    onToggle: () => void;
    onRunAll: () => void;
    onClearQueue: () => void;
    onRemoveJob: (id: number) => void;
    onRetryJob: (id: number) => void;
    onToggleSelection: (id: number) => void;
    onMergeSelected: () => void;
    onUngroupJob: (id: number) => void;
    onCancel: () => void;
    onUpdateJobMetadata: (id: number, field: 'serial' | 'title', value: string) => void;
  } = $props();

  let hoveredGroupId = $state<number | null>(null);

  function queuePercent(): number {
    if (progress.total === 0) return 0;
    if (progress.stage === 'completed') return 100;
    const base = (Math.max(progress.current, 1) - 1) / progress.total;
    const step = progress.stage === 'starting' ? 0.05 : 0.3;
    return Math.min(100, Math.round((base + step / progress.total) * 100));
  }

  function groupInfo(groupId: number): { total: number; index: number } | null {
    const group = jobs.filter((j) => j.groupId === groupId).sort((a, b) => (a.discIndex ?? 0) - (b.discIndex ?? 0));
    if (group.length < 2) return null;
    return { total: group.length, index: 0 };
  }

  function discLabel(job: Job): string | null {
    if (job.groupId === null || job.discIndex === null) return null;
    const info = groupInfo(job.groupId);
    if (!info) return null;
    return `Disc ${job.discIndex + 1} of ${info.total}`;
  }

  function selectedCount(): number {
    return selectedJobIds.size;
  }

  function canMerge(): boolean {
    return selectedCount() >= 2;
  }
</script>

<section class="panel queue-panel">
  <button class="section-header" onclick={onToggle}>
    <h2>Queue</h2>
    <svg class="chevron" class:open={!collapsed} viewBox="0 0 16 16" fill="none">
      <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>
  </button>

  {#if !collapsed}
    <div class="header-actions">
      <button class="btn-secondary" onclick={onClearQueue} disabled={isRunning || jobs.length === 0}>Clear</button>
      <button class="btn-primary" onclick={onRunAll} disabled={isRunning || jobs.length === 0}>
        {isRunning ? 'Running...' : 'Run All'}
      </button>
    </div>

    {#if selectedCount() > 0}
      <div class="selection-bar">
        <span class="selection-count">{selectedCount()} selected</span>
        {#if canMerge()}
          <button class="btn-merge" onclick={onMergeSelected} disabled={isRunning}>Merge</button>
        {/if}
      </div>
    {/if}

    {#if progress.total > 0}
      <div class="progress" aria-live="polite">
        <div class="progress-meta">
          <span>
            {#if progress.stage === 'starting'}
              Starting…
            {:else if progress.stage === 'completed'}
              Finished
            {:else if progress.stage === 'cancelled'}
              Cancelled
            {:else}
              {mode === 'convert' ? 'Converting' : 'Extracting'} {progress.current} of {progress.total}
            {/if}
          </span>
          <span class="progress-percent">{Math.round(queuePercent())}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" style:width="{queuePercent()}%"></div>
        </div>
        {#if progress.fileName}
          <p class="progress-file">
            {progress.fileName}
            {#if progress.stage === 'psxpackager'}
              <span class="muted">· {progress.stage}</span>
            {/if}
          </p>
        {/if}
        {#if progress.stage !== 'completed' && progress.stage !== 'cancelled' && progress.stage !== 'idle'}
          <button class="cancel-btn" onclick={onCancel}>Cancel</button>
        {/if}
      </div>
    {/if}

    {#if jobs.length === 0}
      <div class="empty">
        <svg class="empty-icon" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="6" y="10" width="36" height="28" rx="3" stroke="var(--text-tertiary)" stroke-width="2" stroke-dasharray="4 2"/>
          <path d="M24 18v12M18 24h12" stroke="var(--accent)" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <p class="empty-title">No jobs yet</p>
        <p class="empty-hint">Drop files here or click the input panel to get started.</p>
      </div>
    {:else}
      <div class="jobs">
        {#each jobs as job}
          {@const label = discLabel(job)}
          <article
            class="job"
            class:grouped={job.groupId !== null}
            class:group-highlight={job.groupId !== null && hoveredGroupId === job.groupId}
            onmouseenter={() => { if (job.groupId !== null) hoveredGroupId = job.groupId; }}
            onmouseleave={() => { hoveredGroupId = null; }}
          >
            <button
              class="checkbox"
              class:checked={selectedJobIds.has(job.id)}
              onclick={() => onToggleSelection(job.id)}
              disabled={isRunning}
              aria-label="Select job"
            >
              {#if selectedJobIds.has(job.id)}
                <svg viewBox="0 0 16 16" fill="none" width="12" height="12">
                  <path d="M4 8l3 3 5-5" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              {/if}
            </button>

            <div class="job-info">
              <div>
                <strong>
                  {job.metadata?.title ?? job.fileName}
                </strong>
                {#if label}
                  <span class="disc-badge">{label}</span>
                {/if}
                <span class="job-path">{job.filePath}</span>
                <div class="inline-metadata">
                  <input
                    class="meta-input serial"
                    value={job.metadata?.serial ?? ''}
                    oninput={(e) => onUpdateJobMetadata(job.id, 'serial', e.currentTarget.value)}
                    placeholder="Game ID"
                  />
                  <input
                    class="meta-input title"
                    value={job.metadata?.title ?? ''}
                    oninput={(e) => onUpdateJobMetadata(job.id, 'title', e.currentTarget.value)}
                    placeholder="Title"
                  />
                </div>
                {#if job.message}
                  <span>{job.message}</span>
                {/if}
                {#if job.outputPath}
                  <span>{job.outputPath}</span>
                {/if}
                {#if job.commandPreview}
                  <span>{job.commandPreview}</span>
                {/if}
                <span class="job-mode">{job.mode}</span>
              </div>
              {#if perJobProgress[job.id]}
                {@const p = perJobProgress[job.id]}
                <div class="job-progress-track">
                  <div
                    class="job-progress-fill"
                    class:indet={p.filePercent === null && p.stage !== 'completed' && p.stage !== 'failed'}
                    style:width={p.filePercent !== null && p.filePercent > 0 ? `${Math.round(p.filePercent * 100)}%` : undefined}
                  ></div>
                </div>
              {/if}
            </div>

            <div class="job-actions">
              {#if job.groupId !== null}
                <button class="ungroup-btn" onclick={() => onUngroupJob(job.id)} disabled={isRunning} title="Ungroup discs">&times;</button>
              {/if}
              {#if job.status === 'error'}
                <button class="retry-btn" onclick={() => onRetryJob(job.id)} disabled={isRunning}>Retry</button>
              {/if}
              <button class="remove-btn" onclick={() => onRemoveJob(job.id)} disabled={isRunning}>&times;</button>
            </div>

            <span
              class:done={job.status === 'done'}
              class:error={job.status === 'error'}
              class:running={job.status === 'running'}
              class="status-badge"
            >
              {job.status}
            </span>
          </article>
        {/each}
      </div>
    {/if}
  {/if}
</section>

<style>
  .panel {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--bg);
    padding: 18px;
    box-shadow: 0 1px 2px rgba(16, 24, 40, 0.04);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    border: none;
    background: none;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
  }

  .chevron {
    width: 14px;
    color: var(--text-secondary);
    transition: transform 0.15s;
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .header-actions {
    display: flex;
    gap: 8px;
    margin-top: 14px;
  }

  .btn-secondary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 34px;
    padding: 0 14px;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.01em;
    border: 1px solid var(--btn-border);
    border-radius: 8px;
    background: var(--bg);
    color: var(--btn-text-color);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .btn-secondary:hover {
    background: var(--body-bg);
    border-color: var(--btn-hover-border);
  }

  .btn-secondary:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 36px;
    padding: 0 18px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.01em;
    border: 1px solid var(--btn-primary-border);
    border-radius: 8px;
    background: var(--accent);
    color: var(--btn-text);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .btn-primary:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .selection-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 10px;
    padding: 8px 12px;
    border-radius: 8px;
    background: var(--accent-bg);
    border: 1px solid var(--meta-tag-border);
  }

  .selection-count {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    flex: 1;
  }

  .btn-merge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 28px;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.01em;
    border: 1px solid var(--btn-primary-border);
    border-radius: 7px;
    background: var(--accent);
    color: var(--btn-text);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease;
  }

  .btn-merge:hover {
    background: var(--accent-hover);
  }

  .btn-merge:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .empty {
    display: grid;
    min-height: 180px;
    place-items: center;
    color: var(--text-tertiary);
    border: 1px dashed var(--border);
    border-radius: 10px;
    gap: 4px;
    margin-top: 12px;
  }

  .empty-icon {
    width: 48px;
    height: 48px;
    margin-bottom: 8px;
  }

  .empty-title {
    margin: 0;
    color: var(--text-secondary);
    font-size: 15px;
    font-weight: 600;
  }

  .empty-hint {
    margin: 0;
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .jobs {
    display: grid;
    gap: 8px;
    margin-top: 12px;
  }

  .job {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--bg-tertiary);
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .job.group-highlight {
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  .checkbox {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    border: 1.5px solid var(--btn-border);
    border-radius: 4px;
    background: var(--bg);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    padding: 0;
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .checkbox.checked {
    border-color: var(--accent);
    background: var(--accent);
  }

  .checkbox:disabled {
    cursor: not-allowed;
    opacity: 0.4;
  }

  .job-info {
    flex: 1;
    min-width: 0;
  }

  .job-info > div {
    overflow: hidden;
  }

  .job-info strong {
    display: block;
    font-size: 14px;
    font-weight: 700;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .job-path,
  .job-mode {
    display: block;
    color: var(--text-secondary);
    font-size: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  }

  .job span {
    display: block;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .job-progress-track {
    height: 4px;
    border-radius: 999px;
    background: var(--border-subtle);
    overflow: hidden;
    margin-top: 6px;
  }

  .job-progress-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--accent), var(--accent-hover));
    transition: width 0.3s ease;
  }

  .job-progress-fill.indet {
    width: 30%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.4; transform: translateX(0); }
    50% { opacity: 1; transform: translateX(200%); }
  }

  .disc-badge {
    display: inline-block !important;
    margin: 2px 0 4px;
    padding: 1px 7px;
    border: 1px solid var(--accent);
    border-radius: 4px;
    background: var(--accent-bg);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
  }

  .inline-metadata {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }

  .meta-input {
    font-size: 11px;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg);
    color: var(--text);
    outline: none;
    height: 24px;
    box-sizing: border-box;
  }

  .meta-input.serial {
    width: 120px;
    flex-shrink: 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  }

  .meta-input.title {
    flex: 1;
    min-width: 0;
  }

  .meta-input::placeholder {
    color: var(--text-tertiary);
  }

  .meta-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-bg);
  }

  .status-badge {
    display: inline-block;
    border-radius: 999px;
    padding: 3px 8px;
    background: var(--status-badge-bg);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .status-badge.done {
    background: var(--success-bg);
    color: var(--success-text);
  }

  .status-badge.error {
    background: var(--error-bg);
    color: var(--danger);
  }

  .status-badge.running {
    background: var(--accent-bg);
    color: var(--accent);
  }

  .job-actions {
    display: flex;
    gap: 5px;
    flex-shrink: 0;
  }

  .remove-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--danger-border);
    border-radius: 7px;
    background: var(--danger-bg);
    color: var(--danger);
    font-size: 13px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    transition: background 0.15s ease;
  }

  .remove-btn:hover {
    background: var(--danger-bg);
    filter: brightness(1.5);
  }

  .retry-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 28px;
    padding: 0 10px;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.01em;
    border: 1px solid var(--btn-border);
    border-radius: 7px;
    background: var(--bg);
    color: var(--btn-text-color);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .retry-btn:hover {
    background: var(--body-bg);
    border-color: var(--btn-hover-border);
  }

  .ungroup-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--btn-border);
    border-radius: 7px;
    background: var(--bg);
    color: var(--text-secondary);
    font-size: 14px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .ungroup-btn:hover {
    background: var(--body-bg);
    border-color: var(--btn-hover-border);
    color: var(--btn-text-color);
  }

  .progress {
    display: grid;
    gap: 6px;
    padding: 12px;
    border: 1px solid var(--meta-tag-border);
    border-radius: 10px;
    background: var(--accent-bg);
    margin-top: 12px;
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 13px;
    color: var(--accent);
  }

  .progress-percent {
    color: var(--accent-hover);
    font-weight: 700;
  }

  .progress-track {
    height: 8px;
    border-radius: 999px;
    background: var(--border);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-hover));
    transition: width 0.2s ease;
  }

  .progress-file {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .muted {
    color: var(--text-tertiary);
    font-size: 13px;
  }

  .cancel-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 30px;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.01em;
    border: 1px solid var(--danger);
    border-radius: 7px;
    background: var(--bg);
    color: var(--danger);
    cursor: pointer;
    white-space: nowrap;
    justify-self: start;
    transition: background 0.15s ease;
  }

  .cancel-btn:hover {
    background: var(--error-bg);
  }
</style>
