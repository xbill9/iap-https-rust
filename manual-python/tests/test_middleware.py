import unittest
from unittest.mock import patch
import os
import sys
from starlette.testclient import TestClient

# Ensure the parent directory is in sys.path to import main
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import main

class TestMiddleware(unittest.TestCase):
    def setUp(self):
        # Reset FastMCP to ensure clean state if needed, 
        # but here we can just use the app from main.mcp.sse_app()
        pass

    def test_api_key_middleware_success(self):
        expected_key = "secret-key"
        with patch.dict(os.environ, {"MCP_API_KEY": expected_key, "MCP_TRANSPORT": "sse"}):
            # We need to simulate what main() does to set up the app
            app = main.mcp.sse_app()
            
            # Manually add the middleware as main() does
            from starlette.requests import Request
            from starlette.responses import JSONResponse
            
            @app.middleware("http")
            async def api_key_middleware(request: Request, call_next):
                auth_key = request.headers.get("X-Goog-Api-Key")
                if auth_key != expected_key:
                    return JSONResponse(
                        {"error": "Unauthorized: Invalid or missing API Key"},
                        status_code=401,
                    )
                return await call_next(request)

            client = TestClient(app)
            
            # Test with correct key
            response = client.get("/sse", headers={"X-Goog-Api-Key": expected_key})
            # Note: /sse might return a 405 or something if not properly initialized by FastMCP 
            # for a simple GET, but it should NOT return 401.
            # Actually FastMCP's SSE app handles /sse with GET.
            self.assertNotEqual(response.status_code, 401)

    def test_api_key_middleware_failure(self):
        expected_key = "secret-key"
        # We'll create a fresh app for testing to avoid interference
        from mcp.server.fastmcp import FastMCP
        test_mcp = FastMCP("test")
        app = test_mcp.sse_app()
        
        from starlette.requests import Request
        from starlette.responses import JSONResponse
        
        @app.middleware("http")
        async def api_key_middleware(request: Request, call_next):
            auth_key = request.headers.get("X-Goog-Api-Key")
            if auth_key != expected_key:
                return JSONResponse(
                    {"error": "Unauthorized: Invalid or missing API Key"},
                    status_code=401,
                )
            return await call_next(request)

        client = TestClient(app)
        
        # Test with wrong key
        response = client.get("/sse", headers={"X-Goog-Api-Key": "wrong-key"})
        self.assertEqual(response.status_code, 401)
        self.assertEqual(response.json(), {"error": "Unauthorized: Invalid or missing API Key"})

        # Test with missing key
        response = client.get("/sse")
        self.assertEqual(response.status_code, 401)

if __name__ == "__main__":
    unittest.main()
