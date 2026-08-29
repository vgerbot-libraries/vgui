#!/usr/bin/env python3
"""Simple HTTP server without COOP/COEP headers for static WASM testing."""
import http.server
import socketserver
import sys
import os

class PlainHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add CORS header for WASM files loaded from same origin
        self.send_header('Cross-Origin-Resource-Policy', 'same-origin')
        super().end_headers()

if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    dir = sys.argv[2] if len(sys.argv) > 2 else '.'
    os.chdir(dir)
    with socketserver.TCPServer(('127.0.0.1', port), PlainHandler) as httpd:
        print(f'Serving {dir} on http://127.0.0.1:{port}')
        httpd.serve_forever()
