import base64
import gzip
import io
import http.server
import socketserver
import os
from urllib.parse import parse_qs

PORT = 8888

class RequestHandler(http.server.SimpleHTTPRequestHandler):
    def do_POST(self):
        # Parse content length
        content_length = int(self.headers['Content-Length'])
        post_data = self.rfile.read(content_length)
        
        # Extract the filename from headers
        filename = self.headers.get('Filename')
        if not filename:
            self.send_response(400)
            self.end_headers()
            self.wfile.write(b'Filename header missing')
            return

        # Unzip and decode the data
        try:
            # Decompress the received data
            compressed_stream = io.BytesIO(post_data)
            with gzip.GzipFile(fileobj=compressed_stream) as gz_file:
                encoded_data = gz_file.read()

            # Decode from base64
            file_bytes = base64.b64decode(encoded_data)
            
            # Save the file with the correct extension
            with open(f"output/{filename}", 'wb') as f:
                f.write(file_bytes)

            self.send_response(200)
            self.end_headers()
            self.wfile.write(f"File saved as output{filename}".encode())
        except Exception as e:
            self.send_response(500)
            self.end_headers()
            self.wfile.write(f'Error: {str(e)}'.encode())

def run(server_class=http.server.HTTPServer, handler_class=RequestHandler):
    os.makedirs("output",exist_ok=True)
    server_address = ('0.0.0.0', PORT)
    httpd = server_class(server_address, handler_class)
    print(f'Starting server on port {PORT}...')
    httpd.serve_forever()

if __name__ == '__main__':
    run()
