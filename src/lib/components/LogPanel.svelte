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
    if (level === 'all') return '#a0a0a0';
    if (level === 'info') return '#8bbcff';
    if (level === 'warn') return '#ffd580';
    return '#ff7b7b';
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
            on:click={() => (filter = level)}
          >
            {level}
          </button>
        {/each}
      </div>
      <button on:click={copyLog} disabled={isRunning}>
        {copied ? 'Copied!' : 'Copy'}
      </button>
      <button on:click={onTestBackend} disabled={isRunning}>
        Test Backend
      </button>
    </div>
  </div>

  <pre>{getFilteredLines().join('\n')}</pre>
</section>

<style>
  .panel {
    border: 1px solid #333;
    border-radius: 14px;
    background: rgba(36, 36, 36, 0.86);
    padding: 16px;
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
    border: 1px solid #333;
    border-radius: 8px;
    background: #1a1a1a;
  }

  .filter-btn {
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #a0a0a0;
    padding: 4px 10px;
    font-size: 12px;
    cursor: pointer;
  }

  .filter-btn:hover {
    background: rgba(91, 156, 246, 0.1);
  }

  .filter-btn.active {
    background: #2c2c2c;
  }

  h2 {
    margin: 0;
    font-size: 15px;
  }

  .log-panel {
    flex: 1;
  }

  pre {
    min-height: 150px;
    max-height: 400px;
    margin: 0;
    overflow: auto;
    color: #a0a0a0;
    line-height: 1.6;
    white-space: pre-wrap;
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
</style>
