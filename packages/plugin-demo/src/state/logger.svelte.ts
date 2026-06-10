export interface LogEntry {
  time: string;
  message: string;
  type: 'info' | 'success' | 'error';
}

export class LoggerState {
  logs = $state<LogEntry[]>([]);

  log(message: string, type: LogEntry['type'] = 'info') {
    const now = new Date();
    const h = now.getHours().toString().padStart(2, '0');
    const m = now.getMinutes().toString().padStart(2, '0');
    const s = now.getSeconds().toString().padStart(2, '0');
    const ms = now.getMilliseconds().toString().padStart(3, '0');
    const time = `${h}:${m}:${s}.${ms}`;
    this.logs.push({ time, message, type });
  }

  clear() {
    this.logs = [];
  }

  async runTest(name: string, fn: () => Promise<any>) {
    this.log(`⏳ ${name} 执行中...`);
    try {
      const result = await fn();
      const resultStr =
        result !== undefined ? JSON.stringify(result, null, 2) : '(void)';
      this.log(
        `✓ ${name} 成功: ${resultStr.substring(0, 200)}${resultStr.length > 200 ? '...' : ''}`,
        'success',
      );
    } catch (err: any) {
      this.log(`✗ ${name} 失败: ${err.message || err}`, 'error');
    }
  }
}
