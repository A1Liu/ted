# Intended Architecture

## NOTE: NONE OF THIS IS IMPLEMENTED

lexing/parsing -> one global lexer thread, X parser threads

1.  global thread lexes input
2.  sends relevant data off to a parser thread
    1. parser sends full AST data to global type checker thread when finished
    2. parser sends additional file paths to lexer

3.  goto 1


checking -> one global type database/thread, AST checking done by multiple threads

1.  wait on AST
2.  Get types from the AST and store in global database

    NOTE: Can this be done concurrently? Is there a reason to or not to?

3.  once all types are available, start up type checking threads


Write job system + allocator
Use the idea of Cancellation Token to allow for task cancellation

