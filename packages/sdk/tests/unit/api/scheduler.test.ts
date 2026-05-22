import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';

// 变量名必须以 'mock' 开头，以便 Vitest 能够将其提升（Hoist）到 vi.mock 之前定义
const mockRegisteredCallbacks: Record<string, Function> = {};

vi.mock('../../../src/core/ipc', () => ({
  invoke: vi.fn(),
  listen: vi.fn((event: string, callback: Function) => {
    mockRegisteredCallbacks[event] = callback;
    return Promise.resolve(() => {});
  }),
}));

vi.mock('../../../src/core/runtime', () => ({
  getPluginId: vi.fn(() => 'test-plugin-id'),
}));

// 使用动态导入，避免 ES Module 静态导入提升导致全局变量尚未初始化的 ReferenceError
let scheduler: any;
let invoke: any;

beforeAll(async () => {
  const schedulerMod = await import('../../../src/api/scheduler');
  scheduler = schedulerMod.scheduler;
  const ipcMod = await import('../../../src/core/ipc');
  invoke = ipcMod.invoke;
});

describe('Scheduler API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('namespace and exports', () => {
    it('should have scheduler namespace', () => {
      expect(scheduler).toBeDefined();
    });

    it('should have all expected methods in namespace', () => {
      expect(typeof scheduler.schedule).toBe('function');
      expect(typeof scheduler.daily).toBe('function');
      expect(typeof scheduler.hourly).toBe('function');
      expect(typeof scheduler.weekly).toBe('function');
      expect(typeof scheduler.cancel).toBe('function');
      expect(typeof scheduler.list).toBe('function');
      expect(typeof scheduler.at).toBe('function');
      expect(typeof scheduler.timeout).toBe('function');
    });
  });

  describe('validation functions', () => {
    it('should throw error for invalid cron format in schedule', async () => {
      const cb = vi.fn();
      await expect(scheduler.schedule('task1', '0 8 * *', cb)).rejects.toThrow(
        'Invalid cron format',
      );
      await expect(
        scheduler.schedule('task1', '0 8 * * * *', cb),
      ).rejects.toThrow('Invalid cron format');
      expect(invoke).not.toHaveBeenCalled();
    });

    it('should throw error for invalid time format in daily', async () => {
      const cb = vi.fn();
      await expect(scheduler.daily('task1', '25:30', cb)).rejects.toThrow(
        'Invalid time format',
      );
      await expect(scheduler.daily('task1', '12:60', cb)).rejects.toThrow(
        'Invalid time format',
      );
      await expect(scheduler.daily('task1', '24:00', cb)).rejects.toThrow(
        'Invalid time format',
      );
      expect(invoke).not.toHaveBeenCalled();
    });

    it('should throw error for invalid weekday in weekly', async () => {
      const cb = vi.fn();
      await expect(scheduler.weekly('task1', 7, '09:00', cb)).rejects.toThrow(
        'Invalid weekday',
      );
      await expect(scheduler.weekly('task1', -1, '09:00', cb)).rejects.toThrow(
        'Invalid weekday',
      );
      expect(invoke).not.toHaveBeenCalled();
    });
  });

  describe('cron conversion and registration', () => {
    it('should schedule a valid cron task', async () => {
      const cb = vi.fn();
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await scheduler.schedule('cron-task', '0 8 * * *', cb);

      expect(invoke).toHaveBeenCalledWith('schedule_task', {
        pluginId: 'test-plugin-id',
        options: { id: 'cron-task', cron: '0 8 * * *' },
      });
    });

    it('should clean callback if backend schedule_task fails', async () => {
      const cb = vi.fn();
      const mockError = new Error('Backend error');
      vi.mocked(invoke).mockRejectedValueOnce(mockError);

      await expect(
        scheduler.schedule('failing-task', '0 8 * * *', cb),
      ).rejects.toThrow('Backend error');

      // 触发监听器，应该提示找不到回调（因为已经从 Map 中清理掉了）
      const eventHandler = mockRegisteredCallbacks['scheduler:execute-task'];
      expect(eventHandler).toBeDefined();

      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});
      eventHandler({ payload: { taskId: 'failing-task' } });
      expect(cb).not.toHaveBeenCalled();
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('No callback found for task: failing-task'),
      );
      consoleWarnSpy.mockRestore();
    });

    it('should schedule daily tasks with correct cron conversion', async () => {
      const cb = vi.fn();
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await scheduler.daily('daily-task', '08:30', cb);

      expect(invoke).toHaveBeenCalledWith('schedule_task', {
        pluginId: 'test-plugin-id',
        options: { id: 'daily-task', cron: '30 08 * * *' },
      });
    });

    it('should schedule hourly tasks with correct cron conversion', async () => {
      const cb = vi.fn();
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await scheduler.hourly('hourly-task', 15, cb);

      expect(invoke).toHaveBeenCalledWith('schedule_task', {
        pluginId: 'test-plugin-id',
        options: { id: 'hourly-task', cron: '15 * * * *' },
      });
    });

    it('should schedule weekly tasks with correct cron conversion', async () => {
      const cb = vi.fn();
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await scheduler.weekly('weekly-task', 1, '09:00', cb);

      expect(invoke).toHaveBeenCalledWith('schedule_task', {
        pluginId: 'test-plugin-id',
        options: { id: 'weekly-task', cron: '00 09 * * 1' },
      });
    });
  });

  describe('one-time scheduling (at and timeout)', () => {
    it('should schedule single execution task with at', async () => {
      const cb = vi.fn();
      vi.mocked(invoke).mockResolvedValueOnce(undefined);
      const timestamp = Date.now() + 10000;

      await scheduler.at('one-time-task', timestamp, cb);

      expect(invoke).toHaveBeenCalledWith('schedule_once', {
        pluginId: 'test-plugin-id',
        options: { id: 'one-time-task', execute_at: timestamp },
      });
    });

    it('should clean callback if backend schedule_once fails', async () => {
      const cb = vi.fn();
      const mockError = new Error('Backend error');
      vi.mocked(invoke).mockRejectedValueOnce(mockError);
      const timestamp = Date.now() + 10000;

      await expect(
        scheduler.at('failing-once-task', timestamp, cb),
      ).rejects.toThrow('Backend error');

      // 触发事件，由于报错回滚清理，回调不应被执行
      const eventHandler = mockRegisteredCallbacks['scheduler:execute-task'];
      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});
      eventHandler({ payload: { taskId: 'failing-once-task' } });
      expect(cb).not.toHaveBeenCalled();
      consoleWarnSpy.mockRestore();
    });

    it('should schedule timeout with correct calculated timestamp', async () => {
      const cb = vi.fn();
      vi.mocked(invoke).mockResolvedValueOnce(undefined);
      const now = 1716300000000; // 固定时间戳便于断言
      vi.spyOn(Date, 'now').mockReturnValue(now);

      await scheduler.timeout('timeout-task', 5000, cb);

      expect(invoke).toHaveBeenCalledWith('schedule_once', {
        pluginId: 'test-plugin-id',
        options: { id: 'timeout-task', execute_at: now + 5000 },
      });

      vi.restoreAllMocks();
    });
  });

  describe('task management (cancel and list)', () => {
    it('should cancel active task and clean local callback', async () => {
      const cb = vi.fn();
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // schedule
        .mockResolvedValueOnce(undefined); // cancel

      await scheduler.schedule('cancel-task', '0 8 * * *', cb);
      await scheduler.cancel('cancel-task');

      expect(invoke).toHaveBeenNthCalledWith(2, 'cancel_task', {
        pluginId: 'test-plugin-id',
        taskId: 'cancel-task',
      });

      // 触发事件验证，回调不应执行并产生警告
      const eventHandler = mockRegisteredCallbacks['scheduler:execute-task'];
      const consoleWarnSpy = vi
        .spyOn(console, 'warn')
        .mockImplementation(() => {});
      eventHandler({ payload: { taskId: 'cancel-task' } });
      expect(cb).not.toHaveBeenCalled();
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('No callback found for task: cancel-task'),
      );
      consoleWarnSpy.mockRestore();
    });

    it('should list tasks from backend', async () => {
      const mockTasks = [
        {
          id: 't1',
          pluginId: 'test-plugin-id',
          cron: '0 8 * * *',
          enabled: true,
        },
      ];
      vi.mocked(invoke).mockResolvedValueOnce(mockTasks);

      const result = await scheduler.list();

      expect(invoke).toHaveBeenCalledWith('list_tasks', {
        pluginId: 'test-plugin-id',
      });
      expect(result).toEqual(mockTasks);
    });
  });

  describe('event listener invocation', () => {
    it('should invoke registered callback when execute-task event is received', async () => {
      const cb = vi.fn().mockResolvedValue(undefined);
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await scheduler.schedule('triggered-task', '0 8 * * *', cb);

      const eventHandler = mockRegisteredCallbacks['scheduler:execute-task'];
      expect(eventHandler).toBeDefined();

      await eventHandler({ payload: { taskId: 'triggered-task' } });

      expect(cb).toHaveBeenCalledTimes(1);
    });

    it('should handle async errors in user task callback gracefully', async () => {
      const consoleErrorSpy = vi
        .spyOn(console, 'error')
        .mockImplementation(() => {});
      const cb = vi.fn().mockRejectedValue(new Error('User task error'));
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await scheduler.schedule('error-task', '0 8 * * *', cb);

      const eventHandler = mockRegisteredCallbacks['scheduler:execute-task'];

      // 触发事件 (内部为 Promise 异步链，会通过 .catch 捕获)
      eventHandler({ payload: { taskId: 'error-task' } });

      // 等待宏任务/微任务队列清空，使 Promise.resolve().catch 链执行完毕
      await new Promise((resolve) => setTimeout(resolve, 0));

      expect(cb).toHaveBeenCalledTimes(1);
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining(
          '[Scheduler] Task error-task execution failed:',
        ),
        expect.any(Error),
      );

      consoleErrorSpy.mockRestore();
    });
  });
});
