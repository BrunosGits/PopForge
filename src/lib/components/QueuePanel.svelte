<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { Job, ConversionProgress, Mode } from '$lib/types';

  let {
    jobs,
    progress,
    mode,
    isRunning,
    collapsed,
    onToggle,
    onRunAll,
    onClearQueue,
    onRemoveJob,
    onRetryJob
  }: {
    jobs: Job[];
    progress: ConversionProgress;
    mode: Mode;
    isRunning: boolean;
    collapsed: boolean;
    onToggle: () => void;
    onRunAll: () => void;
    onClearQueue: () => void;
    onRemoveJob: (id: number) => void;
    onRetryJob: (id: number) => void;
  } = $props();

  function queuePercent(): number {
    if (progress.total === 0) return 0;
    if (progress.stage === 'completed') return 100;
    const base = (Math.max(progress.current, 1) - 1) / progress.total;
    const step = progress.stage === 'starting' ? 0.05 : 0.3;
    return Math.min(100, Math.round((base + step / progress.total) * 100));
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

    {#if progress.total > 0}
      <div class="progress" aria-live="polite">
        <div class="progress-meta">
          <span>
            {#if progress.stage === 'starting'}
              Starting…
            {:else if progress.stage === 'completed'}
              Finished
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
      </div>
    {/if}

    {#if jobs.length === 0}
      <div class="empty">
        <svg class="empty-icon" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="6" y="10" width="36" height="28" rx="3" stroke="#98A2B3" stroke-width="2" stroke-dasharray="4 2"/>
          <path d="M24 18v12M18 24h12" stroke="#2F7DF6" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <p class="empty-title">No jobs yet</p>
        <p class="empty-hint">Drop files here or click the input panel to get started.</p>
      </div>
    {:else}
      <div class="jobs">
        {#each jobs as job}
          <article class="job">
            <div class="job-info">
              {#if job.metadata?.coverPath}
                <img
                  class="job-cover"
                  src={convertFileSrc(job.metadata.coverPath)}
                  alt="{job.metadata.title} cover"
                />
              {/if}
              <div>
                <strong>
                  {job.metadata?.title ?? job.fileName}
                </strong>
                <span class="job-path">{job.filePath}</span>
                {#if job.metadata?.serial}
                  <span>
                    <span class="meta-tag">{job.metadata.serial}</span>
                    <span class="meta-tag">{job.metadata.region}</span>
                    {#if job.metadata.cached}
                      <span class="meta-tag muted">cached</span>
                    {:else if job.metadata.source && job.metadata.source !== 'stub' && job.metadata.source !== 'no-serial'}
                      <span class="meta-tag muted">{job.metadata.source}</span>
                    {/if}
                  </span>
                {/if}
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
            </div>

            <div class="job-actions">
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
    color: #1E2329;
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
    border: 1px solid #D0D5DD;
    border-radius: 8px;
    background: #FFFFFF;
    color: #344054;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .btn-secondary:hover {
    background: #F5F6F8;
    border-color: #C1C7CF;
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
    border: 1px solid #2476EE;
    border-radius: 8px;
    background: #2F7DF6;
    color: #FFFFFF;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease;
  }

  .btn-primary:hover {
    background: #1F6FE5;
  }

  .btn-primary:disabled {
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
    color: #667085;
    font-size: 15px;
    font-weight: 600;
  }

  .empty-hint {
    margin: 0;
    color: #98A2B3;
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
    gap: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: #F8FAFC;
  }

  .job-info {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    flex: 1;
    min-width: 0;
  }

  .job-cover {
    width: 64px;
    height: 64px;
    object-fit: cover;
    border-radius: 6px;
    background: var(--bg-tertiary);
    flex-shrink: 0;
  }

  .job-info strong {
    display: block;
    font-size: 14px;
    font-weight: 700;
    color: #1E2329;
  }

  .job-path,
  .job-mode {
    display: block;
    color: #667085;
    font-size: 12px;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  }

  .job span {
    display: block;
    color: #667085;
    font-size: 12px;
  }

  .meta-tag {
    display: inline-block;
    margin-right: 6px;
    padding: 1px 6px;
    border: 1px solid var(--meta-tag-border);
    border-radius: 4px;
    background: var(--meta-tag-bg);
    color: #2F7DF6;
    font-size: 11px;
  }

  .meta-tag.muted {
    border-color: var(--border);
    background: transparent;
    color: #98A2B3;
  }

  .status-badge {
    display: inline-block;
    border-radius: 999px;
    padding: 3px 8px;
    background: #F2F4F7;
    color: #667085;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .status-badge.done {
    background: #ECFDF3;
    color: #25A55F;
  }

  .status-badge.error {
    background: #FEF2F2;
    color: #E5484D;
  }

  .status-badge.running {
    background: #EAF2FF;
    color: #2F7DF6;
  }

  .job-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .remove-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid rgba(229, 72, 77, 0.30);
    border-radius: 7px;
    background: rgba(229, 72, 77, 0.07);
    color: #E5484D;
    font-size: 13px;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    transition: background 0.15s ease;
  }

  .remove-btn:hover {
    background: rgba(229, 72, 77, 0.15);
  }

  .retry-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 30px;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.01em;
    border: 1px solid #D0D5DD;
    border-radius: 7px;
    background: #FFFFFF;
    color: #344054;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .retry-btn:hover {
    background: #F5F6F8;
    border-color: #C1C7CF;
  }

  .progress {
    display: grid;
    gap: 6px;
    padding: 12px;
    border: 1px solid #BFDBFE;
    border-radius: 10px;
    background: #EAF2FF;
    margin-top: 12px;
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 13px;
    color: #2F7DF6;
  }

  .progress-percent {
    color: #1F6FE5;
    font-weight: 700;
  }

  .progress-track {
    height: 8px;
    border-radius: 999px;
    background: #D7DCE3;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #2F7DF6, #1F6FE5);
    transition: width 0.2s ease;
  }

  .progress-file {
    margin: 0;
    font-size: 12px;
    color: #667085;
  }

  .muted {
    color: #98A2B3;
    font-size: 13px;
  }
</style>
