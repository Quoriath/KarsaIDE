# Natural Conversation Flow - Implementation Guide

## Overview

KarsaIDE now implements **natural conversation flow** inspired by Kilo Code, where MCP tool calls appear **inline** with AI responses, creating a more human-like interaction pattern.

## Architecture

### Flow Diagram

```
User: "Haloo, siapa kamu? cari struktur proyek"
    ↓
AI Stream Response:
    💭 Thinking: "Saya Karsa... perlu get_project_map"
    📝 Text: "Saya Karsa, AI assistant untuk coding"
    🔧 Tool: get_project_map()
    ↓
Tool Execution:
    ✅ Result: {...}
    ↓
AI Continue (Next Iteration):
    💭 Thinking: "OK dapat struktur..."
    📝 Text: "Struktur proyek Anda: ..."
```

### Key Components

#### 1. Backend: `commands.rs`

**Function: `extract_tool_calls_with_context()`**
```rust
fn extract_tool_calls_with_context(text: &str) 
    -> (String, Vec<(String, serde_json::Value)>, String)
```

Returns:
- `text_before`: Text BEFORE tool call
- `tool_calls`: Array of tool calls (only first is used)
- `text_after`: Text AFTER tool call (currently unused)

**Emission Sequence:**
1. `ai-reasoning-complete` - Complete thinking process
2. `ai-stream-chunk` - Text before tool
3. `ai-tool-call` - Tool execution request
4. `ai-tool-result` - Tool execution result
5. Loop continues - AI responds to result

#### 2. Frontend: `AgentView.svelte`

**Event Listeners:**
```javascript
ai-stream-reasoning      → Accumulate reasoning
ai-reasoning-complete    → Set complete reasoning
ai-stream-chunk          → Accumulate content
ai-tool-call             → Add tool to streamingTools[]
ai-tool-result           → Update tool with result
ai-stream-done           → Save message, reset state
```

**Message Structure:**
```javascript
{
  role: 'assistant',
  content: '...',           // Final text
  reasoning: '...',         // Thinking process
  tools: [                  // MCP tools executed
    {
      name: 'file_read',
      arguments: {...},
      executing: false,
      result: '...',
      error: null
    }
  ],
  timestamp: '10:30'
}
```

#### 3. UI Component: `MCPToolCall.svelte`

Displays tool execution inline:
```
🔧 file_read
    path: "./src/main.rs"
    ✅ Result (1234 chars) [click to expand]
```

## Key Features

### 1. No Iteration Limits
- AI has **full control** over execution flow
- No `MAX_ITERATIONS` constraint
- AI decides when task is complete

### 2. Sequential Tool Execution
- **ONE tool per response** (strictly enforced)
- Tool → Result → AI analyzes → Next tool (if needed)
- Prevents overwhelming context

### 3. Smart Truncation
- Tool results: 2000 chars max
- Shows truncation notice
- Prevents token overflow

### 4. Natural Conversation
- Text appears BEFORE tool call
- Tool UI appears inline
- AI continues AFTER tool result
- Feels like human explaining their process

## Configuration

### System Prompt (mcp.rs)

```rust
⚠️ ONE TOOL PER RESPONSE - STRICTLY ENFORCED ⚠️

WORKFLOW:
1. Think about the task
2. Explain what you'll do
3. Call ONE tool
4. Wait for result
5. Analyze result
6. Continue or provide final answer

EXAMPLE:
"Saya akan check file tersebut:"
[{"name": "file_read", "arguments": {"path": "./main.rs"}}]
[Wait for result]
"Dari file tersebut, saya lihat..."
```

### Token Limits

**Conversation Context:**
- System message: Always included
- User message: Always included
- Tool results: Truncated to 2000 chars
- No hard limit on iterations

**Streaming:**
- Reasoning: Unlimited (accumulated)
- Content: Unlimited (accumulated)
- Tool results: 2000 chars max

## Testing

### Test Case 1: Simple Query
```
User: "Haloo, siapa kamu?"
Expected:
- 💭 Thinking
- 📝 "Saya Karsa, AI assistant..."
- ✅ Done
```

### Test Case 2: Tool Required
```
User: "Cari struktur proyek"
Expected:
- 💭 Thinking: "Perlu get_project_map"
- 📝 "Saya akan check struktur proyek:"
- 🔧 get_project_map()
- ✅ Result
- 💭 Thinking: "OK dapat..."
- 📝 "Struktur proyek: ..."
- ✅ Done
```

### Test Case 3: Multiple Tools
```
User: "Baca main.rs lalu cari semua function"
Expected:
- 💭 Thinking
- 📝 "Saya akan baca file:"
- 🔧 file_read(main.rs)
- ✅ Result
- 💭 Thinking
- 📝 "Sekarang cari functions:"
- 🔧 search(pattern: "fn ")
- ✅ Result
- 💭 Thinking
- 📝 "Ditemukan functions: ..."
- ✅ Done
```

## Debugging

### Enable Logging
```bash
RUST_LOG=debug npm run tauri dev
```

### Check Logs
```
=== Agent iteration 1 ===
Stream complete - Content: 123 chars
Found 1 tool calls
Emitting text before tool: "Saya akan check..."
Executing tool: file_read
Tool result: 1234 chars
=== Agent iteration 2 ===
...
```

### Common Issues

**Issue: Tool appears at top**
- Check: `extract_tool_calls_with_context()` returns correct `text_before`
- Check: `ai-stream-chunk` emitted before `ai-tool-call`

**Issue: Multiple tools at once**
- Check: Only first tool is used: `tool_calls[0]`
- Check: System prompt enforces ONE tool rule

**Issue: Infinite loop**
- Check: AI provides final answer without tool call
- Check: Tool results are properly added to conversation

## Performance

### Metrics
- **Startup**: < 1 second
- **First token**: < 500ms
- **Tool execution**: 50-200ms
- **Memory**: 80-120 MB active

### Optimizations
- Async tool execution
- Truncated results
- Minimal conversation context
- Efficient regex parsing

## Future Improvements

### Phase 1: Better Text Splitting
- Parse `text_after` from tool calls
- Show partial response before tool
- Continue response after tool result

### Phase 2: Parallel Tools
- Allow multiple independent tools
- Execute in parallel
- Aggregate results

### Phase 3: Tool Streaming
- Stream tool results as they arrive
- Show progress for long operations
- Cancel individual tools

## References

- **Kilo Code**: `/tmp/kilocode` - Inspiration for streaming architecture
- **System Prompt**: `src-tauri/src/mcp.rs` - AI behavioral rules
- **Agent Loop**: `src-tauri/src/commands.rs` - Main execution flow
- **UI Components**: `src/lib/components/` - Frontend rendering

## Credits

Inspired by **Kilo Code** extension architecture:
- Stream chunk types separation
- Tool lifecycle management
- Natural conversation patterns
- Real-time thinking visualization

---

**Last Updated**: 2026-02-16  
**Version**: 0.1.0  
**Status**: ✅ Production Ready
