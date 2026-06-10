import socketserver

class H(socketserver.StreamRequestHandler):
    def handle(self):
        for ln in self.rfile:
            self.wfile.write(ln.decode().upper().encode())

socketserver.ThreadingTCPServer(("", 8080), H).serve_forever()
