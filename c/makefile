CFLAGS = -Wall -Wextra -Wconversion -Werror -g -std=gnu17
CC = gcc

random: random.o
	$(CC) $(CFLAGS) -o $@ random.o

clean:
	-rm -f *.o random

.c.o:
	$(CC) $(CFLAGS) -c $<
