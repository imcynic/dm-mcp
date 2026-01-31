# Reddit r/SS13 Post

**Title:** I built an MCP server that lets Claude/Cursor understand DreamMaker code

---

Hey coders,

I've been working on something that might interest anyone doing SS13 development: **dm-mcp** - an MCP (Model Context Protocol) server specifically for DreamMaker.

**What it does:**
- Connects Claude, Cursor, Windsurf, or any MCP-compatible AI to your DM codebase
- The AI can actually *understand* your code: types, procs, inheritance, the works
- Real-time type checking via SpacemanDMM's dreamchecker
- Go-to-definition, symbol search, map analysis
- Even run/stop DreamDaemon and send Topic() calls

**Why this matters:**
AI coding assistants are great, but they struggle with DM because it's such a niche language. This gives them proper tooling - they can navigate your object tree, understand type hierarchies, find where things are defined. It's like giving them SpacemanDMM superpowers.

**Example workflow:**
1. Open your SS13 codebase in Claude Code or Cursor
2. Ask "How does the atmospheric system work?"
3. The AI can actually explore `/datum/pipeline`, find related procs, check the type hierarchy
4. Get answers that reference your actual code, not hallucinated nonsense

**GitHub:** https://github.com/imcynic/dm-mcp

Built in Rust, uses SpacemanDMM under the hood for parsing. Also includes a full BYOND client protocol implementation if anyone wants to build testing tools.

Would love feedback from anyone who tries it. What features would make DM development easier for you?

---

*Flair: [Codebase] or [Tool] depending on subreddit rules*
