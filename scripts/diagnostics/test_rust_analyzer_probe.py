import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest import mock


MODULE_PATH = Path(__file__).with_name("rust-analyzer-probe.py")
SPEC = importlib.util.spec_from_file_location("rust_analyzer_probe", MODULE_PATH)
probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(probe)


class FakeProcess:
    def __init__(self, *, wait_error=None):
        self.returncode = None
        self.terminated = False
        self.killed = False
        self.wait_error = wait_error

    def poll(self):
        return self.returncode

    def terminate(self):
        self.terminated = True
        self.returncode = -15

    def kill(self):
        self.killed = True
        self.returncode = -9

    def wait(self, timeout=None):
        if self.wait_error is not None:
            error = self.wait_error
            self.wait_error = None
            raise error
        return self.returncode


class ClientReadTests(unittest.TestCase):
    def test_read_rejects_header_without_terminator_at_cap(self):
        client = object.__new__(probe.Client)
        client.buffer = b"X" * (probe.MAX_HEADER + 1)
        client.selector = mock.Mock()

        with self.assertRaises(ValueError):
            client.read(0)
        client.selector.select.assert_not_called()

    def test_read_rejects_terminated_header_over_cap(self):
        client = object.__new__(probe.Client)
        client.buffer = b"X" * (probe.MAX_HEADER + 1) + b"\r\n\r\n"
        client.selector = mock.Mock()

        with self.assertRaises(ValueError):
            client.read(0)
        client.selector.select.assert_not_called()


class ClientCloseTests(unittest.TestCase):
    def make_client(self, process):
        client = object.__new__(probe.Client)
        client.process = process
        client.selector = mock.Mock()
        return client

    def test_close_terminates_after_protocol_value_error(self):
        process = FakeProcess()
        client = self.make_client(process)
        client.request = mock.Mock(side_effect=ValueError("invalid LSP Content-Length"))

        client.close()

        self.assertTrue(process.terminated)
        self.assertFalse(process.killed)
        client.selector.close.assert_called_once_with()

    def test_close_kills_child_if_terminate_does_not_finish(self):
        process = FakeProcess(wait_error=subprocess.TimeoutExpired("wait", 5))
        client = self.make_client(process)
        client.request = mock.Mock(side_effect=ValueError("invalid JSON"))

        client.close()

        self.assertTrue(process.terminated)
        self.assertTrue(process.killed)
        client.selector.close.assert_called_once_with()


if __name__ == "__main__":
    unittest.main()
