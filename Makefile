
all: main.eventlog

main: main.hs
	ghc +RTS -la -RTS --make main.hs -dynamic -j10 -fforce-recomp -threaded -eventlog -rtsopts

main.eventlog: main
	./main +RTS -la -k1M -A10M

clean:
	$(RM) main main.hi 
	$(RM) *.o 
	$(RM) *.eventlog
