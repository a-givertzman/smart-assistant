import logging as log

class Llm:
    def __init__(self, param1, param2) -> None:
        self.param1 = param1
        self.param2 = param2
        # Init here all required parameters, define a type for each
        self.dbg = "Llm"

    def handleQueries(self, query: str) -> str:
        log.debug(f'{self.dbg}.handleQueries | Query: {query}')
        #
        # Pass the query to the LLM here
        reply = f'Reply to query "{query}", LLM answer panding here later ...'
        log.debug(f'{self.dbg}.handleQueries | Reply: {reply}')
        #
        # And return the LLM reply here"
        return reply

    def run(self):
        #
        # Execute LLM here
        pass