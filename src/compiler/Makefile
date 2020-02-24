CFLAGS=-Wall -Wextra -std=c11 -g -Iinclude -I../include
SRCS=$(wildcard *.c)
OBJS=$(SRCS:.c=.o)
CC=clang-9
FORMAT=clang-format-9

build: $(OBJS)
	mv ./*.o ../

format:
	$(FORMAT) -i *.c include/*.h
