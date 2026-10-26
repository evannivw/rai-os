# UART protocol v1

A small bounded line format, without JSON or dynamic guest allocation. This is
request transport, not IPC between isolated processes. UART0 has one host
reader/writer. There is no authentication, negotiation, concurrency, or ID deduplication.

## Format

Examples use literal escape notation for actual TAB, LF, and CR bytes:

```text
@rai1\t1\thelp\n
@rai1\t2\tinfo\n
@rai1\t3\techo\thello from host\n
@rai1\t3\tok\thello from host\r\n
@rai1\t4\terr\tunknown_operation\r\n
```

Grammar: @rai1 TAB id TAB operation [TAB argument] line-ending.
help/info require exactly three fields; echo requires four, with a possibly
empty argument. Names are case-sensitive. ID: 1-10 ASCII digits with value
1..4294967295; leading zeros are allowed and preserved. IDs are not remembered.

- Maximum request: 256 bytes excluding terminator; no truncation.
- Input: printable ASCII; TAB separates machine fields only.
- Echo arguments cannot contain TAB, CR, LF, NUL, escapes, or Unicode.
- Input terminators: LF, CR, or CRLF (one terminator). Replies always use CRLF.
- Lines beginning with @ are machine requests and are not echoed.
- Other lines are human commands: help, info, echo, echo <text>. Human input is
  echoed; Backspace/Delete edit it. There is no guest prompt, keeping machine
  replies at the beginning of a line. Empty human lines do nothing; spaces are
  not normalized.
- Wait for Console ready before sending. One request followed by its response
  is sufficient for this milestone.

## Errors

bad_version, bad_id, bad_frame, bad_args, unknown_operation, invalid_byte,
line_too_long, uart_error. Parsing errors retain a valid ID; invalid IDs and
transport errors use 0. Validation order: version, ID, operation/arguments.
Only a fully validated operation can execute.

Invalid bytes, UART errors, and overflow poison the entire line until its next
terminator. The rejected tail never becomes another command. The guest uses a
fixed buffer and has no line timeout. An incomplete request waits for a terminator;
the host must impose its own timeout and terminate the line or restart QEMU.

## Future Meta bridge

The host console currently starts scripts/qemu.sh and uses its stdin/stdout.
Meta calls are not implemented. The key will live in the host environment.
The bridge must validate an allowed operation, field contents, and lengths before
serializing. Never concatenate arbitrary model output onto UART: a newline
starts another request. The guest repeats validation before execution.
Only informational operations currently exist; there is no file access or
arbitrary code execution.

This interface is not a security boundary against a malicious host. Before
adding side effects, define permissions, policy, exclusive protocol mode, and
subsequently real service isolation.
