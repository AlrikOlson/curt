#!/usr/bin/env python3
"""Mock-host demo for the curt C ABI (host-ffi, SPEC §8).

Plays an embedding host via ctypes: registers an `ask` tool (a mock LLM
backend returning canned JSON), runs a curt program that calls
`host.ask`, shape-validates the response with `json` + `match`, and
asserts the captured output — plus deny-by-default for unregistered tools.

Run from the repo root after `cargo build --release`:
  python3 examples/embed/mock_host.py
"""

import ctypes
import json
import pathlib
import platform
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
DYLIB = ROOT / "target" / "release" / (
    "libcurt.dylib" if platform.system() == "Darwin" else "libcurt.so"
)

# returns c_void_p (not c_char_p) so ctypes does not try to manage the
# string — the library copies it synchronously per the SPEC §8 contract
TOOL_FN = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_char_p, ctypes.c_void_p)


class CurtTool(ctypes.Structure):
    _fields_ = [
        ("name", ctypes.c_char_p),
        ("call", TOOL_FN),
        ("userdata", ctypes.c_void_p),
    ]


# The mock LLM backend: always answers 42, confidently.
_KEEPALIVE: list[bytes] = []  # ctypes strings must outlive the callback return


@TOOL_FN
def mock_ask(arg, _userdata):
    print(f"  [mock host] ask({arg.decode()!r})")
    reply = ctypes.create_string_buffer(
        json.dumps({"answer": "42", "confidence": 0.9}).encode()
    )
    _KEEPALIVE.append(reply)
    return ctypes.addressof(reply)


# The curt program: call the tool, shape-validate via json + match, and
# rescue the unregistered-tool case — the ask "helper" at v1 honesty.
PROGRAM = """\
r = host.ask "meaning of life?"
v = json r
ans = v["answer"] ? "missing"
conf = v["confidence"] ? 0
print (match conf { float c -> "answer {ans} (conf {c})", _ -> "malformed" })
print (host.unregistered "x" ? "denied")
"""

EXPECTED = "answer 42 (conf 0.9)\ndenied\n"


def main() -> int:
    lib = ctypes.CDLL(str(DYLIB))
    lib.curt_eval_tools.restype = ctypes.c_int
    lib.curt_eval_tools.argtypes = [
        ctypes.c_char_p, ctypes.c_uint8, ctypes.c_uint8,
        ctypes.POINTER(CurtTool), ctypes.c_size_t,
        ctypes.POINTER(ctypes.c_char_p),
    ]
    lib.curt_free.argtypes = [ctypes.c_char_p]

    tools = (CurtTool * 1)(CurtTool(b"ask", mock_ask, None))
    out = ctypes.c_char_p()
    rc = lib.curt_eval_tools(PROGRAM.encode(), 0, 0, tools, 1, ctypes.byref(out))
    result = json.loads(out.value.decode())
    # NOTE: out is owned by the library; ctypes copied via .value — free it
    lib.curt_free(out)

    print(f"rc={rc} result={result}")
    if rc != 0 or not result["ok"]:
        print("FAILED: eval did not succeed", file=sys.stderr)
        return 1
    if result["stdout"] != EXPECTED:
        print(f"FAILED: expected {EXPECTED!r}", file=sys.stderr)
        return 1
    print("mock-host demo OK: tool registry + ask shape-validation + deny-by-default")
    return 0


if __name__ == "__main__":
    sys.exit(main())
