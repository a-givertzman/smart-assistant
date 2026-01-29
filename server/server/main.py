import logging as log

from server.server import Server

dbg = "main"

def handleQueries(query: str) -> str:
    log.debug(f'{dbg}.handleQueries | Query: {query}')
    reply = "Pass the query to the LLM and return the LLM reply here"
    return reply,


if __name__ == "__main__":
    log.basicConfig(level = log.DEBUG, force = True)
    server = Server(
        "127.0.0.1", 8181,
        handleQueries
    )
    server.run()
