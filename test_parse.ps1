$init = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
$parse = '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"dm_parse_environment","arguments":{"dme_path":"C:\\Users\\Bryan\\NKV26\\NewKV\\New NKV.dme"}}}'
$input = "$init`n$parse"
$input | & "C:\Users\Bryan\NKV26\dm-mcp\target\release\dm-mcp.exe" 2>$null
