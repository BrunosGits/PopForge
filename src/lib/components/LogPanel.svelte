<script lang="ts">
  type LogLevel = 'info' | 'warn' | 'error';

  let {
    logLines,
    outputFolder,
    backendFile,
    backendMessage,
    isRunning,
    onTestBackend
  }: {
    logLines: string[];
    outputFolder: string;
    backendFile: string;
    backendMessage: string;
    isRunning: boolean;
    onTestBackend: () => void;
  } = $props();

  let filter = $state<LogLevel | 'all'>('all');
  let copied = $state(false);

  const levels: (LogLevel | 'all')[] = ['all', 'info', 'warn', 'error'];

  function matchesFilter(line: string): boolean {
    if (filter === 'all') return true;
    return line.includes(`[${filter}]`);
  }

  function getFilteredLines(): string[] {
    const header = [
      `[info] Output folder: ${outputFolder}`,
      `[info] Last selected file: ${backendFile}`,
      backendMessage
    ].filter(Boolean);
    return [...header, ...logLines].filter(matchesFilter);
  }

  async function copyLog() {
    const text = getFilteredLines().join('\n');
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      const textarea = document.createElement('textarea');
      textarea.value = text;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    }
  }

  function levelColor(level: LogLevel | 'all'): string {
    if (level === 'all') return 'var(--text-secondary)';
    if (level === 'info') return 'var(--accent)';
    if (level === 'warn') return 'var(--warn-text)';
    return 'var(--danger)';
  }
</script>

<section class="panel log-panel">
  <div class="panel-header">
    <h2>Log</h2>
    <div class="actions">
      <div class="filters">
        {#each levels as level}
          <button
            class="filter-btn"
            class:active={filter === level}
            style:color={filter === level ? levelColor(level) : undefined}
            onclick={() => (filter = level)}
          >
            {level}
          </button>
        {/each}
      </div>
      <button class="btn-secondary" onclick={copyLog} disabled={isRunning}>
        {copied ? 'Copied!' : 'Copy'}
      </button>
      <button class="btn-secondary" onclick={onTestBackend} disabled={isRunning}>
        Test Backend
      </button>
    </div>
  </div>

  <pre>{getFilteredLines().join('\n')}</pre>
</section>

<style>
  .panel {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--bg);
    padding: 18px;
    box-shadow: 0 1px 2px rgba(16, 24, 40, 0.04);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 14px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filters {
    display: flex;
    padding: 3px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-tertiary);
  }

  .filter-btn {
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
  }

  .filter-btn:hover {
    background: var(--bg-hover);
  }

  .filter-btn.active {
    background: var(--bg);
    box-shadow: 0 1px 2px rgba(16, 24, 40, 0.06);
  }

  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
  }

  .log-panel {
    flex: 1;
  }

  pre {
    min-height: 150px;
    max-height: 400px;
    margin: 0;
    overflow: auto;
    color: var(--text-secondary);
    line-height: 1.6;
    white-space: pre-wrap;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
    font-size: 13px;
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
</style>
