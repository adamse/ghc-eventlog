
{-# language ForeignFunctionInterface #-}

import Foreign.C.Types
import Control.Concurrent
import GHC.Conc.Sync

foreign import ccall "sleep" sleep :: CInt -> IO CInt
foreign import ccall unsafe "sleep" sleep2 :: CInt -> IO CInt

main = do
    me <- myThreadId
    labelThread me "main"
    yield

    forkIO $ do
        me <- myThreadId
        labelThread me "fork1"
        yield

        sleep2 2
        putStrLn "HJE"

    forkIO $ do
        me <- myThreadId
        labelThread me "fork2"
        yield
        sleep2 3
        putStrLn "HJE"

    sleep 5
    putStrLn "HEJ"
