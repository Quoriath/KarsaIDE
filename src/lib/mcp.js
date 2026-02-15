// MCP Client for Frontend
import { invoke } from '@tauri-apps/api/core';

export class MCPClient {
  async execute(tool, params) {
    return await invoke('mcp_execute', {
      request: { tool, params }
    });
  }

  async getTools() {
    return await invoke('mcp_get_tools');
  }

  async getSystemPrompt() {
    return await invoke('mcp_get_system_prompt');
  }

  async executeChain(commands) {
    const results = [];
    for (const cmd of commands) {
      const result = await this.execute(cmd.tool, cmd.params);
      results.push(result);
      if (!result.success) break;
    }
    return results;
  }
}

export const mcpClient = new MCPClient();
