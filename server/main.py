import logging as log

from llm.llm import Llm
from server.server import Server

dbg = "main"

def main():
    log.basicConfig(level = log.DEBUG, force = True)
    llm = Llm(param1="...", param2="...")
    llm.run()
    server = Server(
        "127.0.0.1", 8181,
        llm.handleQueries
    )
    server.run()

if __name__ == "__main__":
    main()