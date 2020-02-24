CFLAGS=-Wall -Wextra -std=c11 -g -Isrc/include
SRCS=$(wildcard src/*.c)
OBJS=$(SRCS:.c=.o)
CC=clang-9
FORMAT=clang-format-9

dagc: compiler $(OBJS)
	$(CC) -o $@ src/*.o $(CFLAGS)

$(OBJS): 

compiler: 
	make -C src/compiler

clean:
	rm -f core dagc src/*.o *.txt *~ a.out test/*~ *.s

format:
	make -C src/compiler format

.PHONY: clean format
