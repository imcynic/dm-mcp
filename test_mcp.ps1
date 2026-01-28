$init = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
$list = '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
$input = "$init`n$list"
$input | & "C:\Users\Bryan\NKV26\dm-mcp\target\debug\dm-mcp.exe" 2>$null
