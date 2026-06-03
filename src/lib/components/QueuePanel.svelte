<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { Job, ConversionProgress, Mode } from '$lib/types';

  let {
    jobs,
    progress,
    mode,
    isRunning,
    onRunAll,
    onClearQueue,
    onRemoveJob,
    onRetryJob
  }: {
    jobs: Job[];
    progress: ConversionProgress;
    mode: Mode;
    isRunning: boolean;
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
  <div class="panel-header">
    <h2>Queue</h2>
    <div class="actions">
      <button onclick={onClearQueue} disabled={isRunning || jobs.length === 0}>Clear</button>
      <button class="primary" onclick={onRunAll} disabled={isRunning || jobs.length === 0}>
        {isRunning ? 'Running...' : 'Run All'}
      </button>
    </div>
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
        <rect x="6" y="10" width="36" height="28" rx="3" stroke="#555" stroke-width="2" stroke-dasharray="4 2"/>
        <path d="M24 18v12M18 24h12" stroke="#5b9cf6" stroke-width="2" stroke-linecap="round"/>
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
              <span>{job.filePath}</span>
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
              <span>{job.mode}</span>
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
            class="status"
          >
            {job.status}
          </span>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .panel {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--bg);
    padding: 16px;
  }

  h2 {
    margin: 0 0 14px;
    font-size: 15px;
  }

  .panel-header,
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .queue-panel {
    min-height: 330px;
  }

  .empty {
    display: grid;
    min-height: 230px;
    place-items: center;
    color: var(--text-tertiary);
    border: 1px dashed var(--border);
    border-radius: 10px;
    gap: 4px;
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
  }

  .job {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--bg-secondary);
  }

  .job-info {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    flex: 1;
    min-width: 0;
  }

  .job-cover {
    width: 72px;
    height: 72px;
    object-fit: cover;
    border-radius: 6px;
    background: var(--bg-tertiary);
    flex-shrink: 0;
  }

  .meta-tag {
    display: inline-block;
    margin-right: 6px;
    padding: 1px 6px;
    border: 1px solid var(--meta-tag-border);
    border-radius: 4px;
    background: var(--meta-tag-bg);
    color: var(--accent);
    font-size: 11px;
  }

  .meta-tag.muted {
    border-color: var(--border);
    background: var(--bg-secondary);
    color: var(--text-secondary);
  }

  .job strong,
  .job span {
    display: block;
  }

  .job span {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .status {
    color: var(--text-secondary);
    text-transform: uppercase;
    font-size: 12px;
  }

  .status.done {
    color: #6ee785;
  }

  .status.error {
    color: #ff7b7b;
  }

  .status.running {
    color: var(--accent-hover);
  }

  .job-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .remove-btn {
    border: 1px solid #5a3030;
    border-radius: 6px;
    background: rgba(255, 80, 80, 0.1);
    color: #ff7b7b;
    padding: 2px 8px;
    font-size: 14px;
    cursor: pointer;
  }

  .remove-btn:hover {
    background: rgba(255, 80, 80, 0.25);
    border-color: #ff7b7b;
  }

  .retry-btn {
    border: 1px solid #3a5a30;
    border-radius: 6px;
    background: rgba(80, 200, 100, 0.1);
    color: #6ee785;
    padding: 2px 8px;
    font-size: 12px;
    cursor: pointer;
  }

  .retry-btn:hover {
    background: rgba(80, 200, 100, 0.25);
    border-color: #6ee785;
  }

  .progress {
    display: grid;
    gap: 6px;
    padding: 12px;
    border: 1px solid var(--meta-tag-border);
    border-radius: 10px;
    background: var(--accent-bg);
    margin-bottom: 12px;
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
    height: 10px;
    border-radius: 999px;
    background: var(--bg-secondary);
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
    color: var(--text-secondary);
    font-size: 13px;
  }

  button {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-hover);
    color: var(--text);
    padding: 8px 12px;
    cursor: pointer;
  }

  button:hover {
    border-color: var(--accent);
    color: var(--accent-hover);
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  button.primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--btn-text);
  }
</style>
