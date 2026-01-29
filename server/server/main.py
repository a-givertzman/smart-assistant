import logging as log

from server import Server

dbg = "main"

def handleQueries(query: str) -> str:
    log.debug(f'{dbg}.handleQueries | Query: {query}')
    #
    # Pass the query to the LLM here
    reply = f'Reply to query "{query}", LLM answer panding here later ...'
    log.debug(f'{dbg}.handleQueries | Reply: {reply}')
    #
    # And return the LLM reply here"
    return reply

def main():
    log.basicConfig(level = log.DEBUG, force = True)
    server = Server(
        "127.0.0.1", 8181,
        handleQueries
    )
    server.run()

if __name__ == "__main__":
    main()