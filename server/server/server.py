import socket;
import logging as log
from typing import Callable


class Server:
    def __init__(self, addr: str, port: int, queries: Callable[[str], str]) -> None:
        """
        - addr: IP addres of the current server
        - port: port to be opened and listening for incoming connections
        - queries: Callback(query) -> reply where you can handle incoming query (text) and return the reply (text)
        """
        self.addr = addr
        self.port = port
        self.queries = queries
        self.dbg = "Server"
    def run(self):
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.bind((self.addr, self.port))
        server.listen()
        log.debug(f'{self.dbg}.run | Server redy on {self.addr}:{self.port}')
        while True:
            client, client_addr = server.accept()
            log.debug(f'{self.dbg}.run | Connection from {client_addr}')
            if client:
                bytes = client.recv(1024)
                query = bytes.decode(encoding="utf-8").strip()
                # log.debug(f'{self.dbg}.run | Query: {query}')
                reply = self.queries(query)
                # log.debug(f'{self.dbg}.run | Reply: {reply}')
                client.sendall(reply.encode(encoding="utf-8"))
                client.close()