<script lang="ts">
  import { getContext } from 'svelte';
  import { ai } from 'onin-sdk';
  import type { LoggerState } from '../state/logger.svelte';

  const logger = getContext<LoggerState>('logger');
</script>

<div class="api-group">
  <h3>基础检测</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() => logger.runTest('ai.isAvailable', () => ai.isAvailable())}
    >
      <span class="api-name">isAvailable</span>
      <span class="api-desc">检查 AI 是否可用</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('ai.getCapabilities', () => ai.getCapabilities())}
    >
      <span class="api-name">getCapabilities</span>
      <span class="api-desc">获取 AI 能力</span>
    </button>
    <button
      class="test-btn"
      onclick={() => logger.runTest('ai.listModels', () => ai.listModels())}
    >
      <span class="api-name">listModels</span>
      <span class="api-desc">列出可用模型</span>
    </button>
  </div>
</div>

<div class="api-group">
  <h3>简单问答</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('ai.ask (简单)', () =>
          ai.ask('用一句话介绍 TypeScript'),
        )}
    >
      <span class="api-name">ask</span>
      <span class="api-desc">简单问答</span>
    </button>
    <button
      class="test-btn"
      onclick={() =>
        logger.runTest('ai.ask (带选项)', () =>
          ai.ask('写一首关于代码的俳句', {
            temperature: 0.9,
            max_tokens: 100,
          }),
        )}
    >
      <span class="api-name">ask + options</span>
      <span class="api-desc">带参数问答</span>
    </button>
  </div>
</div>

<div class="api-group">
  <h3>流式响应</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={async () => {
        logger.log('⏳ 开始流式响应测试...');
        let fullText = '';
        try {
          await ai.stream('用三句话介绍 Rust 编程语言', (chunk) => {
            fullText += chunk;
            logger.log(`📨 收到 chunk: ${chunk}`, 'info');
          });
          logger.log(`✓ 流式响应完成，总长度: ${fullText.length}`, 'success');
        } catch (err: any) {
          logger.log(`✗ 流式响应失败: ${err.message}`, 'error');
        }
      }}
    >
      <span class="api-name">stream</span>
      <span class="api-desc">流式问答</span>
    </button>
  </div>
</div>

<div class="api-group">
  <h3>多轮对话</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={async () => {
        logger.log('⏳ 创建对话管理器...');
        try {
          const conv = ai.createConversation(
            '你是一个友好的编程助手，用简洁的语言回答问题。',
          );
          logger.log('✓ 对话管理器已创建', 'success');

          logger.log('⏳ 第一轮对话...');
          const resp1 = await conv.ask('什么是闭包？');
          logger.log(`✓ AI: ${resp1.substring(0, 100)}...`, 'success');

          logger.log('⏳ 第二轮对话...');
          const resp2 = await conv.ask('能举个例子吗？');
          logger.log(`✓ AI: ${resp2.substring(0, 100)}...`, 'success');

          const history = conv.getHistory();
          logger.log(`✓ 对话历史共 ${history.length} 条消息`, 'success');
        } catch (err: any) {
          logger.log(`✗ 多轮对话失败: ${err.message}`, 'error');
        }
      }}
    >
      <span class="api-name">createConversation</span>
      <span class="api-desc">多轮对话</span>
    </button>
    <button
      class="test-btn"
      onclick={async () => {
        logger.log('⏳ 测试对话流式响应...');
        try {
          const conv = ai.createConversation('你是一个诗人。');
          let fullText = '';
          await conv.stream('写一首关于月亮的诗', (chunk) => {
            fullText += chunk;
          });
          logger.log(
            `✓ 流式对话完成: ${fullText.substring(0, 100)}...`,
            'success',
          );
        } catch (err: any) {
          logger.log(`✗ 流式对话失败: ${err.message}`, 'error');
        }
      }}
    >
      <span class="api-name">conversation.stream</span>
      <span class="api-desc">对话流式</span>
    </button>
  </div>
</div>

<div class="api-group">
  <h3>消息构建</h3>
  <div class="test-grid">
    <button
      class="test-btn"
      onclick={async () => {
        logger.log('⏳ 测试文本消息构建...');
        try {
          const msg = {
            role: 'user' as const,
            content: [{ type: 'text' as const, text: '这是一条测试消息' }],
          };
          logger.log(`✓ 文本消息: ${JSON.stringify(msg)}`, 'success');
        } catch (err: any) {
          logger.log(`✗ 构建失败: ${err.message}`, 'error');
        }
      }}
    >
      <span class="api-name">createTextMessage</span>
      <span class="api-desc">创建文本消息</span>
    </button>
    <button
      class="test-btn"
      onclick={async () => {
        logger.log('⏳ 测试图片消息构建...');
        try {
          const msg = {
            role: 'user' as const,
            content: [
              { type: 'text' as const, text: '描述这张图片' },
              {
                type: 'image_url' as const,
                image_url: { url: 'https://picsum.photos/200' },
              },
              {
                type: 'image_url' as const,
                image_url: {
                  url: 'https://picsum.photos/300',
                  detail: 'high' as const,
                },
              },
            ],
          };
          logger.log(
            `✓ 图片消息包含 ${msg.content.length} 个内容项`,
            'success',
          );
        } catch (err: any) {
          logger.log(`✗ 构建失败: ${err.message}`, 'error');
        }
      }}
    >
      <span class="api-name">createImageMessage</span>
      <span class="api-desc">创建图片消息</span>
    </button>
    <button
      class="test-btn"
      onclick={async () => {
        logger.log('⏳ 测试图片识别 (需要支持 vision 的模型)...');
        try {
          const msg = {
            role: 'user' as const,
            content: [
              { type: 'text' as const, text: '这张图片里有什么？' },
              {
                type: 'image_url' as const,
                image_url: { url: 'https://picsum.photos/400/300' },
              },
            ],
          };
          const response = await ai.ask([msg]);
          logger.log(
            `✓ AI 识别结果: ${response.substring(0, 150)}...`,
            'success',
          );
        } catch (err: any) {
          logger.log(`✗ 图片识别失败: ${err.message}`, 'error');
        }
      }}
    >
      <span class="api-name">Vision API</span>
      <span class="api-desc">图片识别</span>
    </button>
  </div>
</div>
