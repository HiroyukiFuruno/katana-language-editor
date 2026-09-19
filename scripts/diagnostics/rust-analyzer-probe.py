#!/usr/bin/env python3
"""Read-only, diagnostic-only Rust definition lookup against the fixed reference."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import selectors
import subprocess
import time

from egui_definition_probe import follow_edges
from text_buffer_definition_probe import follow_text_buffer
from cursor_definition_probe import follow_cursor_range
from cursor_call_hierarchy_probe import follow_cursor_call_hierarchy

REVISION = "4f6a6287c650a38633c7baeb544a92e739c68567"
REPO = Path(__file__).resolve().parents[2]
QUERIES = [
    ("multiline", "crates/katana-ui/src/views/panels/editor/text_edit.rs", "egui::TextEdit::multiline", "egui::TextEdit::"),
    ("show", "crates/katana-ui/src/views/panels/editor/text_edit.rs", "text_edit.show", "text_edit."),
    ("shortcut", "crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs", "i.consume_shortcut", "i."),
]
MAX_MESSAGE = 8 * 1024 * 1024
MAX_HEADER = 64 * 1024


def git(root, *arguments):
    return subprocess.run(["rtk", "proxy", "git", "-C", str(root), *arguments],
                          check=True, capture_output=True).stdout


def verify_reference(root):
    if git(root, "rev-parse", "HEAD").decode().strip() != REVISION:
        raise ValueError("reference revision mismatch")
    if git(root, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("reference is not clean")


class Client:
    def __init__(self, root, stderr, target):
        environment = dict(os.environ, CARGO_NET_OFFLINE="true", CARGO_TARGET_DIR=str(target))
        self.process = subprocess.Popen(["rust-analyzer"], cwd=root, env=environment,
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.buffer = b""
        self.notifications = []
        self.counter = 0

    def send(self, message):
        payload = json.dumps(dict(jsonrpc="2.0", **message)).encode()
        self.process.stdin.write(f"Content-Length: {len(payload)}\r\n\r\n".encode() + payload)
        self.process.stdin.flush()

    def read(self, deadline):
        while True:
            header_end = self.buffer.find(b"\r\n\r\n")
            if header_end < 0 and len(self.buffer) > MAX_HEADER:
                raise ValueError("LSP header exceeds maximum size")
            if header_end > MAX_HEADER:
                raise ValueError("LSP header exceeds maximum size")
            if header_end >= 0:
                headers = self.buffer[:header_end].decode("ascii").split("\r\n")
                lengths = [int(line.split(":", 1)[1]) for line in headers
                           if line.lower().startswith("content-length:")]
                if len(lengths) != 1 or not 0 <= lengths[0] <= MAX_MESSAGE:
                    raise ValueError("invalid LSP Content-Length")
                end = header_end + 4 + lengths[0]
                if len(self.buffer) >= end:
                    message = json.loads(self.buffer[header_end + 4:end])
                    self.buffer = self.buffer[end:]
                    return message
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not self.selector.select(remaining):
                raise TimeoutError("LSP observation deadline exceeded")
            chunk = os.read(self.process.stdout.fileno(), 65536)
            if not chunk:
                raise EOFError("rust-analyzer closed its output")
            self.buffer += chunk

    def handle(self, message):
        if "method" in message and "id" in message:
            if message["method"] in ("window/workDoneProgress/create", "client/registerCapability"):
                self.send(dict(id=message["id"], result=None))
            else:
                self.send(dict(id=message["id"], error=dict(code=-32601, message="Unsupported probe request")))
        else:
            self.notifications.append(message)

    def request(self, method, params, timeout=180):
        self.counter += 1
        identifier = self.counter
        self.send(dict(id=identifier, method=method, params=params))
        deadline = time.monotonic() + timeout
        while True:
            message = self.read(deadline)
            if message.get("id") == identifier and "method" not in message:
                if "error" in message:
                    raise RuntimeError(f"{method}: {message['error']}")
                return message.get("result")
            self.handle(message)

    def ready(self):
        deadline = time.monotonic() + 180
        while True:
            message = self.read(deadline)
            self.handle(message)
            if message.get("method") == "experimental/serverStatus":
                status = message.get("params", {})
                if status.get("quiescent"):
                    if status.get("health") == "error":
                        raise RuntimeError(f"rust-analyzer is unhealthy: {status}")
                    return status

    def close(self):
        try:
            if self.process.poll() is None:
                self.request("shutdown", None, timeout=5)
                self.send(dict(method="exit", params=None))
                self.process.wait(timeout=5)
        except (OSError, ValueError, TimeoutError, RuntimeError, EOFError, subprocess.TimeoutExpired):
            if self.process.poll() is None:
                self.process.terminate()
                try:
                    self.process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait()
        finally:
            self.selector.close()


def run(root, output):
    verify_reference(root)
    if not output.is_relative_to(REPO / "target" / "acceptance") or output.exists():
        raise ValueError("output must be new and inside KLE target/acceptance")
    output.parent.mkdir(parents=True, exist_ok=True)
    config = {"cargo": {"buildScripts": {"enable": False}, "extraArgs": ["--locked", "--offline"],
                        "targetDir": str(output.parent / "cargo-target"), "allTargets": False},
              "procMacro": {"enable": False}, "checkOnSave": False, "numThreads": 2}
    report = dict(diagnostic_only=True, semantic_complete=False, execution_performed=False,
                  revision=REVISION, config=config, queries=[], errors=[],
                  limits=["build scripts disabled", "proc macros disabled", "host configuration only",
                          "definition lookup is not transitive closure or native input evidence"])
    report["engine"] = subprocess.run(["rust-analyzer", "--version"], cwd=root, check=True,
                                      capture_output=True, text=True).stdout.strip()
    with (output.parent / "stderr.log").open("xb") as stderr:
        client = Client(root, stderr, output.parent / "cargo-target")
        try:
            report["initialize"] = client.request("initialize", {
                "processId": os.getpid(), "rootUri": root.as_uri(),
                "capabilities": {"experimental": {"serverStatusNotification": True},
                                 "textDocument": {"documentSymbol": {"hierarchicalDocumentSymbolSupport": True}},
                                 "general": {"positionEncodings": ["utf-16"]}},
                "initializationOptions": config,
                "workspaceFolders": [{"uri": root.as_uri(), "name": "fixed-katana"}]})
            client.send(dict(method="initialized", params={}))
            report["server_status"] = client.ready()
            opened = set()
            for name, relative, needle, prefix in QUERIES:
                path = root / relative
                data = path.read_bytes()
                if data != git(root, "cat-file", "blob", f"{REVISION}:{relative}"):
                    raise ValueError(f"source identity mismatch: {relative}")
                text = data.decode("utf-8")
                if text.count(needle) != 1:
                    raise ValueError(f"ambiguous query anchor: {name}")
                before = text[:text.index(needle) + len(prefix)]
                position = dict(line=before.count("\n"), character=len(before.rsplit("\n", 1)[-1].encode("utf-16-le")) // 2)
                if relative not in opened:
                    client.send(dict(method="textDocument/didOpen", params={"textDocument": {
                        "uri": path.as_uri(), "languageId": "rust", "version": 1, "text": text}}))
                    opened.add(relative)
                result = client.request("textDocument/definition", dict(textDocument={"uri": path.as_uri()}, position=position))
                report["queries"].append(dict(name=name, path=relative, source_sha256=hashlib.sha256(data).hexdigest(),
                                              position=position, result=result))
                if not result:
                    report["errors"].append(f"unresolved definition: {name}")
                if name == "show":
                    report["external_edges"] = follow_edges(client, result)
                    report["text_buffer_edges"] = follow_text_buffer(client, result)
                    report["cursor_range_edges"] = follow_cursor_range(client, result)
                    report["cursor_call_hierarchy"] = follow_cursor_call_hierarchy(
                        client, report["cursor_range_edges"])
        except (OSError, ValueError, RuntimeError, TimeoutError, EOFError) as error:
            report["errors"].append(str(error))
        finally:
            client.close()
            report["notifications"] = client.notifications
            report["engine_exit_code"] = client.process.returncode
    verify_reference(root)
    with output.open("x") as destination:
        json.dump(report, destination, ensure_ascii=False, indent=2)
    return 1 if report["errors"] else 0


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    raise SystemExit(run(args.reference.resolve(strict=True), args.output.resolve()))
