import { invoke, listen } from '../core/ipc';
import { dispatch } from '../core/dispatch';

export type CommandHandler = (command: string, args: any) => any | Promise<any>;

let isHandlerRegistered = false;

/**
 * 注册指令处理器。
 *
 * 当宿主应用执行插件指令时，会调用注册的处理器函数。
 * 这个函数在每个插件实例中只应调用一次。
 *
 * @param handler 指令处理器函数
 * @returns {Promise<void>} 注册完成时解析的 Promise
 */
export async function registerCommandHandler(handler: CommandHandler): Promise<void> {
  if (isHandlerRegistered) {
    console.warn("CommandHandler has already been registered. Ignoring subsequent calls.");
    return;
  }

  await dispatch({
    webview: () => {
      const eventCallback = async (event: any) => {
        const { command, args, requestId } = event.payload as {
          command: string;
          args: any;
          requestId: string;
        };

        try {
          const result = await handler(command, args);
          await invoke("plugin_command_result", { requestId, success: true, result });
        } catch (error) {
          await invoke("plugin_command_result", {
            requestId,
            success: false,
            error: error instanceof Error ? error.message : String(error),
          });
        }
      };
      return listen("plugin_command_execute", eventCallback);
    },
    headless: () => listen("plugin_command_execute", handler as any),
  });

  isHandlerRegistered = true;
}