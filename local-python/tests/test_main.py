import unittest
from unittest.mock import patch, MagicMock
import main
import os


class TestMain(unittest.IsolatedAsyncioTestCase):
    def test_collect_system_info(self):
        with (
            patch("main.psutil") as mock_psutil,
            patch("main.subprocess.getoutput") as mock_getoutput,
        ):
            mock_psutil.cpu_count.return_value = 4
            mock_psutil.virtual_memory.return_value.total = 8 * 1024 * 1024 * 1024
            mock_psutil.virtual_memory.return_value.used = 4 * 1024 * 1024 * 1024
            mock_psutil.swap_memory.return_value.total = 2 * 1024 * 1024 * 1024
            mock_psutil.swap_memory.return_value.used = 1 * 1024 * 1024 * 1024

            mock_getoutput.return_value = "test-host"

            mock_psutil.net_if_stats.return_value = {"eth0": MagicMock()}
            mock_psutil.AF_LINK = 1
            mock_psutil.net_if_addrs.return_value = {
                "eth0": [MagicMock(family=1, address="00:11:22:33:44:55")]
            }
            mock_psutil.net_io_counters.return_value = {
                "eth0": MagicMock(bytes_recv=1000, bytes_sent=500)
            }

            report = main.collect_system_info("API STATUS")

            self.assertIn("System Information Report", report)
            self.assertIn("API STATUS", report)
            self.assertIn("Number of Cores:  4", report)
            self.assertIn("Total Memory:     8192 MB", report)
            self.assertIn("eth0", report)
            self.assertIn("00:11:22:33:44:55", report)

    def test_collect_disk_usage(self):
        with patch("main.psutil") as mock_psutil:
            mock_part = MagicMock()
            mock_part.mountpoint = "/"
            mock_part.fstype = "ext4"
            mock_psutil.disk_partitions.return_value = [mock_part]

            mock_usage = MagicMock()
            mock_usage.used = 500 * 1024 * 1024
            mock_usage.total = 1000 * 1024 * 1024
            mock_usage.percent = 50.0
            mock_psutil.disk_usage.return_value = mock_usage

            report = main.collect_disk_usage()

            self.assertIn("Disk Usage Report", report)
            self.assertIn("/", report)
            self.assertIn("500 /", report)
            self.assertIn("1000 MB used (50.0%)", report)

    async def test_fetch_mcp_api_key_success_gcloud(self):
        with patch("main.fetch_mcp_api_key_gcloud") as mock_gcloud:
            mock_gcloud.return_value = "test-key"
            key = await main.fetch_mcp_api_key("test-project")
            self.assertEqual(key, "test-key")
            mock_gcloud.assert_called_once_with("test-project")

    async def test_fetch_mcp_api_key_fallback(self):
        with (
            patch("main.fetch_mcp_api_key_gcloud") as mock_gcloud,
            patch("main.fetch_mcp_api_key_library") as mock_library,
        ):
            mock_gcloud.side_effect = Exception("gcloud failed")
            mock_library.return_value = "test-key-lib"
            key = await main.fetch_mcp_api_key("test-project")
            self.assertEqual(key, "test-key-lib")
            mock_gcloud.assert_called_once()
            mock_library.assert_called_once()

    async def test_check_api_key_status_no_key(self):
        with patch.dict(os.environ, {}, clear=True):
            report, is_valid = await main.check_api_key_status([])
            self.assertIn("Provided Key:     [NOT FOUND]", report)
            self.assertFalse(is_valid)

    async def test_check_api_key_status_with_env_key_match(self):
        with (
            patch.dict(os.environ, {"MCP_API_KEY": "match-key"}),
            patch("main.get_project_id") as mock_get_id,
            patch("main.fetch_mcp_api_key") as mock_fetch,
        ):
            mock_get_id.return_value = "test-project"
            mock_fetch.return_value = "match-key"
            report, is_valid = await main.check_api_key_status([])
            self.assertIn("Provided Key:     [FOUND]", report)
            self.assertIn("Cloud Match:      [MATCHED]", report)
            self.assertTrue(is_valid)
            mock_fetch.assert_called_once_with("test-project")

    async def test_mcp_tools_registered(self):
        # Check if tools are registered with FastMCP
        tools = await main.mcp.list_tools()
        tool_names = [t.name for t in tools]
        self.assertIn("local_system_info", tool_names)
        self.assertIn("disk_usage", tool_names)

    async def test_local_system_info_tool(self):
        with patch("main.collect_system_info") as mock_collect:
            mock_collect.return_value = "mock system info"
            # Call the tool function directly
            result = await main.local_system_info()
            self.assertEqual(result, "mock system info")
            mock_collect.assert_called_once()

    async def test_disk_usage_tool(self):
        with patch("main.collect_disk_usage") as mock_collect:
            mock_collect.return_value = "mock disk usage"
            # Call the tool function directly
            result = await main.disk_usage()
            self.assertEqual(result, "mock disk usage")
            mock_collect.assert_called_once()


if __name__ == "__main__":
    unittest.main()
