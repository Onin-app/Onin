/**
 * Scheduler API - 定时任务管理
 * 
 * 提供基于 cron 表达式的定时任务功能，支持持久化和应用重启后自动恢复。
 * 
 * @module scheduler
 */

import { invoke, listen } from '../core/ipc';

/**
 * 定时任务配置
 */
export interface ScheduleOptions {
  /** 任务唯一标识 */
  id: string;
  /** Cron 表达式 (格式: "分 时 日 月 周") */
  cron: string;
}

/**
 * 定时任务信息
 */
export interface ScheduleTask {
  id: string;
  pluginId: string;
  cron: string;
  enabled: boolean;
}

/**
 * 任务回调函数类型
 */
export type TaskCallback = () => void | Promise<void>;

// 存储任务回调
const taskCallbacks = new Map<string, TaskCallback>();

// 监听后端的任务执行事件
let isListenerSetup = false;

function setupTaskListener() {
  if (isListenerSetup) return;
  isListenerSetup = true;

  listen('scheduler:execute-task', (event: any) => {
    const { taskId } = event.payload;
    const callback = taskCallbacks.get(taskId);
    
    if (callback) {
      Promise.resolve(callback()).catch(err => {
        console.error(`[Scheduler] Task ${taskId} execution failed:`, err);
      });
    } else {
      console.warn(`[Scheduler] No callback found for task: ${taskId}`);
    }
  });
}

// 模块加载时立即初始化监听器
setupTaskListener();

/**
 * 验证时间格式 (HH:MM)
 */
function validateTimeFormat(time: string): void {
  const timeRegex = /^([0-1]?[0-9]|2[0-3]):([0-5][0-9])$/;
  if (!timeRegex.test(time)) {
    throw new Error(`Invalid time format: ${time}. Expected format: HH:MM (e.g., 08:30)`);
  }
}

/**
 * 验证 cron 表达式格式
 */
function validateCronFormat(cron: string): void {
  const parts = cron.split(/\s+/);
  if (parts.length !== 5) {
    throw new Error(
      `Invalid cron format: ${cron}. Expected format: 'minute hour day month weekday'`
    );
  }
}

/**
 * 注册定时任务
 * 
 * @param id - 任务唯一标识
 * @param cron - Cron 表达式，格式: "分 时 日 月 周"
 * @param callback - 任务执行回调
 * 
 * @example
 * ```typescript
 * // 每天早上 8 点执行
 * await scheduler.schedule('daily-quote', '0 8 * * *', async () => {
 *   console.log('Good morning!');
 * });
 * 
 * // 每小时执行
 * await scheduler.schedule('hourly-sync', '0 * * * *', async () => {
 *   await syncData();
 * });
 * 
 * // 每周一早上 9 点
 * await scheduler.schedule('weekly-report', '0 9 * * 1', async () => {
 *   await generateReport();
 * });
 * ```
 */
async function schedule(
  id: string,
  cron: string,
  callback: TaskCallback
): Promise<void> {
  // 验证 cron 格式
  validateCronFormat(cron);
  
  // 保存回调
  taskCallbacks.set(id, callback);
  
  try {
    await invoke('schedule_task', {
      options: { id, cron }
    });
  } catch (error) {
    // 如果注册失败，清理回调
    taskCallbacks.delete(id);
    throw error;
  }
}

/**
 * 每天定时执行（简化版）
 * 
 * @param id - 任务唯一标识
 * @param time - 时间，格式: "HH:MM" (24小时制)
 * @param callback - 任务执行回调
 * 
 * @example
 * ```typescript
 * // 每天早上 8:30
 * await scheduler.daily('morning-reminder', '08:30', async () => {
 *   await notification.show({
 *     title: '早安',
 *     body: '新的一天开始了！'
 *   });
 * });
 * ```
 */
async function daily(
  id: string,
  time: string,
  callback: TaskCallback
): Promise<void> {
  validateTimeFormat(time);
  const [hour, minute] = time.split(':');
  const cron = `${minute} ${hour} * * *`;
  return schedule(id, cron, callback);
}

/**
 * 每小时执行
 * 
 * @param id - 任务唯一标识
 * @param minute - 分钟 (0-59)
 * @param callback - 任务执行回调
 * 
 * @example
 * ```typescript
 * // 每小时的第 30 分钟执行
 * await scheduler.hourly('hourly-check', 30, async () => {
 *   await checkUpdates();
 * });
 * ```
 */
async function hourly(
  id: string,
  minute: number,
  callback: TaskCallback
): Promise<void> {
  const cron = `${minute} * * * *`;
  return schedule(id, cron, callback);
}

/**
 * 每周执行
 * 
 * @param id - 任务唯一标识
 * @param weekday - 星期几 (0=周日, 1=周一, ..., 6=周六)
 * @param time - 时间，格式: "HH:MM"
 * @param callback - 任务执行回调
 * 
 * @example
 * ```typescript
 * // 每周一早上 9:00
 * await scheduler.weekly('weekly-report', 1, '09:00', async () => {
 *   await generateWeeklyReport();
 * });
 * ```
 */
async function weekly(
  id: string,
  weekday: number,
  time: string,
  callback: TaskCallback
): Promise<void> {
  if (weekday < 0 || weekday > 6) {
    throw new Error(`Invalid weekday: ${weekday}. Expected 0-6 (0=Sunday, 6=Saturday)`);
  }
  validateTimeFormat(time);
  const [hour, minute] = time.split(':');
  const cron = `${minute} ${hour} * * ${weekday}`;
  return schedule(id, cron, callback);
}

/**
 * 取消定时任务
 * 
 * @param id - 任务标识
 * 
 * @example
 * ```typescript
 * await scheduler.cancel('daily-quote');
 * ```
 */
async function cancel(id: string): Promise<void> {
  // 先调用后端取消
  await invoke('cancel_task', { taskId: id });
  
  // 后端成功后再清理本地回调
  taskCallbacks.delete(id);
}

/**
 * 获取所有任务列表
 * 
 * @returns 任务列表
 * 
 * @example
 * ```typescript
 * const tasks = await scheduler.list();
 * console.log('已注册任务:', tasks);
 * ```
 */
async function list(): Promise<ScheduleTask[]> {
  return invoke('list_tasks', {});
}



/**
 * Scheduler API 命名空间
 */
export const scheduler = {
  schedule,
  daily,
  hourly,
  weekly,
  cancel,
  list,
};
