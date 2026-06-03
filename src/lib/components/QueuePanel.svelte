<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { Job, ConversionProgress } from '$lib/types';

  let {
    jobs,
    progress,
    isRunning,
    onRunAll,
    onClearQueue
  }: {
    jobs: Job[];
    progress: ConversionProgress;
    isRunning: boolean;
    onRunAll: () => void;
    onClearQueue: () => void;
  } = $props();

  function queuePercent(): number {
    if (progress.total === 0) return 0;
    if (progress.stage === 'completed') return 100;
    const base = (Math.max(progress.current, 1) - 1) / progress.total;
    const step = progress.stage === 'starting' ? 0.05 : progress.stage === 'chdman' ? 0.15 : 0.3;
    return Math.min(100, Math.round((base + step / progress.total) * 100));
  }
</script>

<section class="panel queue-panel">
  <div class="panel-header">
    <h2>Queue</h2>
    <div class="actions">
      <button on:click={onClearQueue} disabled={isRunning || jobs.length === 0}>Clear</button>
      <button class="primary" on:click={onRunAll} disabled={isRunning || jobs.length === 0}>
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
            Converting {progress.current} of {progress.total}
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
          {#if progress.stage === 'chdman' || progress.stage === 'psxpackager'}
            <span class="muted">· {progress.stage}</span>
          {/if}
        </p>
      {/if}
    </div>
  {/if}

  {#if jobs.length === 0}
    <div class="empty">
      No jobs yet. Add files from the input panel.
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
    border: 1px solid #333;
    border-radius: 14px;
    background: rgba(36, 36, 36, 0.86);
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
    color: #777;
    border: 1px dashed #333;
    border-radius: 10px;
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
    border: 1px solid #333;
    border-radius: 10px;
    background: rgba(26, 26, 26, 0.65);
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
    background: #1a1a1a;
    flex-shrink: 0;
  }

  .meta-tag {
    display: inline-block;
    margin-right: 6px;
    padding: 1px 6px;
    border: 1px solid #2c3a55;
    border-radius: 4px;
    background: rgba(91, 156, 246, 0.08);
    color: #cfe1ff;
    font-size: 11px;
  }

  .meta-tag.muted {
    border-color: #333;
    background: rgba(26, 26, 26, 0.5);
    color: #a0a0a0;
  }

  .job strong,
  .job span {
    display: block;
  }

  .job span {
    color: #a0a0a0;
    font-size: 12px;
  }

  .status {
    color: #a0a0a0;
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
    color: #8bbcff;
  }

  .progress {
    display: grid;
    gap: 6px;
    padding: 12px;
    border: 1px solid #2c3a55;
    border-radius: 10px;
    background: rgba(91, 156, 246, 0.08);
    margin-bottom: 12px;
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 13px;
    color: #cfe1ff;
  }

  .progress-percent {
    color: #8bbcff;
    font-weight: 700;
  }

  .progress-track {
    height: 10px;
    border-radius: 999px;
    background: rgba(26, 26, 26, 0.85);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #5b9cf6, #8bbcff);
    transition: width 0.2s ease;
  }

  .progress-file {
    margin: 0;
    font-size: 12px;
    color: #a0a0a0;
  }

  .muted {
    color: #a0a0a0;
    font-size: 13px;
  }

  button {
    border: 1px solid #333;
    border-radius: 8px;
    background: #2c2c2c;
    color: #f2f2f2;
    padding: 8px 12px;
    cursor: pointer;
  }

  button:hover {
    border-color: #5b9cf6;
    color: #8bbcff;
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  button.primary {
    border-color: #5b9cf6;
    background: #5b9cf6;
    color: #08111f;
  }
</style>
