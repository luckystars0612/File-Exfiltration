import base64
import gzip
import io
import http.server
import os
import sys

PORT = 8888

class RequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.total_bytes_received = 0
        self.expected_total_size = None  # Optional: Track expected size if known
        super().__init__(*args, **kwargs)

    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        chunk_index = int(self.headers.get('Chunk-Index', -1))
        filename = self.headers.get('Filename')

        if not filename or chunk_index == -1:
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b'Missing headers: Filename or Chunk-Index')
            return

        # Read data
        post_data = self.rfile.read(content_length)

        # Update received bytes and display progress
        self.total_bytes_received += content_length
        sys.stdout.write(
            f"\rReceived: {self.total_bytes_received} bytes"
        )
        sys.stdout.flush()

        # Process the chunk
        try:
            compressed_stream = io.BytesIO(post_data)
            with gzip.GzipFile(fileobj=compressed_stream) as gz_file:
                encoded_data = gz_file.read()

            file_bytes = base64.b64decode(encoded_data)

            # Save the data
            os.makedirs("output", exist_ok=True)
            with open(f"output/{filename}", 'ab') as f:
                f.write(file_bytes)

            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'Chunk received successfully.')
        except Exception as e:
            self.send_response(500)
            self.end_headers()
            self.wfile.write(f"Error: {str(e)}".encode())

    def log_message(self, format, *args):
        """Suppress default HTTP server logging."""
        pass

def run(server_class=http.server.HTTPServer, handler_class=RequestHandler):
    os.makedirs("output", exist_ok=True)
    server_address = ('0.0.0.0', PORT)
    httpd = server_class(server_address, handler_class)
    print(f"Starting server on port {PORT}...")
    httpd.serve_forever()

if __name__ == '__main__':
    try:
        run()
    except KeyboardInterrupt:
        print("\nServer stopped.")
